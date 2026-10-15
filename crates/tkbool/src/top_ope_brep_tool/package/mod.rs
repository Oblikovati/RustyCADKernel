// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool.hxx/.cxx and
// TopOpeBRepTool_RegularizeS.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `TopOpeBRepTool` package functions: the regularization of wires, faces and
//! shells, the purge of faulty closing edges and the name of a curve option.
mod purge;
mod regularize_w;

pub use purge::{
    correct_on_uviso, fun_tool_closed_w, make_faces, purge_closing_edges, purge_closing_edges_face,
};
pub use regularize_w::{fun_tool_classif_w, regularize, regularize_face, regularize_wires};

use crate::top_ope_brep_tool::geom_tool::TopOpeBRepToolOutCurveType;
use crate::top_ope_brep_tool::regus::TopOpeBRepToolREGUS;
use crate::top_ope_brep_tool::tool::ShapeListMap;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Solid;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::io::{self, Write};

/// `TopOpeBRepTool::Print(t, OS)`: the name of the curve option;
/// `print_out_curve_type(TopOpeBRepToolOutCurveType::Approx, &mut out)?`.
pub fn print_out_curve_type(t: TopOpeBRepToolOutCurveType, os: &mut impl Write) -> io::Result<()> {
    let name = match t {
        TopOpeBRepToolOutCurveType::BSpline1 => "BSPLINE1",
        TopOpeBRepToolOutCurveType::Approx => "APPROX",
        TopOpeBRepToolOutCurveType::Interpol => "INTERPOL",
    };
    write!(os, "{name}")
}

/// `TopOpeBRepTool::RegularizeShells(theSolid, OldSheNewShe, FSplits)`: each shell of
/// the solid regularized (`TopOpeBRepTool_REGUS`), the new shells stored by old shell
/// and the splits of its faces by face; false when a shell cannot be mapped or its
/// faces split; `regularize_shells(&solid, &mut osh_nsh, &mut fsplits)?`.
pub fn regularize_shells(
    the_solid: &Solid,
    old_she_new_she: &mut ShapeListMap,
    f_splits: &mut ShapeListMap,
) -> Result<bool, Failure> {
    old_she_new_she.clear(false);
    f_splits.clear(false);
    let mut regus = TopOpeBRepToolREGUS::new();
    regus.set_osh_nsh(old_she_new_she);
    regus.set_fsplits(f_splits);
    for sh in Explorer::new(the_solid, ShapeEnum::Shell, ShapeEnum::Shape)? {
        regus.init(&sh);
        if !regus.map_s()? || !regus.split_faces()? {
            return Ok(false);
        }
        regus.regu()?;
    }
    *old_she_new_she = regus.get_osh_nsh();
    *f_splits = regus.get_fsplits();
    Ok(true)
}
