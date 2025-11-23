use std::thread::JoinHandle;

use datastructurs::sync::sync_ints::{SyncU64, SyncUsize};

#[test]
fn test_sync_int_st() {
    let idx = SyncU64::new(1);
    idx.inc();
    assert_eq!(*idx.get(), 2);
    idx.inc();
    assert_eq!(*idx.get(), 3);
    *idx.get_mut() = 1337;
    assert_eq!(*idx.get(), 1337);
    idx.set(19);
    assert_eq!(*idx.get(), 19);
    assert_eq!(idx.val(), 19);
}

#[test]
fn test_sync_int_mt() {
    let idx = SyncUsize::new(1);
    idx.inc();
    assert_eq!(*idx.get(), 2);
    idx.set(0);

    const THREADS: usize = 4;
    let iters: usize = 200;
    let mut ths = Vec::new();
    for i in 0..THREADS {
        let idx_ref = idx.clone();
        ths.push(std::thread::spawn(move || {
            for _ in 0..iters {
                idx_ref.inc();
            }
        }));
    }

    for th in ths {
        th.join();
    }

    assert_eq!(*idx.get(), THREADS * iters);
}
