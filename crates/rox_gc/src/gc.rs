use std::{
    cell::{Cell, RefCell},
    mem,
    ptr::{self, NonNull},
};

use crate::Trace;

struct GcState {
    stats: GcStats,
    config: GcConfig,
    box_start: Cell<Option<NonNull<GcBox<dyn Trace>>>>,
}

impl Drop for GcState {
    fn drop(&mut self) {
        if !self.config.leak_on_drop {
            collect_garbage(self);
        }
    }
}

thread_local! { pub static GC_DROP: Cell<bool> = Cell::new(false); }
struct DropGuard;

impl DropGuard {
    fn new() -> Self {
        GC_DROP.with(|drop| drop.set(true));
        Self
    }
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        GC_DROP.with(|drop| drop.set(false));
    }
}

pub fn finalizer_safe() -> bool {
    GC_DROP.with(|drop| !drop.get())
}

const MARK_MASK: usize = 1 << (usize::BITS - 1);
const ROOTS_MASK: usize = !MARK_MASK;
const ROOTS_MAX: usize = ROOTS_MASK; // max allowed value of roots

thread_local! {
    static GC_STATE: RefCell<GcState>  = RefCell::new(GcState {
        stats: GcStats::default(),
        config: GcConfig::default(),
        box_start: Cell::new(None),
    });
}

/// Collects garbage
fn collect_garbage(st: &mut GcState) {
    st.stats.collections_perfomed += 1;

    struct Unmarked<'a> {
        incoming: &'a Cell<Option<NonNull<GcBox<dyn Trace>>>>,
        // the current unmarked node
        this: NonNull<GcBox<dyn Trace>>,
    }

    unsafe fn mark(head: &Cell<Option<NonNull<GcBox<dyn Trace>>>>) -> Vec<Unmarked<'_>> {
        //walk the tree and mark all reachable nodes
        //It starts at the head of the list
        let mut mark_head = head.get();
        while let Some(node) = mark_head {
            if (*node.as_ptr()).header.roots() > 0 {
                (*node.as_ptr()).trace_inner();
            }

            // Then follows the `next` pointer until it reaches the end
            mark_head = (*node.as_ptr()).header.next.get();
        }

        // Collect a vector of all unmarked nodes, and unmark the ones which were
        let mut unmarked = Vec::new();
        let mut unmark_head = head;
        while let Some(node) = unmark_head.get() {
            if (*node.as_ptr()).header.is_marked() {
                // Unmark the node for the next collection
                (*node.as_ptr()).header.unmark();
            } else {
                // Collect the unmarked node
                unmarked.push(Unmarked {
                    // Incoming stills points to the start
                    incoming: unmark_head,
                    this: node,
                });
            }

            // Move to the raw pointer's next slot
            unmark_head = &(*node.as_ptr()).header.next;
        }

        unmarked
    }

    // Sweep the tree, dropping all unmarked nodes
    unsafe fn sweep(finalized: Vec<Unmarked<'_>>, bytes_allocated: &mut usize) {
        let _guard = DropGuard::new();
        for node in finalized.into_iter().rev() {
            if (*node.this.as_ptr()).header.is_marked() {
                // Don't claim the memory if it's still marked
                continue;
            }
            let incoming = node.incoming;
            // This is how sweep works:
            // Raw pointer is owned by Box after below call, and will be deallocated
            // the memory when `Box` goes out of scope
            let node = Box::from_raw(node.this.as_ptr());
            *bytes_allocated -= mem::size_of_val::<GcBox<_>>(&*node);
            // Take the value and lave `None` in its place
            incoming.set(node.header.next.take());
        }
    }

    unsafe {
        let unmarked = mark(&st.box_start);
        if unmarked.is_empty() {
            return;
        }
        for node in unmarked.iter() {
            Trace::finalize_glue(&(*node.this.as_ptr()).data);
        }
        mark(&st.box_start);
        sweep(unmarked, &mut st.stats.bytes_allocated);
    }
}

