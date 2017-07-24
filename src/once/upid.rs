extern crate libc;
use std::sync::atomic::{AtomicUsize, Ordering};

static UPID: AtomicUsize = AtomicUsize::new(<usize>::max_value());

/// Get the current process' UPID.
///
/// The UPID (Unique Process ID) is an identifier used to distinguish a process (that has been
/// forked but has not exec'd) from its parent process. A process' UPID is guaranteed to be
/// distinct from the UPIDs of any of its ancestors with two important caveats:
/// * The UPID is not guaranteed to be distinct from non-ancestors such as sibling processes.
/// * The UPID is not guaranteed to be distinct from ancestors that called exec. To be precise, the
///   UPID is only guaranteed to be distinct from the UPID of a process that begat this process
///   via a series of calls to `fork` without any intervening calls to `exec`.
pub fn upid() -> usize {
    let upid = UPID.load(Ordering::Relaxed);
    if upid != <usize>::max_value() {
        return upid;
    }

    // NOTE: It's possible that multiple threads will end up in this code and thus atfork will get
    // registered multiple times. That's OK - it just means that the UPID will increase a bit
    // faster than if there were only one callback. This isn't a problem for correctness.
    //
    // Note also that the alternative - trying to somehow synchronize and ensure that
    // pthread_atfork is only called once - introduces a chicken-and-egg problem where we have to
    // figure out how to essentially implement Once without the UPID functionality that we're
    // building.
    unsafe { libc::pthread_atfork(None, None, Some(atfork)) };
    UPID.compare_and_swap(upid, 0, Ordering::Relaxed);
    // Since we didn't enter the first if block, we know that regardless of whether or not the CAS
    // succeeded, we're the initializing process, and thus our UPID is 0.
    0
}

unsafe extern "C" fn atfork() {
    // This could probably be a non-atomic increment (since this is only called right after fork
    // when no other threads are running), but it's easier this way.
    UPID.fetch_add(1, Ordering::Relaxed);
}

#[cfg(test)]
mod test {
    extern crate libc;

    #[test]
    fn upid() {
        // TODO: What's the right way to test this and play nicely with the Rust testing harness?
        // This test is very sketchy in that regard.
        unsafe {
            let parent = libc::getpid();
            let upid = super::upid();

            libc::fork();

            let pid = libc::getpid();
            if pid == parent {
                assert_eq!(upid, super::upid());
            } else {
                assert_ne!(upid, super::upid());
            }
        }
    }
}
