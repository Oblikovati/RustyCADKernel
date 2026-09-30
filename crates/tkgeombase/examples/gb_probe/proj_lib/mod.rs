//! The Rust twin of tools/oracle/tkgeombase_probe_projlib.cxx and
//! tkgeombase_probe_projlib_curves.cxx.
mod comp;
mod curves;
mod elementary;
mod on_plane;

pub use comp::comp_case;
pub use curves::{approx_case, polar_case, projected_case};
pub use elementary::{elem_case, prjfunc_case, prjresolve_case, static_case};
pub use on_plane::on_plane_case;
