use core::time::Duration;

use ecmascript_atomics::Racy;

use crate::{FutexError, private::ECMAScriptAtomicWaitImpl};

impl ECMAScriptAtomicWaitImpl for Racy<'_, u32> {
    type AtomicInner = u32;

    fn wait_timeout(
        &self,
        value: Self::AtomicInner,
        timeout: Option<Duration>,
    ) -> Result<(), FutexError> {
        let result = unsafe {
            if let Some(time) = timeout {
                libc::os_sync_wait_on_address_with_timeout(
                    self.addr() as *mut libc::c_void,
                    value as u64,
                    size_of::<Self>(),
                    libc::OS_SYNC_WAIT_ON_ADDRESS_NONE,
                    libc::CLOCK_MONOTONIC,
                    time.as_nanos().min(u64::MAX as u128) as u64,
                )
            } else {
                libc::os_sync_wait_on_address(
                    self.addr() as *mut libc::c_void,
                    value as u64,
                    size_of::<Self>(),
                    libc::OS_SYNC_WAIT_ON_ADDRESS_NONE,
                )
            }
        };
        if result >= 0 {
            eprintln!("Result: {result:?}");
            Ok(())
        } else {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            eprintln!("Errno: {errno:?}");
            if errno == libc::ETIMEDOUT {
                Err(FutexError::Timeout)
            } else {
                Err(FutexError::Unknown)
            }
        }
    }

    fn notify_all(&self) -> usize {
        let result = unsafe {
            libc::os_sync_wake_by_address_all(
                self.addr() as *mut libc::c_void,
                size_of::<Self>(),
                libc::OS_SYNC_WAKE_BY_ADDRESS_NONE,
            )
        };
        if result == 0 {
            // At least one thread was woken up
            usize::MAX
        } else {
            // No threads were woken up.
            0
        }
    }

    fn notify_many(&self, count: usize) -> usize {
        let result = unsafe {
            libc::os_sync_wake_by_address_any(
                self.addr() as *mut libc::c_void,
                size_of::<Self>(),
                libc::OS_SYNC_WAKE_BY_ADDRESS_NONE,
            )
        };
        if result == 0 {
            // At least one thread was woken up; assume count.
            count
        } else {
            // No threads were woken up.
            0
        }
    }
}

impl ECMAScriptAtomicWaitImpl for Racy<'_, u64> {
    type AtomicInner = u64;

    fn wait_timeout(
        &self,
        value: Self::AtomicInner,
        timeout: Option<Duration>,
    ) -> Result<(), FutexError> {
        let result = unsafe {
            if let Some(time) = timeout {
                libc::os_sync_wait_on_address_with_timeout(
                    self.addr() as *mut libc::c_void,
                    value,
                    size_of::<Self>(),
                    libc::OS_SYNC_WAIT_ON_ADDRESS_NONE,
                    libc::CLOCK_MONOTONIC,
                    time.as_nanos().min(u64::MAX as u128) as u64,
                )
            } else {
                libc::os_sync_wait_on_address(
                    self.addr() as *mut libc::c_void,
                    value,
                    size_of::<Self>(),
                    libc::OS_SYNC_WAIT_ON_ADDRESS_NONE,
                )
            }
        };
        if result >= 0 {
            // Result indicates how many waiters remain.
            Ok(())
        } else {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            if errno == libc::ETIMEDOUT {
                Err(FutexError::Timeout)
            } else {
                Err(FutexError::Unknown)
            }
        }
    }

    fn notify_all(&self) -> usize {
        let result = unsafe {
            libc::os_sync_wake_by_address_all(
                self.addr() as *mut libc::c_void,
                size_of::<Self>(),
                libc::OS_SYNC_WAKE_BY_ADDRESS_NONE,
            )
        };
        if result == 0 {
            // At least one thread was woken up
            usize::MAX
        } else {
            // No threads were woken up.
            0
        }
    }

    fn notify_many(&self, count: usize) -> usize {
        let result = unsafe {
            libc::os_sync_wake_by_address_any(
                self.addr() as *mut libc::c_void,
                size_of::<Self>(),
                libc::OS_SYNC_WAKE_BY_ADDRESS_NONE,
            )
        };
        if result == 0 {
            // At least one thread was woken up; assume count.
            count
        } else {
            // No threads were woken up.
            0
        }
    }
}
