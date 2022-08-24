use std::marker::PhantomData;
use std::ptr::NonNull;

/*
We need an array of bytes, since we don't know how big the array needs to be before we start
compiling a chunk, it must be dynamic,so we want to implement a Vec for Lox
    - Cache-friendly, dense storage
    - Constant-time indexed element lookup
    - Constant-time appending to the end of the array
*/
pub struct Lec<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
    _maker: PhantomData<T>,
}

unsafe impl<T: Send> Send for Lec<T> {}
unsafe impl<T: Sync> Sync for Lec<T> {}
