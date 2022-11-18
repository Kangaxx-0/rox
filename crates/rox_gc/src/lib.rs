mod gc;
mod trace;

use core::fmt;
use std::{
    cell::{Cell, UnsafeCell},
    cmp::Ordering,
    fmt::Display,
    hash::{self, Hasher},
    mem,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

pub use crate::gc::{finalizer_safe, GcBox};
pub use crate::trace::{Finalize, Trace};

pub struct Gc<T: Trace + ?Sized + 'static> {
    ptr_root: Cell<NonNull<GcBox<T>>>,
    // marker: PhantomData<Rc<T>>,
}

impl<T: Trace> Gc<T> {
    /// Constructs a new `Gc<T>` with the given value.
    ///
    /// # Collection
    ///
    /// This method could trigger a garbage collection.
    ///
    /// # Examples
    ///
    /// ```

    /// use gc::Gc;
    ///
    /// let five = Gc::new(5);
    /// assert_eq!(*five, 5);
    /// ```
    pub fn new(value: T) -> Self {
        assert!(mem::align_of::<Gc<T>>() > 1);

        unsafe {
            // Allocate the memory for the object.
            let ptr = GcBox::new(value);

            // When we create a Gc<T>. all pointers which have been moved to the heap
            // no longer need to be rooted, so we unroot them.
            (*ptr.as_ptr()).value().unroot();
            let gc = Gc {
                ptr_root: Cell::new(NonNull::new_unchecked(ptr.as_ptr())),
            };

            gc.set_root();
            gc
        }
    }
}

unsafe fn clear_root_bit<T: Trace + ?Sized>(ptr: NonNull<GcBox<T>>) -> NonNull<GcBox<T>> {
    // Calculate the address of the GcBox which needs to be passed to `set_data_ptr`.
    let ptr = ptr.as_ptr();
    let data = ptr as *mut u8;
    let addr = data as isize;
    let ptr = set_data_ptr(ptr, data.wrapping_offset((addr & !1) - addr));
    NonNull::new_unchecked(ptr)
}

impl<T: Trace + ?Sized> Gc<T> {
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        GcBox::ptr_eq(this.inner(), other.inner())
    }

    fn rooted(&self) -> bool {
        self.ptr_root.get().as_ptr() as *mut u8 as usize & 1 != 0
    }

    unsafe fn set_root(&self) {
        // Calculate the address of the GcBox which needs to be passed to `set_data_ptr`.
        let ptr = self.ptr_root.get().as_ptr();
        let data = ptr as *mut u8;
        let addr = data as isize;
        let ptr = set_data_ptr(ptr, data.wrapping_offset((addr | 1) - addr));
        self.ptr_root.set(NonNull::new_unchecked(ptr));
    }

    unsafe fn clear_root(&self) {
        let val = clear_root_bit(self.ptr_root.get());
        self.ptr_root.set(val);
    }

    #[inline]
    fn inner_ptr(&self) -> *mut GcBox<T> {
        // If we are currently in the dropping phase of gc,
        // it would be undefined behavior to dereference the pointer.
        // By opting into `Trace` you are agreeing to not dereference the pointer
        // within your drop method, meaning that it should be safe.
        //
        // This assert exists just in case.
        assert!(finalizer_safe());

        unsafe { clear_root_bit(self.ptr_root.get()).as_ptr() }
    }

    #[inline]
    fn inner(&self) -> &GcBox<T> {
        unsafe { &*self.inner_ptr() }
    }
}

// Default implementation of `Finalize` for `Gc<T>`.
impl<T: Trace + ?Sized> Finalize for Gc<T> {}

// Default implementation of `Trace` for `Gc<T>`.
unsafe impl<T: Trace + ?Sized> Trace for Gc<T> {
    #[inline]
    unsafe fn trace(&self) {
        self.inner().trace_inner();
    }

    #[inline]
    unsafe fn root(&self) {
        assert!(!self.rooted(), "Cannot double root a Gc pointer");

        // Try to get inner before modifying out state. Inner may be inaccessable due to this
        // method being invoked during the sweeping phase, and we don't want to modify our state
        // before panicking.
        self.inner().root_inner();

        self.set_root();
    }

