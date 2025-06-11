//! Stable references for intrusive data structures.
//!
//! This module provides [`StableRef`] and [`StableRefMut`], which are references that guarantee
//! the pointed-to value will never change its memory address. This is essential for intrusive
//! data structures that store raw pointers to elements they don't own.
//!
//! # Why Stable References?
//!
//! Intrusive data structures like [`IntrusiveList`](crate::intrusive_linked_list::IntrusiveList)
//! store pointers directly to user data. If that data moves in memory (e.g., when a `Vec`
//! reallocates), those pointers become dangling, leading to undefined behavior.
//!
//! # Safety Requirements
//!
//! This module provides three storage strategies:
//!
//! 1. **Owned Box** (`Boxed` variant): Safe. The `StableRef` owns the allocation.
//! 2. **Borrowed Box** (`Ref` variant): The caller must ensure the Box outlives the `StableRef`.
//! 3. **Raw Pointer** (`Raw` variant): Fully unsafe. The caller must ensure validity.
//!
//! # Examples
//!
//! ## Safe: Owned Box
//! ```
//! use datastructurs::stable_ref::StableRef;
//!
//! let stable = StableRef::from_box(Box::new(42));
//! // Safe: StableRef owns the Box
//! assert_eq!(*stable.as_ref(), 42);
//! ```
//!
//! ## Semi-Safe: Borrowed Box
//!
//! ```compile_fail
//! use datastructurs::stable_ref::StableRefMut;
//!
//! let mut boxed = Box::new(42);
//! let stable = StableRefMut::from_boxref(&mut boxed);
//! drop(boxed); // ERROR: can't drop the box while it is borrowed by stable
//! assert_eq!(*stable.as_ref(), 42);
//! ```
//!
//! This works:
//!
//! ```
//! use datastructurs::stable_ref::StableRefMut;
//!
//! let mut boxed = Box::new(42);
//! let stable = StableRefMut::from_boxref(&mut boxed);
//! assert_eq!(*stable.as_ref(), 42);
//! ```
//!
//! ## Unsafe: Raw Pointer
//!
//! You can turn a raw pointer into a [StableRef] or [StableRefMut], but you have to guarantee that
//! the pointer remains valid.
//!
//! ```
//! use datastructurs::stable_ref::StableRef;
//! use std::ptr::NonNull;
//!
//! let value = Box::leak(Box::new(42));
//! let stable = unsafe {
//!     StableRef::from_raw(NonNull::from(value))
//! };
//! ```
//!
//! # Common Pitfalls
//!
//! These compile but result in undefined behavior.
//!
//! ## Stack Values
//! ```
//! use std::ptr::NonNull;
//! use datastructurs::stable_ref::StableRef;
//! let stable;
//! {
//! let value = 42;
//! stable = unsafe { StableRef::from_raw(NonNull::from(&value)) };
//! // WRONG: value will be out of scope and deallocated!
//! }
//! assert_eq!(*stable.as_ref(), 42);
//! ```
//!
//! ## Vec Elements  
//! ```
//! use std::ptr::NonNull;
//! use datastructurs::stable_ref::StableRef;
//! let mut vec = vec![1, 2, 3];
//! let stable = unsafe { StableRef::from_raw(NonNull::from(&vec[0])) };
//! vec.reserve(40); // Reallocation! stable is now dangling!
//! assert_eq!(*stable.as_ref(), 1);
//! ```

// I think this is what I actually need to make sure the thing never
// moves in the memory
#![allow(clippy::borrowed_box)]

use std::ptr::NonNull;

