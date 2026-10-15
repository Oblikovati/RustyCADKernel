// Rust translation of ModelingAlgorithms/TKFillet/ChFiKPart at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! ChFiKPart: the fillets and chamfers of the particular cases computed in closed form
//! (planes, cylinders, cones, the corner spheres and tori), written into the
//! TopOpeBRepDS data structure.
mod ch_pln_con;
mod ch_pln_cyl;
mod ch_pln_cyl_line;
mod common;
mod compute_chamfer;
mod compute_data;
mod corner_spine;
mod fcts;
mod fil_pln_con;
mod fil_pln_cyl;
mod pln_pln;
mod sphere_rotule;

pub use ch_pln_con::{make_ch_asym_pln_con, make_chamfer_pln_con};
pub use ch_pln_cyl::{make_ch_asym_pln_cyl_circle, make_chamfer_pln_cyl_circle};
pub use ch_pln_cyl_line::{make_ch_asym_pln_cyl_line, make_chamfer_pln_cyl_line};
pub use common::{index_curve_in_ds, index_surface_in_ds};
pub use compute_data::ChFiKPartComputeData;
pub use corner_spine::corner_spine;
pub use fcts::{in_period, pcurve, proj_pc};
pub use fil_pln_con::make_fillet_pln_con;
pub use fil_pln_cyl::{make_fillet_pln_cyl_circle, make_fillet_pln_cyl_line};
pub use pln_pln::{make_ch_asym_pln_pln, make_chamfer_pln_pln, make_fillet_pln_pln};
pub use sphere_rotule::{make_rotule, sphere};