    #[inline]
    unsafe fn unroot(&self) {
        assert!(!self.rooted(), "Cannot double root a Gc pointer");

        // Try to get inner before modifying out state. Inner may be inaccessable due to this
        // method being invoked during the sweeping phase, and we don't want to modify our state
        // before panicking.
        self.inner().unroot_inner();

        self.clear_root();
    }

    #[inline]
    fn finalize_glue(&self) {
        Finalize::finalize(self);
    }
}

impl<T: Trace + ?Sized> Clone for Gc<T> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            self.inner().root_inner();
            let gc = Gc {
                ptr_root: Cell::new(self.ptr_root.get()),
            };
            gc.set_root();
            gc
        }
    }
}

impl<T: Trace + ?Sized> Deref for Gc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.inner().value()
    }
}

impl<T: Trace + ?Sized> Drop for Gc<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.inner().unroot_inner();
        }
    }
}

impl<T: Trace + Default> Default for Gc<T> {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Trace + ?Sized + fmt::Debug> fmt::Debug for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: Trace + ?Sized + fmt::Display> fmt::Display for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: Trace + ?Sized + PartialEq> PartialEq for Gc<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Trace + ?Sized + Eq> Eq for Gc<T> {}

impl<T: Trace + ?Sized + PartialOrd> PartialOrd for Gc<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }

    #[inline(always)]
    fn lt(&self, other: &Self) -> bool {
        **self < **other
    }

    #[inline(always)]
    fn le(&self, other: &Self) -> bool {
        **self <= **other
    }

    #[inline(always)]
    fn gt(&self, other: &Self) -> bool {
        **self > **other
    }

    #[inline(always)]
    fn ge(&self, other: &Self) -> bool {
        **self >= **other
    }
}

impl<T: Trace + ?Sized + Ord> Ord for Gc<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<T: Trace + ?Sized + hash::Hash> hash::Hash for Gc<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl<T: Trace> From<T> for Gc<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Trace + ?Sized> std::borrow::Borrow<T> for Gc<T> {
    #[inline]
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T: Trace + ?Sized> std::convert::AsRef<T> for Gc<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &**self
    }
}

//////////////////////////////////////////////////////////////////////////////
// GcCell //
//////////////////////////////////////////////////////////////////////////////

/// The BorrowFlag used by GC is split into two parts:
/// 1. The upper 63 or 31 bits are used to store the number of borrowed references to the type.
/// 2. The lower bit is used to record the rootedness of the type.
#[derive(Clone, Copy)]
struct BorrowFlag(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BorrowState {
    Reading,
    Writing,
    Unused,
}

const ROOT: usize = 1;
const WRITING: usize = !1;
const UNUSED: usize = 0;

/// The base borrowflag init is rooted, and has no outstanding borrows.
const BOF_INIT: BorrowFlag = BorrowFlag(ROOT);

impl BorrowFlag {
    fn borrowed(self) -> BorrowState {
        match self.0 {
            WRITING => BorrowState::Writing,
            UNUSED => BorrowState::Unused,
            _ => BorrowState::Reading,
        }
    }

    fn rooted(self) -> bool {
        match self.0 & ROOT {
            ROOT => true,
            _ => false,
        }
    }

    fn set_writing(self) -> Self {
        // Set every bit other than the root bit, which is preserved.
        BorrowFlag(self.0 | WRITING)
    }

    fn set_unused(self) -> Self {
        // Set every bit other than the root bit, which is preserved.
        BorrowFlag(self.0 & ROOT)
    }

    fn add_reading(self) -> Self {
        assert!(self.borrowed() != BorrowState::Writing);
        // Add one to the integer starting at the second binary digit. As our borrowstate is not
        // writing, we know that overflow cannot happen, so this is equivalent to the following,
        // more complicated, code:
        //
        // BorrowFlag(self.0 & ROOT | (((self.0 >> 1) + 1) << 1))

        BorrowFlag(self.0 + 0b10)
    }

    fn sub_reading(self) -> Self {
        assert!(self.borrowed() == BorrowState::Reading);
        // Subtract 1 from the integer starting at the second binary digit. As
        // our borrowstate is not writing or unused, we know that overflow or
        // undeflow cannot happen, so this is equivalent to the following, more
        // complicated, expression:
        //
        // BorrowFlag((self.0 & ROOT) | (((self.0 >> 1) - 1) << 1))
        BorrowFlag(self.0 - 0b10)
    }

    fn set_rooted(self, rooted: bool) -> Self {
        // Preserve the non-root bits
        BorrowFlag((self.0 & !ROOT) | (rooted as usize))
    }
}

/// A mutable memory location with dynamically checked borrow rules that can be used inside of a gc
/// pointer.
pub struct GcCell<T: ?Sized> {
    flags: Cell<BorrowFlag>,
    cell: UnsafeCell<T>,
}

impl<T: Trace> GcCell<T> {
    /// Creates a new `GcCell` containing `value`.
    #[inline]
    pub fn new(value: T) -> GcCell<T> {
        GcCell {
            flags: Cell::new(BOF_INIT),
            cell: UnsafeCell::new(value),
        }
    }

    /// Consumes the `GcCell`, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.cell.into_inner()
    }
}

