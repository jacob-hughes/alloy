#![allow(missing_docs)]
use core::alloc::Layout;
use core::any::Any;
use core::fmt;
use core::gc::ManageableContents;
use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

use boehm_shim;

use crate::alloc::AllocRef;
use crate::boehm::BoehmGcAllocator;
use crate::vec::Vec;

/// A garbage collected pointer.
///
/// The type `Gc<T>` provides shared ownership of a value of type `T`,
/// allocted in the heap. `Gc` pointers are `Copyable`, so new pointers to
/// the same value in the heap can be produced trivially. The lifetime of
/// `T` is tracked automatically: it is freed when the application
/// determines that no references to `T` are in scope. This does not happen
/// deterministically, and no guarantees are given about when a value
/// managed by `Gc` is freed.
///
/// Shared references in Rust disallow mutation by default, and `Gc` is no
/// exception: you cannot generally obtain a mutable reference to something
/// inside an `Gc`. If you need mutability, put a `Cell` or `RefCell` inside
/// the `Gc`.
///
/// Unlike `Rc<T>`, cycles between `Gc` pointers are allowed and can be
/// deallocated without issue.
///
/// `Gc<T>` automatically dereferences to `T` (via the `Deref` trait), so
/// you can call `T`'s methods on a value of type `Gc<T>`.
#[unstable(feature = "gc", reason = "gc", issue = "none")]
pub struct Gc<T: ?Sized> {
    ptr: NonNull<GcBox<T>>,
    _phantom: PhantomData<T>,
}

impl<T> Gc<T> {
    /// Constructs a new `Gc<T>`.
    #[unstable(feature = "gc", reason = "gc", issue = "none")]
    pub fn new(v: T) -> Self {
        Gc { ptr: unsafe { NonNull::new_unchecked(GcBox::new(v)) }, _phantom: PhantomData }
    }

    /// Constructs a new `Gc<MaybeUninit<T>>` which is capable of storing data
    /// up-to the size permissible by `layout`.
    ///
    /// This can be useful if you want to store a value with a custom layout,
    /// but have the collector treat the value as if it were T.
    ///
    /// # Panics
    ///
    /// If `layout` is smaller than that required by `T` and/or has an alignment
    /// which is smaller than that required by `T`.
    pub fn new_from_layout(layout: Layout) -> Gc<MaybeUninit<T>> {
        let tl = Layout::new::<T>();
        if layout.size() < tl.size() || layout.align() < tl.align() {
            panic!(
                "Requested layout {:?} is either smaller than size {} and/or not aligned to {}",
                layout,
                tl.size(),
                tl.align()
            );
        }
        unsafe { Gc::new_from_layout_unchecked(layout) }
    }

    /// Constructs a new `Gc<MaybeUninit<T>>` which is capable of storing data
    /// up-to the size permissible by `layout`.
    ///
    /// This can be useful if you want to store a value with a custom layout,
    /// but have the collector treat the value as if it were T.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that both `layout`'s size and
    /// alignment must match or exceed that required to store `T`.
    pub unsafe fn new_from_layout_unchecked(layout: Layout) -> Gc<MaybeUninit<T>> {
        Gc::from_inner(GcBox::new_from_layout(layout))
    }
}

impl Gc<dyn Any> {
    #[unstable(feature = "gc", reason = "gc", issue = "none")]
    pub fn downcast<T: Any>(&self) -> Result<Gc<T>, Gc<dyn Any>> {
        if (*self).is::<T>() {
            let ptr = self.ptr.cast::<GcBox<T>>();
            Ok(Gc::from_inner(ptr))
        } else {
            Err(Gc::from_inner(self.ptr))
        }
    }
}

impl<T: ?Sized> Gc<T> {
    /// Get a raw pointer to the underlying value `T`.
    #[unstable(feature = "gc", reason = "gc", issue = "none")]
    pub fn into_raw(this: Self) -> *const T {
        this.ptr.as_ptr() as *const T
    }

    #[unstable(feature = "gc", reason = "gc", issue = "none")]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr.as_ptr() == other.ptr.as_ptr()
    }

    #[unstable(feature = "gc", reason = "gc", issue = "none")]
    pub fn from_raw(raw: *const T) -> Gc<T> {
        Gc { ptr: unsafe { NonNull::new_unchecked(raw as *mut GcBox<T>) }, _phantom: PhantomData }
    }

    fn from_inner(ptr: NonNull<GcBox<T>>) -> Self {
        Self { ptr, _phantom: PhantomData }
    }
}

