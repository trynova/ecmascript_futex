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
                let wait_timespec = libc::_umtx_time {
                    _clockid: libc::CLOCK_MONOTONIC as u32,
                    _flags: libc::UMTX_ABSTIME,
                    _timeout: libc::timespec {
                        tv_sec: time.as_secs() as i64,
                        tv_nsec: time.subsec_nanos() as i64,
                    },
                };

                libc::_umtx_op(
                    self.addr(),
                    libc::UMTX_OP_WAIT_UINT_PRIVATE,
                    value as u64,
                    size_of::<libc::_umtx_time>() as *mut _,
                    &wait_timespec as *const _ as *mut _,
                )
            } else {
                libc::_umtx_op(
                    self.addr(),
                    libc::UMTX_OP_WAIT_UINT_PRIVATE,
                    value as u64,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            }
        };
        if result >= 0 {
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

    fn notify_all(&self) -> usize {
        let result = unsafe {
            libc::_umtx_op(
                self.addr(),
                libc::UMTX_OP_WAKE_PRIVATE,
                i32::MAX as libc::c_ulong,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if result > 0 {
            result as usize
        } else if result == 0 {
            usize::MAX
        } else {
            0
        }
    }

    fn notify_many(&self, count: usize) -> usize {
        let result = unsafe {
            libc::_umtx_op(
                self.addr(),
                libc::UMTX_OP_WAKE_PRIVATE,
                1 as libc::c_ulong,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if result > 0 {
            (result as usize).min(count)
        } else if result == 0 {
            count
        } else {
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
                let wait_timespec = libc::_umtx_time {
                    _clockid: libc::CLOCK_MONOTONIC as u32,
                    _flags: libc::UMTX_ABSTIME,
                    _timeout: libc::timespec {
                        tv_sec: time.as_secs() as i64,
                        tv_nsec: time.subsec_nanos() as i64,
                    },
                };

                libc::_umtx_op(
                    self.addr(),
                    libc::UMTX_OP_WAIT,
                    value,
                    size_of::<libc::_umtx_time>() as *mut _,
                    &wait_timespec as *const _ as *mut _,
                )
            } else {
                libc::_umtx_op(
                    self.addr(),
                    libc::UMTX_OP_WAIT,
                    value,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            }
        };
        if result >= 0 {
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

    fn notify_all(&self) -> usize {
        let result = unsafe {
            libc::_umtx_op(
                self.addr(),
                libc::UMTX_OP_WAKE_PRIVATE,
                i32::MAX as libc::c_ulong,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if result > 0 {
            result as usize
        } else if result == 0 {
            usize::MAX
        } else {
            0
        }
    }

    fn notify_many(&self, count: usize) -> usize {
        let result = unsafe {
            libc::_umtx_op(
                self.addr(),
                libc::UMTX_OP_WAKE_PRIVATE,
                1 as libc::c_ulong,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if result > 0 {
            (result as usize).min(count)
        } else if result == 0 {
            count
        } else {
            0
        }
    }
}
