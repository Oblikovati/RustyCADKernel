//! CSLib surface-normal computation and 2D point-in-polygon classification.
//!
//! The package-level static functions of OCCT's `CSLib` class are free functions
//! here; C++ output references become the returned [`NormalResult`] and
//! [`SingularNormal`] values, and C++ exceptions become [`Failure`] errors.
//!
//! [`Failure`]: rustycad_tkernel::standard::Failure
mod class2d;
mod derivatives;
mod normal;
mod normal_poly;
mod singular;
mod status;

pub use class2d::{Class2d, Class2dResult};
pub use derivatives::{dn_normal, dnnuv, dnnuv_osculating};
pub use normal::{normal, normal_from_second_derivatives, normal_with_magnitude};
pub use normal_poly::NormalPolyDef;
pub use singular::{ParametricPosition, SingularNormal, normal_at_singularity};
pub use status::{DerivativeStatus, NormalResult, NormalStatus};
