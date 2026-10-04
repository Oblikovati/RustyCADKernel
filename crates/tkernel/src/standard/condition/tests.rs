use super::StandardCondition;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn check_reads_atomic_flag_without_acquiring_waiter_mutex() {
    let condition = StandardCondition::new(true);
    let guard = condition.waiter_guard();
    std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::channel();
        let event = &condition;
        scope.spawn(move || sender.send(event.check()).unwrap());
        let observed = receiver.recv_timeout(Duration::from_secs(1));
        // Release before asserting so a regressed locking check cannot strand a worker.
        drop(guard);
        assert_eq!(observed, Ok(true));
    });
}

#[test]
fn duration_adapter_preserves_manual_reset_state() {
    let condition = StandardCondition::new(true);
    assert!(condition.wait_for(Duration::ZERO));
    assert!(condition.check_reset());
    assert!(!condition.wait_for(Duration::ZERO));
    condition.set();
    assert!(condition.wait_for(Duration::ZERO));
    condition.reset();
    assert!(!condition.wait_for(Duration::ZERO));
}
