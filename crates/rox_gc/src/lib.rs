mod gc;
mod trace;

use std::{
    cell::Cell,
    marker::PhantomData,
    mem,
    ptr::{self, NonNull},
    rc::Rc,
};

use crate::gc::{finalizer_safe, GcBox};
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

// Sets the data pointer of a `?Sized` raw pointer.
//
// For a slice/trait object, this sets the `data` field and leaves the rest unchanged.
// For a sized raw pointer, this simply sets the pointer.
unsafe fn set_data_ptr<T: ?Sized, U>(mut ptr: *mut T, data: *mut U) -> *mut T {
    ptr::write(&mut ptr as *mut _ as *mut *mut u8, data as *mut u8);
    ptr
}
