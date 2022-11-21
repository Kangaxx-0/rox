/// A trait which needs to be implemented on garbage collected
pub trait Finalize {
    /// finalize is called when the object is about to be dropped
    fn finalize(&self) {}
}

/// A trait for types that can be traced by the garbage collector.
pub unsafe trait Trace: Finalize {
    /// # Safety
    ///
    /// Marks all contained `Gc`s
    unsafe fn trace(&self);

    /// # Safety
    ///
    /// Increments the root count of all contained `Gc`, it should only be called only to avoid double root.
    unsafe fn root(&self);

    /// # Safety
    ///
    /// Decrements the root count of all contained `Gc`
    unsafe fn unroot(&self);

    /// # Safety
    ///
    /// Runs Finalize::finalize on all contained `Gc`
    fn finalize_glue(&self);
}

macro_rules! unsafe_empty_trace {
    () => {
        #[inline]
        unsafe fn trace(&self) {}

        #[inline]
        unsafe fn root(&self) {}

        #[inline]
        unsafe fn unroot(&self) {}

        #[inline]
        fn finalize_glue(&self) {
            $crate::Finalize::finalize(self)
        }
    };
}

impl<T> Finalize for &'static T {}
unsafe impl<T> Trace for &'static T {
    unsafe_empty_trace!();
}

/// Simply implements `Trace` for primitive types that being used for `Rox`
macro_rules! simple_empty_finalize_trace {
    ($($t:ty),*) => {
        $(
            impl Finalize for $t {}
            unsafe impl Trace for $t {
                unsafe_empty_trace!(); }
        )*
    };
}

simple_empty_finalize_trace![(), bool, isize, usize, u8, u16, f64, u64, String, Box<str>];

macro_rules! custom_trace {
    ($this:ident, $body:expr) => {
        #[inline]
        unsafe fn trace(&self) {
            #[inline]
            unsafe fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                $crate::Trace::trace(it);
            }
            let $this = self;
            $body
        }
        #[inline]
        unsafe fn root(&self) {
            #[inline]
            unsafe fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                $crate::Trace::root(it);
            }
            let $this = self;
            $body
        }
        #[inline]
        unsafe fn unroot(&self) {
            #[inline]
            unsafe fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                $crate::Trace::unroot(it);
            }
            let $this = self;
            $body
        }
        #[inline]
        fn finalize_glue(&self) {
            $crate::Finalize::finalize(self);
            #[inline]
            fn mark<T: $crate::Trace + ?Sized>(it: &T) {
                $crate::Trace::finalize_glue(it);
            }
            let $this = self;
            $body
        }
    };
}
impl<T: Trace, const N: usize> Finalize for [T; N] {}
unsafe impl<T: Trace, const N: usize> Trace for [T; N] {
    custom_trace!(this, {
        for v in this {
            mark(v);
        }
    });
}

impl<T: Trace + ?Sized> Finalize for Box<T> {}
unsafe impl<T: Trace + ?Sized> Trace for Box<T> {
    custom_trace!(this, {
        mark(&**this);
    });
}

impl<T: Trace> Finalize for Vec<T> {}
unsafe impl<T: Trace> Trace for Vec<T> {
    custom_trace!(this, {
        for it in this {
            mark(it);
        }
    });
}

impl<T: Trace> Finalize for Option<T> {}
unsafe impl<T: Trace> Trace for Option<T> {
    custom_trace!(this, {
        if let Some(it) = this {
            mark(it);
        }
    });
}

impl<T: Trace, E: Trace> Finalize for Result<T, E> {}
unsafe impl<T: Trace, E: Trace> Trace for Result<T, E> {
    custom_trace!(this, {
        match this {
            Ok(it) => mark(it),
            Err(it) => mark(it),
        }
    });
}
