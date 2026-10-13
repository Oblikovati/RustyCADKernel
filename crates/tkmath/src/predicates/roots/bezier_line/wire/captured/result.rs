// SPDX-License-Identifier: GPL-2.0-only
use super::super::{BezierWireError, BezierWireLocation};
/// Completed exact wire classification plus actual operation-local work receipts.
#[derive(Clone, Debug)]
pub struct CapturedBezierWireResult {
    pub(super) location: BezierWireLocation,
    pub(super) work: CaptureWorkReceipt,
}
/// Actual attempt cost remains observable when no complete wire is published.
#[derive(Debug)]
pub struct CapturedBezierWireFailure {
    pub(super) cause: BezierWireError,
    pub(super) work: CaptureWorkReceipt,
}
/// New execution work is distinct from historical evidence reused by assembly.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CaptureWorkReceipt {
    /// Completed subdivisions actually performed by this call.
    pub new_bisections: u16,
    /// Completed subdivisions charged across this operation, including failed attempts.
    pub operation_bisections: u16,
    /// Existing successful certificates consumed in this traversal.
    pub reused_certificates: usize,
    /// Historical certificate costs for successful occurrences, including reused proofs.
    pub certificate_bisections: u64,
}
impl CapturedBezierWireResult {
    /// Read the exact classification; example: `result.location()`.
    pub fn location(&self) -> &BezierWireLocation {
        &self.location
    }
    /// Read actual costs separately from reused proof costs; example: `result.work()`.
    pub fn work(&self) -> CaptureWorkReceipt {
        self.work
    }
}
impl CapturedBezierWireFailure {
    /// Preserve the original typed cause; example: `failure.cause()`.
    pub fn cause(&self) -> &BezierWireError {
        &self.cause
    }
    /// Read failed-attempt work without fabricating a successful proof; example: `failure.work()`.
    pub fn work(&self) -> CaptureWorkReceipt {
        self.work
    }
}

impl std::fmt::Display for CapturedBezierWireFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cause.fmt(formatter)
    }
}
impl std::error::Error for CapturedBezierWireFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
