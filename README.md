# ecmascript_futex

Cross platform library for implementing ECMAScript `Atomics.wait`,
`Atomics.wakeAsync`, and `Atomics.notify` (aka futex) functionality in Rust,
operating on ECMAScript memory as produced by the
[`ecmascript_atomics`](https://github.com/trynova/ecmascript_atomics) crate.
This crate is a fork of
[`wait_on_address`](https://github.com/DouglasDwyer/wait_on_address) which is
itself a fork of [`atomic-wait`](https://github.com/m-ou-se/atomic-wait). The
changes inherited and kept from `wait_on_address` are:

- Support for waiting with a timeout
- Support for `wasm32` on nightly using `std::arch`
- Polyfill for all other platforms

The main 

Natively-supported platforms:

- Windows 8+, Windows Server 2012+
- macOS 14.4+, iOS 17.4+, watchOS 10.4+
- Linux 2.6.22+ (using fallback for 64-bit futexes)
- wasm32
- All other platforms with `std` support (using fallback)

## Usage

```rust
use core::time::Duration;
use ecmascript_atomics::{Racy, RacyBox};
use ecmascript_futex::ECMAScriptAtomicWait;

let a = RacyBox::new(0u64).unwrap();
let a = a.as_slice().get(0).unwrap();

a.wait(1); // If the value is 1, wait.

a.wait_timeout(2, Duration::from_millis(100));  // If the value is 2, wait at most 100 milliseconds

a.notify_many(1); // Wake one waiting thread.

a.notify_all(); // Wake all waiting threads.
```

## Implementation

On Linux, this uses the `SYS_futex` syscall.

On FreeBSD, this uses the `_umtx_op` syscall.

On Windows, this uses the `WaitOnAddress` and `WakeByAddress` APIs.

On macOS (and iOS and watchOS), this uses the `os_sync_wait_on_address` and
`os_sync_wake_by_address` APIs.

On wasm32 with `nightly`, this uses `memory_atomic_wait32`,
`memory_atomic_wait64`, and `memory_atomic_notify` instructions.

All other platforms with `std` support fall back to a fixed-size hashmap of
`Condvar`s, similar to `libstdc++`'s implementation for `std::atomic<T>`.
