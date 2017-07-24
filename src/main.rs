#![feature(const_fn)]

mod once;

extern crate libc;
use std::{thread, time};
use once::Once;

fn main() {
    unsafe {
        let parent = libc::getpid();

        // Spawn and then sleep for 100 ms to give the spawned thread a chance to start (but not
        // finish) computing slowly_do.
        thread::spawn(|| SLOW.call_once(slowly_do));
        thread::sleep(time::Duration::from_millis(100));

        libc::fork();

        // Print the current PID after executing slowly_do in SLOW. Since SLOW should already be in
        // the middle of being initialized (but not done - slowly_do takes 200 ms, and we've only
        // waited for 100 ms),  using SLOW should block until the spawned thread finishes computing
        // slowly_do. However, the child has a copy-on-write version of the parent's memory space,
        // and so the waiter object that is enqueued in only written to the child's memory space,
        // and isn't reflected in the parent. Thus, when the call to slowly_do completes, the
        // thread executing it won't know to wake up the child process, and so using SLOW in the
        // child will block forever.
        SLOW.call_once(slowly_do);
        println!("SLOW in {}", libc::getpid());


        if libc::getpid() == parent {
            // Give the child a full second to complete before giving up (once the parent exits,
            // the terminal/play.rust-lang.org could close stdin/stdout and clean up, so we
            // wouldn't be sure that we hadn't just missed the output from the childing printing
            // above).
            thread::sleep(time::Duration::from_secs(1));
        }
    }
}

static SLOW: Once = Once::new();

fn slowly_do() {
    // If this is changed to 0 ms, the child will print its value because slowly_do will return
    // before the child is spawned.
    thread::sleep(time::Duration::from_millis(200));
}