impl<T: Trace + ?Sized> GcCell<T> {
    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `GcCellRef` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    #[inline]
    pub fn borrow(&self) -> GcCellRef<'_, T> {
        match self.try_borrow() {
            Ok(value) => value,
            Err(e) => panic!("{}", e),
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `GcCellRefMut` exits scope.
    /// The value cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    #[inline]
    pub fn borrow_mut(&self) -> GcCellRefMut<'_, T> {
        match self.try_borrow_mut() {
            Ok(value) => value,
            Err(e) => panic!("{}", e),
        }
    }

    /// Immutably borrows the wrapped value, returning an error if the value is currently mutably
    /// borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRef` exits scope. Multiple immutable borrows can be
    /// taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::GcCell;
    ///
    /// let c = GcCell::new(5);
    ///
    /// {
    ///     let m = c.borrow_mut();
    ///     assert!(c.try_borrow().is_err());
    /// }
    ///
    /// {
    ///     let m = c.borrow();
    ///     assert!(c.try_borrow().is_ok());
    /// }
    /// ```
    pub fn try_borrow(&self) -> Result<GcCellRef<'_, T>, BorrowError> {
        if self.flags.get().borrowed() == BorrowState::Writing {
            return Err(BorrowError);
        }
        self.flags.set(self.flags.get().add_reading());

        // This will fail if the borrow count overflows, which shouldn't happen,
        // but let's be safe
        assert!(self.flags.get().borrowed() == BorrowState::Reading);

        unsafe {
            Ok(GcCellRef {
                flags: &self.flags,
                value: &*self.cell.get(),
            })
        }
    }

    /// Mutably borrows the wrapped value, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRefMut` exits scope.
    /// The value cannot be borrowed while this borrow is active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::GcCell;
    ///
    /// let c = GcCell::new(5);
    ///
    /// {
    ///     let m = c.borrow();
    ///     assert!(c.try_borrow_mut().is_err());
    /// }
    ///
    /// assert!(c.try_borrow_mut().is_ok());
    /// ```
    pub fn try_borrow_mut(&self) -> Result<GcCellRefMut<'_, T>, BorrowMutError> {
        if self.flags.get().borrowed() != BorrowState::Unused {
            return Err(BorrowMutError);
        }
        self.flags.set(self.flags.get().set_writing());

        unsafe {
            // Force the val_ref's contents to be rooted for the duration of the
            // mutable borrow
            if !self.flags.get().rooted() {
                (*self.cell.get()).root();
            }

            Ok(GcCellRefMut {
                gc_cell: self,
                value: &mut *self.cell.get(),
            })
        }
    }
}

impl<T: Trace + ?Sized> Finalize for GcCell<T> {}

unsafe impl<T: Trace + ?Sized> Trace for GcCell<T> {
    #[inline]
    unsafe fn trace(&self) {
        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => (*self.cell.get()).trace(),
        }
    }

    #[inline]
    unsafe fn root(&self) {
        assert!(!self.flags.get().rooted(), "Can't root a GcCell twice!");
        self.flags.set(self.flags.get().set_rooted(true));

        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => (*self.cell.get()).root(),
        }
    }

    #[inline]
    unsafe fn unroot(&self) {
        assert!(self.flags.get().rooted(), "Can't unroot a GcCell twice!");
        self.flags.set(self.flags.get().set_rooted(false));

        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => (*self.cell.get()).unroot(),
        }
    }

    #[inline]
    fn finalize_glue(&self) {
        Finalize::finalize(self);
        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => unsafe { (*self.cell.get()).finalize_glue() },
        }
    }
}

