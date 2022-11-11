use std::{cell::Cell, ptr::NonNull};

use crate::Trace;

#[allow(dead_code)]
struct GcState {
    stats: GcStats,
    config: GcConfig,
    box_start: Cell<Option<NonNull<GcBox<dyn Trace>>>>,
}

impl Drop for GcState {
    fn drop(&mut self) {
        if !self.config.leak_on_drop {
            collect_garbage();
        }
    }
}

fn collect_garbage() {
    todo!()
}

pub struct GcStats {
    /// The number of bytes allocated by the GC
    pub bytes_allocated: usize,
    /// Collections since the last time the stats were reset
    pub collections_perfomed: usize,
}

impl GcStats {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            bytes_allocated: 0,
            collections_perfomed: 0,
        }
    }
}

impl Default for GcStats {
    fn default() -> Self {
        Self {
            bytes_allocated: 0,
            collections_perfomed: 0,
        }
    }
}

pub struct GcConfig {
    /// The threshold at which the GC will run
    pub threshold: usize,
    /// After collection we want to he ratio of used/total to be no more than this
    pub used_space_ratio: f64,
    /// For short running processes it is not worth it to run the GC
    pub leak_on_drop: bool,
}

impl GcConfig {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            threshold: 100,
            used_space_ratio: 0.8,
            leak_on_drop: false,
        }
    }
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            threshold: 100,
            used_space_ratio: 0.8,
            leak_on_drop: false,
        }
    }
}

#[allow(dead_code)]
pub struct GcBoxHeader {
    roots: Cell<usize>,
    next: Cell<Option<NonNull<GcBox<dyn Trace>>>>,
}

impl GcBoxHeader {
    #[allow(dead_code)]
    #[inline]
    pub fn new(next: Option<NonNull<GcBox<dyn Trace>>>) -> Self {
        Self {
            roots: Cell::new(0),
            next: Cell::new(next),
        }
    }

    #[inline]
    fn root(&self) -> usize {
        self.roots.get()
    }

    #[inline]
    fn inc_root(&self) {
        self.roots.set(self.roots.get() + 1)
    }

    #[inline]
    fn dec_root(&self) {
        self.roots.set(self.roots.get() - 1)
    }

    #[inline]
    fn is_marked(&self) -> bool {
        todo!()
    }

    #[inline]
    fn mark(&self) {
        todo!()
    }

    #[inline]
    fn unmark(&self) {
        todo!()
    }
}

impl Default for GcBoxHeader {
    fn default() -> Self {
        Self {
            roots: Cell::new(0),
            next: Cell::new(None),
        }
    }
}

#[allow(dead_code)]
#[repr(C)]
pub struct GcBox<T: Trace + ?Sized> {
    header: GcBoxHeader,
    data: T,
}
