use std::marker::PhantomData;

/// Allocates a value in a Box and makes it available with a raw pointer dereference across threads
///
/// # Safety
///
/// This is probably not really thread safe, but good enough for incrementing numbers
/// quickly.
#[derive(Debug)]
pub struct SyncBox<T: Sized + Send + Sync> {
    inner: *mut SyncBoxInner<T>,
    pub(crate) dtype: PhantomData<T>,
}

#[derive(Debug)]
struct SyncBoxInner<T: Sized + Send + Sync> {
    value: T,
    refs: u32,
}

impl<T: Sized + Send + Sync> SyncBox<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        let inner = SyncBoxInner { value, refs: 1 };
        let inner_box = Box::new(inner);
        Self {
            dtype: PhantomData,
            inner: Box::leak(inner_box),
        }
    }

    #[inline(always)]
    pub fn get(&self) -> &T {
        unsafe { &*self.pointer() }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.pointer() }
    }

    #[inline(always)]
    pub fn set(&self, new: T) {
        unsafe {
            (*self.pointer()) = new;
        }
    }

    #[inline(always)]
    pub fn pointer(&self) -> *mut T {
        self.inner as *mut T
    }
}

impl<T: Copy + Sized + Send + Sync> SyncBox<T> {
    #[inline(always)]
    pub fn val(&self) -> T {
        unsafe { *self.pointer() }
    }
}

impl<T: Sized + Send + Sync + Default> Default for SyncBox<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Sized + Send + Sync> Clone for SyncBox<T> {
    fn clone(&self) -> Self {
        unsafe { (*self.inner).refs += 1 };
        Self {
            dtype: self.dtype,
            inner: self.inner,
        }
    }
}

impl<T: Sized + Send + Sync> Drop for SyncBox<T> {
    fn drop(&mut self) {
        unsafe { (*self.inner).refs = (*self.inner).refs.saturating_sub(1) };
        if unsafe { (*self.inner).refs < 1 } {
            let buf: Box<_> = unsafe { Box::from_raw(self.inner) };
            drop(buf)
        }
    }
}

impl<T: Sized + Send + Sync> From<T> for SyncBox<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

unsafe impl<T: Sized + Send + Sync> Send for SyncBox<T> {}
unsafe impl<T: Sized + Send + Sync> Sync for SyncBox<T> {}
