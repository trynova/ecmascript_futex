use core::time::Duration;

use ecmascript_atomics::Racy;
use windows_sys::Win32::{
    Foundation::ERROR_TIMEOUT,
    System::Threading::{INFINITE, WaitOnAddress, WakeByAddressAll, WakeByAddressSingle},
};

use crate::{FutexError, private::ECMAScriptAtomicWaitImpl};

impl ECMAScriptAtomicWaitImpl for Racy<'_, u32> {
    type AtomicInner = u32;

    fn wait_timeout(
        &self,
        value: Self::AtomicInner,
        timeout: Option<Duration>,
    ) -> Result<(), FutexError> {
        let result = unsafe {
            WaitOnAddress(
                self.addr() as *const core::ffi::c_void,
                &value as *const core::ffi::c_void,
                size_of::<Self>(),
                timeout
                    .map(|x| {
                        // Clamp to a finite u32 millisecond timeout. INFINITE (0xFFFFFFFF)
                        // means no timeout, so avoid ever passing that when a timeout is set.
                        let ms = x.as_millis();
                        let capped = ms.min(u32::MAX as u128 - 1);
                        capped as u32
                    })
                    .unwrap_or(INFINITE),
            )
        };
        if result {
            Ok(())
        } else {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            if errno == ERROR_TIMEOUT as i32 {
                Err(FutexError::Timeout)
            } else {
                Err(FutexError::Unknown)
            }
        }
    }

    fn notify_all(&self) -> usize {
        unsafe { WakeByAddressAll(self.addr()) };
        usize::MAX
    }

    fn notify_many(&self, count: usize) -> usize {
        for _ in 0..count {
            unsafe { WakeByAddressSingle(self.addr()) };
        }
        count
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
            WaitOnAddress(
                self.addr() as *const core::ffi::c_void,
                &value as *const core::ffi::c_void,
                size_of::<Self>(),
                timeout
                    .map(|x| x.as_millis().min(u32::MAX as u128 - 1) as u32)
                    .unwrap_or(INFINITE),
            )
        };
        if result == 1 {
            Ok(())
        } else {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            if errno == ERROR_TIMEOUT as i32 {
                Err(FutexError::Timeout)
            } else {
                Err(FutexError::Unknown)
            }
        }
    }

    fn notify_all(&self) -> usize {
        unsafe { WakeByAddressAll(self.addr()) };
        usize::MAX
    }

    fn notify_many(&self, count: usize) -> usize {
        for _ in 0..count {
            unsafe { WakeByAddressSingle(self.addr()) };
        }
        count
    }
}