impl<T: Trace + Clone> Clone for GcCell<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.borrow().clone())
    }
}

impl<T: Trace + Default> Default for GcCell<T> {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Trace + ?Sized + PartialEq> PartialEq for GcCell<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: Trace + ?Sized + Eq> Eq for GcCell<T> {}

impl<T: Trace + ?Sized + PartialOrd> PartialOrd for GcCell<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (*self.borrow()).partial_cmp(&*other.borrow())
    }

    #[inline(always)]
    fn lt(&self, other: &Self) -> bool {
        *self.borrow() < *other.borrow()
    }

    #[inline(always)]
    fn le(&self, other: &Self) -> bool {
        *self.borrow() <= *other.borrow()
    }

    #[inline(always)]
    fn gt(&self, other: &Self) -> bool {
        *self.borrow() > *other.borrow()
    }

    #[inline(always)]
    fn ge(&self, other: &Self) -> bool {
        *self.borrow() >= *other.borrow()
    }
}

impl<T: Trace + ?Sized + Ord> Ord for GcCell<T> {
    #[inline]
    fn cmp(&self, other: &GcCell<T>) -> Ordering {
        (*self.borrow()).cmp(&*other.borrow())
    }
}

impl<T: Trace + ?Sized + fmt::Debug> fmt::Debug for GcCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.flags.get().borrowed() {
            BorrowState::Unused | BorrowState::Reading => f
                .debug_struct("GcCell")
                .field("value", &self.borrow())
                .finish(),
            BorrowState::Writing => f
                .debug_struct("GcCell")
                .field("value", &"<borrowed>")
                .finish(),
        }
    }
}

// An error returned by [`GcCell::try_borrow`](struct.GcCell.html#method.try_borrow).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct BorrowError;

impl std::fmt::Display for BorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt("GcCell<T> already mutably borrowed", f)
    }
}

/// An error returned by [`GcCell::try_borrow_mut`](struct.GcCell.html#method.try_borrow_mut).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct BorrowMutError;

impl std::fmt::Display for BorrowMutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt("GcCell<T> already borrowed", f)
    }
}

/// A wrapper type for an immutable borrow of a value in a `GcCell` over T.
pub struct GcCellRef<'a, T: ?Sized + 'a> {
    flags: &'a Cell<BorrowFlag>,
    value: &'a T,
}

impl<'a, T: ?Sized> GcCellRef<'a, T> {
    /// Copies a `GcCellRef`.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as `GcCellRef::clone(...)`. A method
    /// would interfere with the use of `c.borrow().clone()` to clone the contents of the `GcCell`.
    #[inline]
    pub fn clone(orig: &GcCellRef<'a, T>) -> GcCellRef<'a, T> {
        orig.flags.set(orig.flags.get().add_reading());
        GcCellRef {
            flags: orig.flags,
            value: orig.value,
        }
    }

    pub fn map<U, F>(orig: Self, f: F) -> GcCellRef<'a, U>
    where
        U: ?Sized,
        F: FnOnce(&T) -> &U,
    {
        let ret = GcCellRef {
            flags: orig.flags,
            value: f(orig.value),
        };

        std::mem::forget(orig);

        ret
    }

    /// Splits a `GcCellRef` into multiple `GcCellRef`s for different components of the borrowed data.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as GcCellRef::map_split(...).
    /// A method would interfere with methods of the same name on the contents of a `GcCellRef` used through `Deref`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::{GcCell, GcCellRef};
    ///
    /// let cell = GcCell::new((1, 'c'));
    /// let borrow = cell.borrow();
    /// let (first, second) = GcCellRef::map_split(borrow, |x| (&x.0, &x.1));
    /// assert_eq!(*first, 1);
    /// assert_eq!(*second, 'c');
    /// ```
    #[inline]
    pub fn map_split<U, V, F>(orig: Self, f: F) -> (GcCellRef<'a, U>, GcCellRef<'a, V>)
    where
        U: ?Sized,
        V: ?Sized,
        F: FnOnce(&T) -> (&U, &V),
    {
        let (a, b) = f(orig.value);

        orig.flags.set(orig.flags.get().add_reading());

        let ret = (
            GcCellRef {
                flags: orig.flags,
                value: a,
            },
            GcCellRef {
                flags: orig.flags,
                value: b,
            },
        );

        // We have to tell the compiler not to call the destructor of GcCellRef,
        // because it will update the borrow flags.
        std::mem::forget(orig);

        ret
    }
}

impl<'a, T: ?Sized> Deref for GcCellRef<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<'a, T: ?Sized> Drop for GcCellRef<'a, T> {
    fn drop(&mut self) {
        debug_assert!(self.flags.get().borrowed() == BorrowState::Reading);
        self.flags.set(self.flags.get().sub_reading());
    }
}

