use ecmascript_atomics::{Ordering, RacyBox};
use ecmascript_futex::{ECMAScriptAtomicWait, FutexError};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

#[test]
fn wake_nothing() {
    let a = RacyBox::new(0u32).unwrap();
    let a = a.as_slice().get(0).unwrap();
    assert_eq!(a.notify_many(1), 0);
    assert_eq!(a.notify_all(), 0);
}

#[test]
fn wait_unexpected() {
    let t = Instant::now();
    let a = RacyBox::new(0u32).unwrap();
    let a = a.as_slice().get(0).unwrap();
    // Note: Windows doesn't report early-exits.
    #[cfg(not(windows))]
    assert_eq!(a.wait(1), Err(FutexError::NotEqual));
    #[cfg(windows)]
    assert_eq!(a.wait(1), Ok(()));
    assert!(t.elapsed().as_millis() < 100);
}

#[test]
fn wait_wake() {
    let t = Instant::now();
    let a = RacyBox::new(0u32).unwrap();
    let a = a.as_slice().get(0).unwrap();
    std::thread::scope(|s| {
        s.spawn(|| {
            sleep(Duration::from_millis(100));
            a.store(1, Ordering::Unordered);
            assert_eq!(a.notify_many(1), 1);
        });
        while a.load(Ordering::Unordered) == 0 {
            assert!(a.wait(0).is_ok());
        }
        assert_eq!(a.load(Ordering::Unordered), 1);
        assert!((90..400).contains(&t.elapsed().as_millis()));
    });
}

#[test]
fn wait_timeout() {
    let a = RacyBox::new(0u32).unwrap();
    let a = a.as_slice().get(0).unwrap();
    assert_eq!(
        a.wait_timeout(0, Duration::from_millis(1)),
        Err(FutexError::Timeout)
    );
}

#[test]
fn stress_many_waiters_notify_all() {
    let a = RacyBox::new(0u32).unwrap();
    let woke = RacyBox::new(0u32).unwrap();

    let threads = 64;
    let a = a.as_slice().get(0).unwrap();
    let woke = woke.as_slice().get(0).unwrap();
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(move || {
                while a.load(Ordering::Unordered) == 0 {
                    assert!(a.wait(0).is_ok());
                }
                woke.fetch_add(1);
            });
        }

        // Give threads time to start waiting
        sleep(Duration::from_millis(50));
        a.store(1, Ordering::Unordered);
        assert!(a.notify_all() >= threads as usize);
    });
    assert_eq!(woke.load(Ordering::Unordered), threads);
}

#[test]
fn stress_ping_pong_many_iters() {
    let state = RacyBox::new(0u32).unwrap();
    let iters = 5_000u32;

    let state = state.as_slice().get(0).unwrap();
    std::thread::scope(|s| {
        s.spawn(move || {
            // Consumer: wait for 1, reset to 0, and notify producer.
            for _ in 0..iters {
                while state.load(Ordering::Unordered) != 1 {
                    // Wait while the state is 0; use a short timeout to be resilient to spurious wakes.
                    let _ = state.wait_timeout(0, Duration::from_millis(10)).is_ok();
                }
                state.store(0, Ordering::Unordered);
                assert!(state.notify_many(1) <= 1);
            }
        });

        // Producer: set to 1, notify consumer, then wait until it resets to 0.
        for _ in 0..iters {
            state.store(1, Ordering::Unordered);
            assert!(state.notify_many(1) <= 1);
            while state.load(Ordering::Unordered) != 0 {
                let _ = state.wait_timeout(1, Duration::from_millis(10));
            }
        }
    });
    // Final state should be 0 after a complete ping-pong.
    assert_eq!(state.load(Ordering::Unordered), 0);
}