pub struct GcStats {
    /// The number of bytes allocated by the GC
    pub bytes_allocated: usize,
    /// Collections since the last time the stats were reset
    pub collections_perfomed: usize,
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

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            threshold: 100,
            used_space_ratio: 0.8,
            leak_on_drop: false,
        }
    }
}

pub struct GcBoxHeader {
    roots: Cell<usize>,
    next: Cell<Option<NonNull<GcBox<dyn Trace>>>>,
}

impl GcBoxHeader {
    #[inline]
    pub fn new(next: Option<NonNull<GcBox<dyn Trace>>>) -> Self {
        Self {
            roots: Cell::new(1),
            next: Cell::new(next),
        }
    }

    #[inline]
    pub fn roots(&self) -> usize {
        self.roots.get() & ROOTS_MASK
    }

    pub fn inc_roots(&self) {
        let roots = self.roots.get();

        // abort if the count overflows to prevent `mem::forget` loops
        // that could otherwise lead to erroneous drops
        if (roots & ROOTS_MASK) < ROOTS_MAX {
            self.roots.set(roots + 1); // we checked that this wont affect the high bit
        } else {
            panic!("roots counter overflow");
        }
    }

    #[inline]
    fn dec_roots(&self) {
        self.roots.set(self.roots.get() - 1)
    }

    #[inline]
    fn is_marked(&self) -> bool {
        self.roots.get() & MARK_MASK != 0
    }

    #[inline]
    fn mark(&self) {
        self.roots.set(self.roots.get() | MARK_MASK)
    }

    #[inline]
    fn unmark(&self) {
        self.roots.set(self.roots.get() & !MARK_MASK)
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

#[repr(C)]
pub struct GcBox<T: Trace + ?Sized + 'static> {
    header: GcBoxHeader,
    data: T,
}

impl<T: Trace> GcBox<T> {
    #[inline]
    pub fn new(data: T) -> NonNull<Self> {
        GC_STATE.with(|st| {
            let mut st = st.borrow_mut();

            if st.stats.bytes_allocated > st.config.threshold {
                collect_garbage(&mut st);

                if st.stats.bytes_allocated as f64
                    > st.config.threshold as f64 * st.config.used_space_ratio
                {
                    // We did not collect enough, so increase the threadhold for next time
                    st.config.threshold =
                        (st.stats.bytes_allocated as f64 / st.config.used_space_ratio) as usize;
                }
            }

            let gcbox = Box::into_raw(Box::new(GcBox {
                header: GcBoxHeader::new(st.box_start.take()),
                data,
            }));

            st.box_start
                .set(Some(unsafe { NonNull::new_unchecked(gcbox) }));

            // We allocated some bytes, let's record it
            st.stats.bytes_allocated += std::mem::size_of::<GcBox<T>>();

            // return the pointer to the newly allocated data
            unsafe { NonNull::new_unchecked(gcbox) }
        })
    }
}

impl<T: Trace + ?Sized> GcBox<T> {
    /// Returns `true` if the two references point to the same `GcBox`
    pub fn ptr_eq(this: &GcBox<T>, other: &GcBox<T>) -> bool {
        ptr::eq(&this.header, &other.header)
    }

    /// Marks this `GcBox` and marks through its data
    pub unsafe fn trace_inner(&self) {
        if !self.header.is_marked() {
            self.header.mark();
            self.data.trace();
        }
    }

    /// Increments the root count of this `GcBox`
    /// Roots prevent the `GcBox` from being destroyed by the GC
    pub unsafe fn root_inner(&self) {
        self.header.inc_roots();
    }

    /// Decrements the root count of this `GcBox`
    /// Roots prevent the `GcBox` from being destroyed by the GC
    pub unsafe fn unroot_inner(&self) {
        self.header.dec_roots();
    }

    /// Returns a pointer to the `GcBox`'s value without dereferencing it
    pub fn value_ptr(this: *const GcBox<T>) -> *const T {
        unsafe { ptr::addr_of!((*this).data) }
    }

    /// Returns a reference to the `GcBox`'s value
    pub fn value(&self) -> &T {
        &self.data
    }
}
