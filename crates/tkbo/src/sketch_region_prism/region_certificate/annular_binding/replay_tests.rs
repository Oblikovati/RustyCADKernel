// SPDX-License-Identifier: GPL-2.0-only
//! Whole owned proof operands replay without transient addresses across processes and threads.
use super::*;
fn fresh(binding: &AnnularPrismBinding) -> Vec<u8> {
    replay::encode(&binding.region, &binding.operands).unwrap_or_else(|_| panic!("fresh owned operand encoding"))
}
#[test]
fn annular_binding_process_replay() {
    let binding = tests::actual_binding();
    let serial = fresh(&binding);
    assert_eq!(serial, binding.replay_bytes());
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..3).map(|_| scope.spawn(|| {
            for _ in 0..4 { assert_eq!(fresh(&binding), serial); }
            let independent = tests::actual_binding();
            assert_eq!(fresh(&independent), serial);
        })).collect();
        for worker in workers { worker.join().unwrap(); }
    });
    let encoded: String = serial.iter().map(|b| format!("{b:02x}")).collect();
    println!("actual-annular-binding-v2:{encoded}");
}