/// A reference that guarantees the pointed-to value has a stable memory address.
///
/// This type is used by intrusive data structures to ensure pointers remain valid.
///
/// # Variants
///
/// - `Boxed(Box<T>)`: Owns a heap allocation. Always safe.
/// - `Ref(&'a Box<T>)`: Borrows a Box. Box can not be dropped while the [StableRef] exists
/// - `Raw(*const T)`: Raw pointer. User must ensure validity.
///
/// # Safety
///
/// For the `Raw` variant, see [`from_raw`](Self::from_raw).
#[derive(Debug)]
pub enum StableRef<'a, T: 'a> {
    /// Owned Box that guarantees a stable heap address.
    Boxed(Box<T>),
    /// Borrowed reference to a Box. The Box can not be dropped while this reference exists
    BoxRef(&'a Box<T>),
    /// Raw pointer with manual lifetime management.
    Raw(NonNull<T>),
}

/// A mutable reference that guarantees the pointed-to value has a stable memory address.
///
/// This is the mutable counterpart to [`StableRef`].
///
/// # Safety
///
/// The same safety requirements as [`StableRef`] apply, with additional
/// caution for mutable access.
#[derive(Debug)]
pub enum StableRefMut<'a, T: 'a> {
    /// Owned Box that guarantees a stable heap address.
    Boxed(Box<T>),
    /// Borrowed mutable reference to a Box. The Box must outlive this reference.
    BoxRef(&'a mut Box<T>),
    /// Raw mutable pointer with manual lifetime management.
    Raw(NonNull<T>),
}

impl<'a, T> StableRef<'a, T> {
    /// Creates a [`StableRef`] from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - The pointer is valid for reads for the lifetime `'a`
    /// - The pointed-to value will not move in memory
    /// - The pointed-to value will not be deallocated while this `StableRef` exists
    /// - The pointer is properly aligned and points to a valid `T`
    ///
    /// # Example
    /// ```
    /// use datastructurs::stable_ref::StableRef;
    /// use std::ptr::NonNull;
    ///
    /// // Safe: leaked memory never moves or gets deallocated
    /// let leaked = Box::leak(Box::new(42));
    /// let stable = unsafe { StableRef::from_raw(NonNull::from(leaked)) };
    /// assert_eq!(*stable.as_ref(), 42);
    /// // After being done, the user must manually deallocate the value, otherwise, some memory
    /// // leaks.
    /// ```
    #[inline]
    pub unsafe fn from_raw(ptr: NonNull<T>) -> Self {
        Self::Raw(ptr)
    }

    /// Creates a [`StableRef`] that owns a [Box<T>].
    ///
    /// This is always safe as the [Box] heap allocation has a stable address.
    ///
    /// # Example
    /// ```
    /// use datastructurs::stable_ref::StableRef;
    ///
    /// let stable = StableRef::from_box(Box::new(42));
    /// assert_eq!(*stable.as_ref(), 42);
    /// // The value 42 is now owned by stable
    /// ```
    #[inline]
    pub fn from_box(bx: Box<T>) -> Self {
        Self::Boxed(bx)
    }

    /// [Box] a value and create a [StableRef] with [StableRef::from_box]
    #[inline]
    pub fn create_box(bx: impl Into<Box<T>>) -> Self {
        Self::from_box(bx.into())
    }

    /// Creates a [`StableRef`] that borrows a [Box].
    ///
    /// # Safety
    ///
    /// The borrowed [Box] must outlive this [`StableRef`]. The [Box] itself must not
    /// be moved (e.g., by moving the variable holding it).
    ///
    /// # Example
    ///
    /// ```
    /// use datastructurs::stable_ref::StableRef;
    ///
    /// let boxed = Box::new(42);
    /// let stable = StableRef::from_boxref(&boxed);
    /// // boxed cannot be dropped while stable exists
    /// assert_eq!(*stable.as_ref(), 42);
    /// ```
    ///
    /// ```compile_fail
    /// use datastructurs::stable_ref::StableRef;
    ///
    /// let boxed = Box::new(42);
    /// let stable = StableRef::from_boxref(&boxed);
    /// drop(boxed); // ERROR: boxed cannot be dropped while it is referenced
    /// assert_eq!(*stable.as_ref(), 42);
    /// ```
    #[inline]
    pub fn from_boxref(r: &'a Box<T>) -> Self {
        Self::BoxRef(r)
    }

    #[inline]
    pub unsafe fn from_ref_to_raw(r: &'a mut T) -> Self {
        unsafe { Self::from_raw(NonNull::new_unchecked(r as *mut T)) }
    }

