//! Confirms the native sleeper returns when `usleep` is interrupted by a signal.
#![cfg(target_os = "linux")]
#![allow(unsafe_code)]

use rustycad_tkernel::osd::Osd;
use std::{
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

static ALARM_CAUGHT: AtomicBool = AtomicBool::new(false);
const CHILD_MARKER: &str = "RUSTYCAD_SEC_SLEEP_SIGNAL_CHILD";

#[test]
fn sec_sleep_returns_when_signal_interrupts_usleep() {
    let mut child = spawn_sleep_child();
    assert!(child_exits_before_timeout(&mut child));
}

#[test]
fn sec_sleep_child() {
    if std::env::var_os(CHILD_MARKER).is_none() {
        return;
    }
    let sleeper_thread = install_alarm();
    let signaler = thread::spawn(move || send_alarm(sleeper_thread));
    Osd::sec_sleep(30);
    assert!(ALARM_CAUGHT.load(Ordering::Relaxed));
    signaler.join().expect("signal delivery thread");
}

fn spawn_sleep_child() -> Child {
    Command::new(std::env::current_exe().expect("test executable path"))
        .args(["--exact", "sec_sleep_child"])
        .env(CHILD_MARKER, "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn bounded signal probe")
}

fn child_exits_before_timeout(child: &mut Child) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("poll signal probe") {
            return status.success();
        }
        thread::sleep(Duration::from_millis(10));
    }
    let _ = child.kill();
    let _ = child.wait();
    false
}

fn install_alarm() -> libc::pthread_t {
    ALARM_CAUGHT.store(false, Ordering::Relaxed);
    let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
    action.sa_sigaction = catch_alarm as *const () as usize;
    action.sa_flags = 0;
    // SAFETY: action is initialized and the signal APIs receive valid process-owned storage.
    unsafe {
        libc::sigemptyset(&mut action.sa_mask);
        libc::sigaction(libc::SIGALRM, &action, std::ptr::null_mut());
    }
    // SAFETY: this returns the calling pthread identifier for targeted signal delivery.
    unsafe { libc::pthread_self() }
}

fn send_alarm(thread_id: libc::pthread_t) {
    thread::sleep(Duration::from_millis(100));
    // SAFETY: the child test's sleeper thread remains alive while blocked in OSD::SecSleep.
    let result = unsafe { libc::pthread_kill(thread_id, libc::SIGALRM) };
    assert_eq!(result, 0);
}

extern "C" fn catch_alarm(_: libc::c_int) {
    ALARM_CAUGHT.store(true, Ordering::Relaxed);
}
