use core::time::Duration;

use ecmascript_atomics::{Ordering, Racy};

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
                    self.addr(),
                    value as u64,
                    size_of::<Self>(),
                    libc::OS_SYNC_WAIT_ON_ADDRESS_NONE,
                    libc::CLOCK_MONOTONIC,
                    time.as_nanos().min(u64::MAX as u128) as u64,
                )
            } else {
                libc::os_sync_wait_on_address(
                    self.addr(),
                    value as u64,
                    size_of::<Self>(),
                    libc::OS_SYNC_WAIT_ON_ADDRESS_NONE,
                )
            }
        };
        if result >= 0 {
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
        unsafe {
            libc::os_sync_wake_by_address_all(
                self.addr(),
                size_of::<Self>(),
                libc::OS_SYNC_WAKE_BY_ADDRESS_NONE,
            );
        };
    }

    fn notify_many(&self, count: usize) -> usize {
        unsafe {
            libc::os_sync_wake_by_address_any(
                self.addr(),
                size_of::<Self>(),
                libc::OS_SYNC_WAKE_BY_ADDRESS_NONE,
            );
        };
    }
}

impl ECMAScriptAtomicWaitImpl for Racy<'_, u64> {
    type AtomicInner = u64;

    fn wait_timeout(
        &self,
        value: Self::AtomicInner,
        timeout: Option<Duration>,
    ) -> Result<(), FutexError> {
        unsafe {
            if let Some(time) = timeout {
                libc::os_sync_wait_on_address_with_timeout(
                    self.addr(),
                    value,
                    size_of::<Self>(),
                    libc::OS_SYNC_WAIT_ON_ADDRESS_NONE,
                    libc::CLOCK_MONOTONIC,
                    time.as_nanos().min(u64::MAX as u128) as u64,
                );
            } else {
                libc::os_sync_wait_on_address(
                    self.addr(),
                    value,
                    size_of::<Self>(),
                    libc::OS_SYNC_WAIT_ON_ADDRESS_NONE,
                );
            }
        }
    }

    fn notify_all(&self) -> usize {
        unsafe {
            libc::os_sync_wake_by_address_all(
                self.addr(),
                size_of::<Self>(),
                libc::OS_SYNC_WAKE_BY_ADDRESS_NONE,
            );
        };
    }

    fn notify_many(&self, count: usize) -> usize {
        unsafe {
            libc::os_sync_wake_by_address_any(
                self.addr(),
                size_of::<Self>(),
                libc::OS_SYNC_WAKE_BY_ADDRESS_NONE,
            );
        };
    }
}
