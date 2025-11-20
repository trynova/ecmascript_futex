use core::time::Duration;

use ecmascript_atomics::{Ordering, Racy};

use crate::{FutexError, condvar_table, private::ECMAScriptAtomicWaitImpl};

impl ECMAScriptAtomicWaitImpl for Racy<'_, u32> {
    type AtomicInner = u32;

    fn wait_timeout(
        &self,
        value: Self::AtomicInner,
        timeout: Option<Duration>,
    ) -> Result<(), FutexError> {
        unsafe {
            let wait_timespec = timeout.map(|x| libc::timespec {
                tv_sec: x.as_secs() as i64,
                tv_nsec: x.subsec_nanos() as i64,
            });

            let result = libc::syscall(
                libc::SYS_futex,
                self.addr(),
                libc::FUTEX_WAIT | libc::FUTEX_PRIVATE_FLAG,
                value,
                wait_timespec
                    .as_ref()
                    .map(|x| x as *const _)
                    .unwrap_or(std::ptr::null()),
            );
            if result == 0 {
                Ok(())
            } else {
                let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                if errno == libc::EAGAIN {
                    Err(FutexError::NotEqual)
                } else if errno == libc::ETIMEDOUT {
                    Err(FutexError::Timeout)
                } else if errno == libc::EINTR {
                    // We consider spurious interrupts to still be valid
                    // wakeups.
                    Ok(())
                } else {
                    Err(FutexError::Unknown)
                }
            }
        }
    }

    fn notify_all(&self) -> usize {
        unsafe {
            libc::syscall(
                libc::SYS_futex,
                self.addr(),
                libc::FUTEX_WAKE | libc::FUTEX_PRIVATE_FLAG,
                i32::MAX,
            )
            .unsigned_abs() as usize
        }
    }

    fn notify_many(&self, count: usize) -> usize {
        unsafe {
            libc::syscall(
                libc::SYS_futex,
                self.addr(),
                libc::FUTEX_WAKE | libc::FUTEX_PRIVATE_FLAG,
                count.min(i32::MAX as usize) as i32,
            )
            .unsigned_abs() as usize
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
        condvar_table::wait(
            self.addr(),
            || self.load(Ordering::SeqCst) == value,
            timeout,
        )
    }

    fn notify_all(&self) -> usize {
        condvar_table::notify_all(self.addr())
    }

    fn notify_many(&self, count: usize) -> usize {
        condvar_table::notify_many(self.addr(), count)
    }
}
