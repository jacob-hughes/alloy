#[repr(C)]
#[derive(Default)]
pub struct ProfileStats {
    /// Heap size in bytes (including area unmapped to OS).
    pub(crate) heapsize_full: usize,
    /// Total bytes contained in free and unmapped blocks.
    pub(crate) free_bytes_full: usize,
    /// Amount of memory unmapped to OS.
    pub(crate) unmapped_bytes: usize,
    /// Number of bytes allocated since the recent collection.
    pub(crate) bytes_allocd_since_gc: usize,
    /// Number of bytes allocated before the recent collection.
    /// The value may wrap.
    pub(crate) allocd_bytes_before_gc: usize,
    /// Number of bytes not considered candidates for garbage collection.
    pub(crate) non_gc_bytes: usize,
    /// Garbage collection cycle number.
    /// The value may wrap.
    pub(crate) gc_no: usize,
    /// Number of marker threads (excluding the initiating one).
    pub(crate) markers_m1: usize,
    /// Approximate number of reclaimed bytes after recent collection.
    pub(crate) bytes_reclaimed_since_gc: usize,
    /// Approximate number of bytes reclaimed before the recent collection.
    /// The value may wrap.
    pub(crate) reclaimed_bytes_before_gc: usize,
    /// Number of bytes freed explicitly since the recent GC.
    pub(crate) expl_freed_bytes_since_gc: usize,
}

#[link(name = "gc")]
extern "C" {
    pub(crate) fn GC_malloc(nbytes: usize) -> *mut u8;

    #[cfg(not(feature = "rustgc"))]
    pub(crate) fn GC_malloc_uncollectable(nbytes: usize) -> *mut u8;

    pub(crate) fn GC_realloc(old: *mut u8, new_size: usize) -> *mut u8;

    pub(crate) fn GC_free(dead: *mut u8);

    pub(crate) fn GC_register_finalizer(
        ptr: *mut u8,
        finalizer: Option<unsafe extern "C" fn(*mut u8, *mut u8)>,
        client_data: *mut u8,
        old_finalizer: *mut extern "C" fn(*mut u8, *mut u8),
        old_client_data: *mut *mut u8,
    );

    pub(crate) fn GC_register_finalizer_no_order(
        ptr: *mut u8,
        finalizer: Option<unsafe extern "C" fn(*mut u8, *mut u8)>,
        client_data: *mut u8,
        old_finalizer: *mut extern "C" fn(*mut u8, *mut u8),
        old_client_data: *mut *mut u8,
    );

    pub(crate) fn GC_gcollect();

    pub(crate) fn GC_start_performance_measurement();

    pub(crate) fn GC_get_full_gc_total_time() -> usize;

    pub(crate) fn GC_get_prof_stats(prof_stats: *mut ProfileStats, stats_size: usize) -> usize;

    #[cfg(feature = "rustgc")]
    pub(crate) fn GC_malloc_explicitly_typed(size: usize, descriptor: usize) -> *mut u8;

    #[cfg(feature = "rustgc")]
    pub(crate) fn GC_make_descriptor(bitmap: *const usize, len: usize) -> usize;

    #[cfg(feature = "rustgc")]
    pub(crate) fn GC_malloc_atomic(nbytes: usize) -> *mut u8;
}