impl<T> Gc<MaybeUninit<T>> {
    /// As with `MaybeUninit::assume_init`, it is up to the caller to guarantee
    /// that the inner value really is in an initialized state. Calling this
    /// when the content is not yet fully initialized causes immediate undefined
    /// behaviour.
    #[unstable(feature = "gc", reason = "gc", issue = "none")]
    pub unsafe fn assume_init(self) -> Gc<T> {
        let ptr = self.ptr.as_ptr() as *mut GcBox<MaybeUninit<T>>;
        unsafe { Gc::from_inner((&mut *ptr).assume_init()) }
    }
}

/// A `GcBox` is a 0-cost wrapper which allows a single `Drop` implementation
/// while also permitting multiple, copyable `Gc` references. The `drop` method
/// on `GcBox` acts as a guard, preventing the destructors on its contents from
/// running unless the object is really dead.
#[unstable(feature = "gc", reason = "gc", issue = "none")]
struct GcBox<T: ?Sized>(ManuallyDrop<T>);

impl<T> GcBox<T> {
    fn new(value: T) -> *mut GcBox<T> {
        let layout = Layout::new::<T>();
        let ptr = BoehmGcAllocator.alloc(layout).unwrap().as_ptr() as *mut GcBox<T>;
        let gcbox = GcBox(ManuallyDrop::new(value));

        unsafe {
            ptr.copy_from_nonoverlapping(&gcbox, 1);
            if !Self::is_manageable_contents() && Self::needs_drop() {
                GcBox::register_finalizer(&mut *ptr);
            }
        }

        mem::forget(gcbox);
        ptr
    }

    fn new_from_layout(layout: Layout) -> NonNull<GcBox<MaybeUninit<T>>> {
        unsafe {
            let base_ptr = BoehmGcAllocator.alloc(layout).unwrap().as_ptr() as *mut usize;
            NonNull::new_unchecked((base_ptr.add(1)) as *mut GcBox<MaybeUninit<T>>)
        }
    }

    fn register_finalizer(&mut self) {
        unsafe extern "C" fn fshim<T>(obj: *mut u8, _meta: *mut u8) {
            unsafe {
                ManuallyDrop::drop(&mut *(obj as *mut ManuallyDrop<T>));
            }
        }

        unsafe {
            boehm_shim::gc_register_finalizer(
                self as *mut _ as *mut u8,
                fshim::<T>,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
        }
    }
}

trait IsManageableContents {
    fn is_manageable_contents() -> bool;
}

impl<T> IsManageableContents for GcBox<T> {
    default fn is_manageable_contents() -> bool {
        false
    }
}

impl<T: ManageableContents> IsManageableContents for GcBox<Vec<T>> {
    fn is_manageable_contents() -> bool {
        true
    }
}

trait NeedsDrop {
    fn needs_drop() -> bool;
}

impl<T> NeedsDrop for GcBox<T> {
    default fn needs_drop() -> bool {
        mem::needs_drop::<T>()
    }
}

impl<T> NeedsDrop for GcBox<Vec<T>> {
    fn needs_drop() -> bool {
        mem::needs_drop::<T>()
    }
}

impl<T> GcBox<MaybeUninit<T>> {
    unsafe fn assume_init(&mut self) -> NonNull<GcBox<T>> {
        // With T now considered initialized, we must make sure that if GcBox<T>
        // is reclaimed, T will be dropped. We need to find its vptr and replace the
        // GcDummyDrop vptr in the block header with it.
        self.register_finalizer();
        unsafe { NonNull::new_unchecked(self as *mut _ as *mut GcBox<T>) }
    }
}

#[unstable(feature = "gc", reason = "gc", issue = "none")]
impl<T: ?Sized> Copy for Gc<T> {}

#[unstable(feature = "gc", reason = "gc", issue = "none")]
impl<T: ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[unstable(feature = "gc", reason = "gc", issue = "none")]
impl<T: ?Sized> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr.as_ptr() as *const T) }
    }
}

#[unstable(feature = "gc", reason = "gc", issue = "none")]
impl<T: ?Sized> DerefMut for Gc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.ptr.as_ptr() as *mut T) }
    }
}

#[unstable(feature = "gc", reason = "gc", issue = "none")]
impl<T: ?Sized + fmt::Debug> fmt::Debug for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

#[unstable(feature = "gc", reason = "gc", issue = "none")]
impl<T: ?Sized + fmt::Display> fmt::Display for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
