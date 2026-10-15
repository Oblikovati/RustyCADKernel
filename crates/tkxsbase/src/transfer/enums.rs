// Rust port of DataExchange/TKXSBase/Transfer/Transfer_StatusResult.hxx,
// Transfer_StatusExec.hxx, Transfer_TransferFailure.hxx and Transfer_TransferDeadLoop.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The statuses of a transfer binder and the exceptions of the Transfer package (as
//! [`Failure`] values with their native names).
use rustycad_tkernel::standard::Failure;

/// `Transfer_StatusResult`: whether a binder has a result, and whether it was used.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TransferStatusResult {
    /// `Transfer_StatusVoid`.
    #[default]
    StatusVoid = 0,
    /// `Transfer_StatusDefined`.
    StatusDefined = 1,
    /// `Transfer_StatusUsed`.
    StatusUsed = 2,
}

/// `Transfer_StatusExec`: the execution status of a transfer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TransferStatusExec {
    /// `Transfer_StatusInitial`.
    #[default]
    StatusInitial = 0,
    /// `Transfer_StatusRun`.
    StatusRun = 1,
    /// `Transfer_StatusDone`.
    StatusDone = 2,
    /// `Transfer_StatusError`.
    StatusError = 3,
    /// `Transfer_StatusLoop`.
    StatusLoop = 4,
}

/// `Transfer_TransferFailure` (an `Interface_InterfaceError`);
/// `transfer_failure("Binder : SetResult, Result is Already Set")`.
pub fn transfer_failure(message: &str) -> Failure {
    Failure::with_type("Transfer_TransferFailure", message, "")
}

/// `Transfer_TransferDeadLoop` (a `Transfer_TransferFailure`);
/// `transfer_dead_loop("TransferProcess : Transfer in Loop")`.
pub fn transfer_dead_loop(message: &str) -> Failure {
    Failure::with_type("Transfer_TransferDeadLoop", message, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_and_names() {
        assert_eq!(TransferStatusResult::StatusUsed as i32, 2);
        assert_eq!(TransferStatusExec::StatusLoop as i32, 4);
        assert_eq!(
            transfer_failure("x").exception_type(),
            "Transfer_TransferFailure"
        );
        assert_eq!(
            transfer_dead_loop("x").exception_type(),
            "Transfer_TransferDeadLoop"
        );
    }

    /// Every enumerator keeps its native Transfer_Status*.hxx order and value.
    #[test]
    fn every_enumerator_keeps_native_value() {
        use TransferStatusExec as X;
        let exec = [
            X::StatusInitial,
            X::StatusRun,
            X::StatusDone,
            X::StatusError,
            X::StatusLoop,
        ];
        assert!(exec.iter().enumerate().all(|(i, v)| *v as i32 == i as i32));
        use TransferStatusResult as R;
        let result = [R::StatusVoid, R::StatusDefined, R::StatusUsed];
        assert!(
            result
                .iter()
                .enumerate()
                .all(|(i, v)| *v as i32 == i as i32)
        );
    }
}