    /// Returns a raw pointer to the referenced value.
    ///
    /// The pointer is valid for the lifetime of this [`StableRef`].
    #[inline]
    pub fn as_ptr(&self) -> NonNull<T> {
        match self {
            Self::Raw(r) => *r,
            Self::Boxed(r) => box_to_raw(r),
            Self::BoxRef(r) => ref_to_raw(r),
        }
    }

    /// Convert the [StableRef] into a [StableRefMut]
    ///
    /// # Safety
    ///
    /// If [self] is a [`BoxRef`], then this casts a `&T` into `&mut T`.
    #[inline]
    pub unsafe fn into_stable_mut(self) -> StableRefMut<'a, T> {
        match self {
            StableRef::BoxRef(r) => StableRefMut::BoxRef(unsafe { ref_to_mut(r) }),
            StableRef::Raw(r) => StableRefMut::Raw(r),
            StableRef::Boxed(r) => StableRefMut::Boxed(r),
        }
    }
}

impl<'a, T> StableRefMut<'a, T> {
    /// Creates a [`StableRefMut`] from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - The pointer is valid for reads for the lifetime `'a`
    /// - The pointed-to value will not move in memory
    /// - The pointed-to value will not be deallocated while this `StableRef` exists
    /// - The pointer is properly aligned and points to a valid `T`
    ///
    /// In addition to these requirements of [`StableRef::from_raw`], the caller
    /// must ensure no other references (mutable or immutable) to the same data
    /// exist while this `StableRefMut` is alive, or that no race conditions can
    /// occur if there are other references.
    ///
    /// # Example
    /// ```
    /// use datastructurs::stable_ref::StableRefMut;
    /// use std::ptr::NonNull;
    ///
    /// // Safe: leaked memory never moves or gets deallocated
    /// let leaked = Box::leak(Box::new(42));
    /// let stable = unsafe { StableRefMut::from_raw(NonNull::from(leaked)) };
    /// assert_eq!(*stable.as_ref(), 42);
    /// // After being done, the user must manually deallocate the value, otherwise, some memory
    /// // leaks.
    /// ```
    #[inline]
    pub unsafe fn from_raw(ptr: NonNull<T>) -> Self {
        Self::Raw(ptr)
    }

    /// Creates a [`StableRefMut`] that owns a [Box<T>].
    ///
    /// This is always safe as the [Box] heap allocation has a stable address.
    ///
    /// # Example
    /// ```
    /// use datastructurs::stable_ref::StableRefMut;
    ///
    /// let stable = StableRefMut::from_box(Box::new(42));
    /// assert_eq!(*stable.as_ref(), 42);
    /// // The value 42 is now owned by stable
    /// ```
    #[inline]
    pub fn from_box(bx: Box<T>) -> Self {
        Self::Boxed(bx)
    }

    /// [Box] a value and create a [StableRefMut] with [StableRefMut::from_box]
    #[inline]
    pub fn create_box(bx: impl Into<Box<T>>) -> Self {
        Self::from_box(bx.into())
    }

    /// Creates a [`StableRefMut`] that borrows a [Box].
    ///
    /// # Safety
    ///
    /// The borrowed [Box] must outlive this [`StableRefMut`]. The [Box] itself must not
    /// be moved (e.g., by moving the variable holding it). No other references to the box or its
    /// content may exist while the borrow is active
    ///
    /// # Example
    ///
    /// ```
    /// use datastructurs::stable_ref::StableRefMut;
    ///
    /// let mut boxed = Box::new(42);
    /// let stable = StableRefMut::from_boxref(&mut boxed);
    /// // boxed cannot be dropped while stable exists
    /// assert_eq!(*stable.as_ref(), 42);
    /// ```
    ///
    /// ```compile_fail
    /// use datastructurs::stable_ref::StableRefMut;
    ///
    /// let mut boxed = Box::new(42);
    /// let stable = StableRefMut::from_boxref(&mut boxed);
    /// drop(boxed); // ERROR: boxed cannot be dropped while it is referenced
    /// assert_eq!(*stable.as_ref(), 42);
    /// ```
    #[inline]
    pub fn from_boxref(r: &'a mut Box<T>) -> Self {
        Self::BoxRef(r)
    }

