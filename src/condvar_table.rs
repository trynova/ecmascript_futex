use std::{
    hint::spin_loop,
    sync::{Condvar, Mutex, MutexGuard},
    time::Duration,
};

use crate::FutexError;

/// The number of OS synchronization primitives to use.
const TABLE_SIZE: usize = 256;

/// The table of OS synchronization primitives.
static TABLE: [TableEntry; TABLE_SIZE] = [TableEntry::DEFAULT; TABLE_SIZE];

/// Puts the current thread to sleep if `condition` evaluates to `true`.
/// The thread will be woken after `timeout` if it is provided.
pub fn wait(
    ptr: *const (),
    condition: impl Fn() -> bool,
    timeout: Option<Duration>,
) -> Result<(), FutexError> {
    let entry = &TABLE[entry_for_ptr(ptr) as usize];
    let mut guard = spin_lock(&entry.mutex);
    let mut timedout = false;
    if condition() {
        if guard.waiting_count == 0 {
            guard.address = ptr;
        } else if guard.address != ptr {
            guard.address = std::ptr::null();
        }

        guard.waiting_count += 1;

        guard = if let Some(time) = timeout {
            let (guard, result) = entry
                .condvar
                .wait_timeout(guard, time)
                .expect("Failed to lock mutex");
            timedout = result.timed_out();
            guard
        } else {
            entry.condvar.wait(guard).expect("Failed to lock mutex")
        };

        guard.waiting_count -= 1;

        if timedout {
            Err(FutexError::Timeout)
        } else {
            Ok(())
        }
    } else {
        Err(FutexError::NotEqual)
    }
}

/// Wakes all threads waiting on `ptr`.
pub fn notify_all(ptr: *const ()) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let entry = &TABLE[entry_for_ptr(ptr) as usize];
    let metadata = *spin_lock(&entry.mutex);
    if 0 < metadata.waiting_count {
        entry.condvar.notify_all();
    }
    metadata.waiting_count
}

/// Wakes at least one thread waiting on `ptr`.
pub fn notify_many(ptr: *const (), count: usize) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let entry = &TABLE[entry_for_ptr(ptr) as usize];
    let metadata = *spin_lock(&entry.mutex);
    if metadata.waiting_count == 0 {
        0
    } else if metadata.waiting_count < count || metadata.address.is_null() {
        entry.condvar.notify_all();
        metadata.waiting_count
    } else {
        for _ in 0..count {
            entry.condvar.notify_one();
        }
        count
    }
}

/// Locks `mutex` without allowing the thread to sleep.
/// Assumes that `mutex` is not poisoned.
fn spin_lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    loop {
        if let Ok(x) = mutex.try_lock() {
            return x;
        }

        spin_loop();
    }
}

/// Gets the entry index to use for the given address.
fn entry_for_ptr(ptr: *const ()) -> u8 {
    let x_64 = ptr as u64;
    let x_32 = (x_64 >> 32) as u32 ^ x_64 as u32;
    let x_16 = (x_32 >> 16) as u16 ^ x_32 as u16;
    (x_16 >> 8) as u8 ^ (x_16 >> 2) as u8
}

/// Holds metadata that gets written while locking.
#[derive(Copy, Clone)]
struct WaitMetadata {
    /// The address upon which all threads are waiting,
    /// or [`std::ptr::null`] if the address is different
    /// for each thread.
    pub address: *const (),
    /// The number of threads waiting on this table entry.
    pub waiting_count: usize,
}

impl WaitMetadata {
    /// The starting value for metadata.
    pub const DEFAULT: Self = Self {
        address: std::ptr::null(),
        waiting_count: 0,
    };
}

unsafe impl Send for WaitMetadata {}
unsafe impl Sync for WaitMetadata {}

/// Holds OS synchronization primitives for locking.
struct TableEntry {
    /// The condition variable on which to wait.
    pub condvar: Condvar,
    /// The mutex for locking before sleep.
    pub mutex: Mutex<WaitMetadata>,
}

impl TableEntry {
    /// The starting value for a table entry.
    #[allow(clippy::declare_interior_mutable_const)]
    pub const DEFAULT: Self = Self {
        condvar: Condvar::new(),
        mutex: Mutex::new(WaitMetadata::DEFAULT),
    };
}
