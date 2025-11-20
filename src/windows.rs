use core::time::Duration;

use ecmascript_atomics::{Ordering, Racy};
use windows_sys::Win32::System::Threading::{
    INFINITE, WaitOnAddress, WakeByAddressAll, WakeByAddressSingle,
};

use crate::{FutexError, private::ECMAScriptAtomicWaitImpl};

impl ECMAScriptAtomicWaitImpl for Racy<'_, u32> {
    type AtomicInner = u32;

    fn wait_timeout(
        &self,
        value: Self::AtomicInner,
        timeout: Option<Duration>,
    ) -> Result<(), FutexError> {
        unsafe {
            WaitOnAddress(
                self.addr(),
                &value as *const _ as *const _,
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
            );
        }
    }

    fn notify_all(&self) -> usize {
        unsafe { WakeByAddressAll(self.addr()) };
    }

    fn notify_many(&self, count: usize) -> usize {
        unsafe { WakeByAddressSingle(self.addr()) };
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
            WaitOnAddress(
                self.addr(),
                &value as *const _ as *const _,
                size_of::<Self>(),
                timeout
                    .map(|x| x.as_millis().min(u32::MAX as u128 - 1) as u32)
                    .unwrap_or(INFINITE),
            );
        }
    }

    fn notify_all(&self) -> usize {
        unsafe { WakeByAddressAll(self.addr()) };
    }

    fn notify_many(&self, count: usize) -> usize {
        unsafe { WakeByAddressSingle(self.addr()) };
    }
}
