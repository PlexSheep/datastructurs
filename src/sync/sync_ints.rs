use std::ops::{Deref, DerefMut};

use crate::sync::SyncBox;

macro_rules! atomic_syncbox_int {
    ($name:ident, $primitive:ty) => {
        #[derive(Debug, Hash, Clone)]
        pub struct $name {
            inner: SyncBox<$primitive>,
        }

        impl $name {
            #[inline(always)]
            pub fn inc(&self) {
                unsafe {
                    (*self.inner.pointer()) += 1;
                }
            }
        }

        impl Deref for $name {
            type Target = SyncBox<$primitive>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
    };
}

atomic_syncbox_int!(SyncU128, u128);
atomic_syncbox_int!(SyncU64, u64);
atomic_syncbox_int!(SyncU32, u32);
atomic_syncbox_int!(SyncU16, u16);
atomic_syncbox_int!(SyncU8, u8);
atomic_syncbox_int!(SyncI128, i128);
atomic_syncbox_int!(SyncI64, i64);
atomic_syncbox_int!(SyncI32, i32);
atomic_syncbox_int!(SyncI16, i16);
atomic_syncbox_int!(SyncI8, i8);
