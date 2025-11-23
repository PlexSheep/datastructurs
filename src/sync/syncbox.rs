use std::marker::PhantomData;

/// Allocates a value in a Box and makes it available with a raw pointer dereference across threads
///
/// # Safety
///
/// This is probably not really thread safe, but good enough for incrementing numbers
/// quickly.
#[derive(Debug, Hash, Clone)]
pub struct SyncBox<T: Sized + Send + Sync> {
    pub(crate) pointer: *mut T,
    pub(crate) dtype: PhantomData<T>,
}

impl<T: Sized + Send + Sync> SyncBox<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        let buf: Box<T> = Box::new(value);
        Self {
            dtype: PhantomData,
            pointer: Box::leak(buf),
        }
    }

    #[inline(always)]
    pub fn get(&self) -> &T {
        unsafe { &*self.pointer }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.pointer }
    }

    #[inline(always)]
    pub fn set(&self, new: T) {
        unsafe {
            (*self.pointer) = new;
        }
    }

    #[inline(always)]
    pub fn pointer(&self) -> *mut T {
        self.pointer
    }
}

impl<T: Sized + Send + Sync + Default> Default for SyncBox<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Sized + Send + Sync> Drop for SyncBox<T> {
    fn drop(&mut self) {
        let buf: Box<T> = unsafe { Box::from_raw(self.pointer) };
        drop(buf)
    }
}

unsafe impl<T: Sized + Send + Sync> Send for SyncBox<T> {}
unsafe impl<T: Sized + Send + Sync> Sync for SyncBox<T> {}
