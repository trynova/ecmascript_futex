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
        condvar_table::wait(
            self.addr(),
            || self.load(Ordering::SeqCst) == value,
            timeout,
        )
    }

    fn notify_all(&self) -> usize {
        condvar_table::notify_all(self.addr());
    }

    fn notify_many(&self, count: usize) -> usize {
        condvar_table::notify_many(self.addr(), count);
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
        condvar_table::notify_all(self.addr());
    }

    fn notify_many(&self, count: usize) -> usize {
        condvar_table::notify_many(self.addr(), count);
    }
}