    #[inline]
    pub unsafe fn from_ref_to_raw(r: &'a mut T) -> Self {
        unsafe { Self::from_raw(NonNull::new_unchecked(r as *mut T)) }
    }

    /// Returns a raw pointer to the referenced value.
    ///
    /// The pointer is valid for the lifetime of this [`StableRefMut`].
    #[inline]
    pub fn as_ptr(&self) -> NonNull<T> {
        match self {
            Self::Raw(r) => *r,
            Self::Boxed(r) => box_to_raw(r),
            Self::BoxRef(r) => ref_to_raw(r),
        }
    }

    /// Convert a reference to a [StableRefMut] into a [StableRef]
    ///
    /// # Safety
    ///
    /// [`StableRefMut::Boxed`] becomes a [`StableRef::BoxRef`],
    /// but this is a safe operation.
    #[inline]
    pub fn as_stable_ref(&'a self) -> StableRef<'a, T> {
        match self {
            Self::Boxed(b) => StableRef::BoxRef(b),
            Self::BoxRef(b) => StableRef::BoxRef(b),
            Self::Raw(r) => StableRef::Raw(*r),
        }
    }

    /// Convert the [StableRefMut] into a [StableRef]
    ///
    /// # Safety
    ///
    /// If [self] is a [`BoxRef`], then this casts a `&mut T` into `&T`,
    /// but this is a safe operation.
    #[inline]
    pub fn into_stable_ref(self) -> StableRef<'a, T> {
        match self {
            Self::Boxed(b) => StableRef::Boxed(b),
            Self::BoxRef(r) => StableRef::BoxRef(r),
            Self::Raw(r) => StableRef::Raw(r),
        }
    }

    /// Clone the [StableRefMut], creating a **second mutable reference**
    ///
    /// # Safety
    ///
    /// The user must take measures to ensure that no race condition arises
    /// from multiple mutable references existing to the same value.
    #[inline]
    #[allow(elided_named_lifetimes)]
    pub unsafe fn clone(&'a self) -> StableRefMut<'_, T> {
        match self {
            Self::Boxed(b) => Self::BoxRef(unsafe { ref_to_mut(b) }),
            Self::BoxRef(r) => Self::BoxRef(unsafe { ref_to_mut(*r) }),
            Self::Raw(r) => Self::Raw(*r),
        }
    }
}

impl<'a, T> AsRef<T> for StableRef<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Boxed(bx) => bx,
            Self::BoxRef(r) => r,
            Self::Raw(ptr) => unsafe { ptr.as_ref() },
        }
    }
}

impl<'a, T> AsRef<T> for StableRefMut<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Boxed(bx) => bx,
            Self::BoxRef(r) => r,
            Self::Raw(ptr) => unsafe { ptr.as_ref() },
        }
    }
}

impl<'a, T> AsMut<T> for StableRefMut<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        match self {
            Self::Boxed(bx) => bx,
            Self::BoxRef(r) => r,
            Self::Raw(ptr) => unsafe { ptr.as_mut() },
        }
    }
}

impl<T> From<Box<T>> for StableRef<'_, T> {
    fn from(value: Box<T>) -> Self {
        Self::Boxed(value)
    }
}

impl<'a, T> From<&'a Box<T>> for StableRef<'a, T> {
    fn from(value: &'a Box<T>) -> Self {
        Self::BoxRef(value)
    }
}

impl<T> From<Box<T>> for StableRefMut<'_, T> {
    fn from(value: Box<T>) -> Self {
        Self::Boxed(value)
    }
}

impl<'a, T> From<&'a mut Box<T>> for StableRefMut<'a, T> {
    fn from(value: &'a mut Box<T>) -> Self {
        Self::BoxRef(value)
    }
}

impl<'a, T> From<StableRefMut<'a, T>> for StableRef<'a, T> {
    fn from(value: StableRefMut<'a, T>) -> Self {
        match value {
            StableRefMut::BoxRef(r) => StableRef::BoxRef(r),
            StableRefMut::Raw(r) => StableRef::Raw(r),
            StableRefMut::Boxed(r) => StableRef::Boxed(r),
        }
    }
}

