use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};
use std::{isize, mem};

/*
We need an array of bytes, since we don't know how big the array needs to be before we start
compiling a chunk, it must be dynamic,so we want to implement a Vec for Lox
    - Cache-friendly, dense storage
    - Constant-time indexed element lookup
    - Constant-time appending to the end of the array
*/

pub struct Vec<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
    _maker: PhantomData<T>,
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

// for warning's sake
#[allow(dead_code)]
impl<T> Vec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        Vec {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
            _maker: PhantomData,
        }
    }

    pub fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).expect("Unable to get layout"))
        } else {
            // this can't overflow since self.cap <= isize.Max.
            let new_cap = 2 * self.cap;

            let new_layout = Layout::array::<T>(new_cap).expect("Unable to get layout");
            (new_cap, new_layout)
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).expect("Unable to get layout");
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { realloc(old_ptr, old_layout, new_layout.size()) }
        };

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            // Instead of unwinding, we choose to abort here.
            None => handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            /*
            We don't want to either evaluation or drop involved
            If the Vec length is 10, then we want to write the 10th index for push value
            */
            ptr::write(self.ptr.as_ptr().add(self.len), value);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.len))) }
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            while self.pop().is_some() {}
            let layout = Layout::array::<T>(self.cap).expect("Unable to get layout");
            unsafe {
                dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let lec: Vec<u8> = Vec::new();

        assert_eq!(0, lec.len);
        assert_eq!(0, lec.len());
        assert_eq!(0, lec.capacity());
    }

    #[test]
    fn push_and_pop() {
        let mut lec: Vec<u8> = Vec::new();

        assert_eq!(0, lec.len());
        assert_eq!(0, lec.capacity());
        lec.push(1);
        assert_eq!(1, lec.len());
        assert_eq!(1, lec.capacity());
        lec.push(2);
        assert_eq!(2, lec.len());
        assert_eq!(2, lec.capacity());
        lec.push(3);
        assert_eq!(3, lec.len());
        assert_eq!(4, lec.capacity());
        let value = lec.pop();
        assert_eq!(3, value.unwrap());
        assert_eq!(2, lec.len());
        assert_eq!(4, lec.capacity());
    }
}
