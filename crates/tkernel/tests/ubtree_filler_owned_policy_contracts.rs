//! Retained assignment policy on the actual filler queue, including native noexcept Drop.
#[path = "support/ubtree_allocation_trace.rs"]
pub mod allocation_trace;
#[path = "ubtree_filler_owned_policy_contracts/compatibility.rs"]
mod compatibility;
#[path = "ubtree_filler_owned_policy_contracts/phases.rs"]
mod phases;
#[cfg(not(miri))]
#[path = "ubtree_filler_owned_policy_contracts/termination.rs"]
mod termination;
use allocation_trace::*;
use rustycad_tkernel::ncollection::ObjBnd;
fn self_assign(pair: &mut ObjBnd<Payload<'O'>, Payload<'B'>>) {
    pair.assign_self_with(Payload::assign_self, Payload::assign_self);
}
fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
fn native_mode(mode: &str) -> Vec<&'static str> {
    include_str!("../../../tools/oracle/tkernel_ubtree_filler_owned_policy_contracts.expected")
        .split(&format!("case {mode} exit "))
        .nth(1)
        .unwrap()
        .lines()
        .skip(1)
        .take_while(|line| !line.starts_with("case "))
        .collect()
}