impl<'a, T: Clone> Clone for StableRef<'a, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Boxed(b) => Self::Boxed(b.clone()),
            Self::BoxRef(r) => Self::BoxRef(r),
            Self::Raw(r) => Self::Raw(*r),
        }
    }
}

#[inline]
fn box_to_raw<T>(b: &Box<T>) -> NonNull<T> {
    ref_to_raw(b.as_ref())
}

#[inline]
pub(crate) fn ref_to_raw<T>(b: &T) -> NonNull<T> {
    let a: *const T = b;
    NonNull::new(a as *mut T).expect("pointer was null!")
}

#[allow(unused)]
#[allow(clippy::mut_from_ref)]
pub(crate) unsafe fn ref_to_mut<T>(t: &T) -> &mut T {
    let p = t as *const T as *mut T;
    unsafe { p.as_mut().expect("pointer was null") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Clone, Debug, PartialEq)]
    #[allow(unused)]
    struct Thing {
        a: i32,
        b: String,
    }

    impl Thing {
        fn new(a: i32) -> Self {
            Self {
                a,
                b: format!("Thing-{a}"),
            }
        }
    }

    fn inspect_thing(sref: StableRef<Thing>) {
        let thing = sref.as_ref();
        dbg!(thing);
    }

    fn change_thing(sref: &mut StableRefMut<Thing>) {
        let thing = sref.as_mut();
        thing.a *= 100;
        thing.b.push_str("-changed");
    }

    #[test]
    fn test_stable_ref_imm() {
        let box_thing = Box::new(Thing::new(0));
        let raw_thing = Box::into_raw(Box::new(Thing::new(1)));
        let ref_thing = Box::new(Thing::new(2));

        let rbox: StableRef<'_, Thing> = StableRef::from_box(box_thing);
        let rraw: StableRef<'_, Thing> =
            unsafe { StableRef::from_raw(NonNull::new(raw_thing).unwrap()) };
        let rref: StableRef<'_, Thing> = StableRef::from_boxref(&ref_thing);

        inspect_thing(rbox);
        inspect_thing(rraw);
        inspect_thing(rref);
    }

    #[test]
    fn test_stable_ref_mut() {
        let box_thing = Box::new(Thing::new(0));
        let raw_thing = Box::into_raw(Box::new(Thing::new(1)));
        let mut ref_thing = Box::new(Thing::new(2));

        let mut rbox: StableRefMut<'_, Thing> = StableRefMut::from_box(box_thing);
        let mut rraw: StableRefMut<'_, Thing> =
            unsafe { StableRefMut::from_raw(NonNull::new(raw_thing).unwrap()) };
        let mut rref: StableRefMut<'_, Thing> = StableRefMut::from_boxref(&mut ref_thing);

        inspect_thing(rbox.as_stable_ref());
        inspect_thing(rraw.as_stable_ref());
        inspect_thing(rref.as_stable_ref());

        change_thing(&mut rbox);
        change_thing(&mut rraw);
        change_thing(&mut rref);

        inspect_thing(rbox.as_stable_ref());
        inspect_thing(rraw.as_stable_ref());
        inspect_thing(rref.as_stable_ref());
    }

    #[test]
    fn test_stable_ref_drop_box() {
        let box_thing = Box::new(Thing::new(0));
        let rbox = StableRef::from_box(box_thing);
        inspect_thing(rbox);
    }

    // NOTE: This test does not panic but abort, the stable ref to value is constructed to be
    // invalid after the function finishes.
    #[test]
    #[ignore = "should abort instead of panic"]
    fn test_stable_ref_abort() {
        use std::ptr::NonNull;
        let mut stable: StableRef<Thing> = StableRef::create_box(Thing::new(0));
        fn foo(s: &mut StableRef<Thing>) {
            let value = Thing::new(42);
            *s = unsafe { StableRef::from_raw(NonNull::from(&value)) };
            // WRONG: value will be out of scope and deallocated!
        }
        foo(&mut stable);
        assert_eq!(*stable.as_ref(), Thing::new(42));
    }
}
