// SPDX-License-Identifier: GPL-2.0-only
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualError;
#[derive(Debug)]
pub(crate) enum CapturedGeometryCause {
    Native(Failure),
    MissingCurve {
        representation: &'static str,
    },
    Unsupported {
        representation: &'static str,
        dynamic_type: String,
    },
    ParameterRanges {
        curve: [f64; 2],
        pcurve: [f64; 2],
    },
    MissingSameParameter,
    MissingStoredPCurve,
    Exact(ResidualError),
}
impl From<Failure> for CapturedGeometryCause {
    fn from(cause: Failure) -> Self {
        Self::Native(cause)
    }
}
impl From<ResidualError> for CapturedGeometryCause {
    fn from(cause: ResidualError) -> Self {
        Self::Exact(cause)
    }
}