impl<'a, T: ?Sized + fmt::Debug> fmt::Debug for GcCellRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + Display> Display for GcCellRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

pub struct GcCellRefMut<'a, T: Trace + ?Sized + 'static, U: ?Sized = T> {
    gc_cell: &'a GcCell<T>,
    value: &'a mut U,
}

impl<'a, T: Trace + ?Sized, U: ?Sized> Deref for GcCellRefMut<'a, T, U> {
    type Target = U;

    #[inline]
    fn deref(&self) -> &U {
        self.value
    }
}

impl<'a, T: Trace + ?Sized, U: ?Sized> DerefMut for GcCellRefMut<'a, T, U> {
    #[inline]
    fn deref_mut(&mut self) -> &mut U {
        self.value
    }
}

impl<'a, T: Trace + ?Sized, U: ?Sized> Drop for GcCellRefMut<'a, T, U> {
    #[inline]
    fn drop(&mut self) {
        debug_assert!(self.gc_cell.flags.get().borrowed() == BorrowState::Writing);
        // Restore the rooted state of the GcCell's contents to the state of the GcCell.
        // During the lifetime of the GcCellRefMut, the GcCell's contents are rooted.
        if !self.gc_cell.flags.get().rooted() {
            unsafe {
                (*self.gc_cell.cell.get()).unroot();
            }
        }
        self.gc_cell
            .flags
            .set(self.gc_cell.flags.get().set_unused());
    }
}

impl<'a, T: Trace + ?Sized, U: fmt::Debug + ?Sized> fmt::Debug for GcCellRefMut<'a, T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*(self.deref()), f)
    }
}

impl<'a, T: Trace + ?Sized, U: Display + ?Sized> Display for GcCellRefMut<'a, T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}
impl<'a, T: Trace + ?Sized, U: ?Sized> GcCellRefMut<'a, T, U> {
    /// Makes a new `GcCellRefMut` for a component of the borrowed data, e.g., an enum
    /// variant.
    ///
    /// The `GcCellRefMut` is already mutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `GcCellRefMut::map(...)`. A method would interfere with methods of the same
    /// name on the contents of a `GcCell` used through `Deref`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::{GcCell, GcCellRefMut};
    ///
    /// let c = GcCell::new((5, 'b'));
    /// {
    ///     let b1: GcCellRefMut<(u32, char)> = c.borrow_mut();
    ///     let mut b2: GcCellRefMut<(u32, char), u32> = GcCellRefMut::map(b1, |t| &mut t.0);
    ///     assert_eq!(*b2, 5);
    ///     *b2 = 42;
    /// }
    /// assert_eq!(*c.borrow(), (42, 'b'));
    /// ```
    #[inline]
    pub fn map<V, F>(orig: Self, f: F) -> GcCellRefMut<'a, T, V>
    where
        V: ?Sized,
        F: FnOnce(&mut U) -> &mut V,
    {
        let value = unsafe { &mut *(orig.value as *mut U) };

        let ret = GcCellRefMut {
            gc_cell: orig.gc_cell,
            value: f(value),
        };

        // We have to tell the compiler not to call the destructor of GcCellRefMut,
        // because it will update the borrow flags.

        std::mem::forget(orig);

        ret
    }
}

// Sets the data pointer of a `?Sized` raw pointer.
//
// For a slice/trait object, this sets the `data` field and leaves the rest unchanged.
// For a sized raw pointer, this simply sets the pointer.
unsafe fn set_data_ptr<T: ?Sized, U>(mut ptr: *mut T, data: *mut U) -> *mut T {
    ptr::write(&mut ptr as *mut _ as *mut *mut u8, data as *mut u8);
    ptr
}
