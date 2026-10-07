// Rust translation of BRepAlgo::IsValid and BRepAlgo::IsTopologicallyValid
// (ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_1.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The validity of a shape, and of the result of an operation on arguments: its new
//! faces checked (their same-parameter flags forced again), then its shells.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_check::{
    BRepCheckAnalyzer, BRepCheckResult, BRepCheckShell, BRepCheckStatus,
};
use rustycad_tktopalgo::brep_lib;

/// `BRepAlgo::IsValid(S)`: whether BRepCheck_Analyzer (with the geometric controls)
/// finds the shape valid; `brep_algo::is_valid(&shape)?`.
pub fn is_valid(s: &Shape) -> Result<bool, Failure> {
    BRepCheckAnalyzer::new(s, true, false, false)?.is_valid()
}

/// `BRepAlgo::IsTopologicallyValid(S)`: the analyzer without geometric controls;
/// `brep_algo::is_topologically_valid(&shape)?`.
pub fn is_topologically_valid(s: &Shape) -> Result<bool, Failure> {
    BRepCheckAnalyzer::new(s, false, false, false)?.is_valid()
}

/// The faces of the arguments, and whether the result is one of them.
fn argument_faces(
    the_args: &List<Shape>,
    the_result: &Shape,
) -> Result<(MapOfShape, bool), Failure> {
    let mut all_faces = MapOfShape::default();
    for arg in the_args.iter() {
        if arg.is_same(the_result) {
            return Ok((all_faces, true));
        }
        let mut t_ex = Explorer::new(arg, ShapeEnum::Face, ShapeEnum::Shape)?;
        while t_ex.more() {
            all_faces.add(t_ex.current().clone());
            t_ex.next()?;
        }
    }
    Ok((all_faces, false))
}

/// The compound of the faces of the result that are not faces of the arguments
/// (updated), `None` without any.
fn new_faces(the_result: &Shape, all_faces: &MapOfShape) -> Result<Option<Compound>, Failure> {
    let b_b = BRepBuilder::default();
    let mut to_check: Option<Compound> = None;
    let mut t_ex = Explorer::new(the_result, ShapeEnum::Face, ShapeEnum::Shape)?;
    while t_ex.more() {
        let curf = topods::face(t_ex.current())?;
        t_ex.next()?;
        if all_faces.contains(curf.as_ref()) {
            continue;
        }
        let compound = to_check.get_or_insert_with(|| {
            let mut c = Compound::default();
            b_b.make_compound(&mut c);
            c
        });
        brep_tools::update(curf.as_ref())?;
        b_b.add(compound, &curf)?;
    }
    Ok(to_check)
}

/// The edges of a faulty face whose only problem is a same-parameter or same-range
/// flag: the flags reset and the edge made same-parameter again.
fn force_same_parameter(ana: &BRepCheckAnalyzer, face: &Shape) -> Result<(), Failure> {
    let b_b = BRepBuilder::default();
    let mut exp = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while exp.more() {
        let edge_shape = exp.current().clone();
        exp.next()?;
        let res = ana.result(&edge_shape)?;
        let r = res.get().ok_or_else(null_result)?;
        r.init_context_iterator()?;
        while r.more_shape_in_context() {
            if r.contextual_shape()?.is_same(face) {
                let flagged = r.status_on_shape()?.iter().any(|s| {
                    *s == BRepCheckStatus::InvalidSameParameterFlag
                        || *s == BRepCheckStatus::InvalidSameRangeFlag
                });
                if flagged {
                    let edge = topods::edge(&edge_shape)?;
                    b_b.same_range(&edge, false)?;
                    b_b.same_parameter(&edge, false)?;
                    brep_lib::same_parameter_edge(&edge, BRepTool::edge_tolerance(&edge)?)?;
                }
            }
            r.next_shape_in_context();
        }
    }
    Ok(())
}

/// The new faces checked: when invalid, the same-parameter problems of the first
/// faulty status of each face forced, then checked again.
fn check_new_faces(to_check: &Compound) -> Result<bool, Failure> {
    let mut ana = BRepCheckAnalyzer::new(to_check.as_ref(), true, false, false)?;
    if ana.is_valid()? {
        return Ok(true);
    }
    let mut t_ex = Explorer::new(to_check.as_ref(), ShapeEnum::Face, ShapeEnum::Shape)?;
    while t_ex.more() {
        let face = t_ex.current().clone();
        t_ex.next()?;
        let result = ana.result(&face)?;
        let Some(r) = result.get() else { continue };
        // Only the first incorrect status of the face is processed.
        if r.status()?.iter().any(|s| *s != BRepCheckStatus::NoError) {
            force_same_parameter(&ana, &face)?;
        }
    }
    // Remake the control: there can be a problem of another type, or one that cannot
    // be corrected.
    ana.init(to_check.as_ref(), true)?;
    ana.is_valid()
}

/// The shells of the result: valid, oriented and (for a closed solid) closed.
fn check_shells(the_result: &Shape, closed_solid: bool) -> Result<bool, Failure> {
    let mut hr: Option<BRepCheckShell> = None;
    let mut t_ex = Explorer::new(the_result, ShapeEnum::Shell, ShapeEnum::Shape)?;
    while t_ex.more() {
        let shell = topods::shell(t_ex.current())?;
        t_ex.next()?;
        match &hr {
            None => hr = Some(BRepCheckShell::new(&shell)?),
            Some(h) => h.init(shell.as_ref())?,
        }
        let h = hr.as_ref().ok_or_else(null_result)?;
        if *h.status()?.first().ok_or_else(null_result)? != BRepCheckStatus::NoError {
            return Ok(false);
        }
        if h.orientation(false)? != BRepCheckStatus::NoError {
            return Ok(false);
        }
        if closed_solid && h.closed(false)? != BRepCheckStatus::NoError {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `BRepAlgo::IsValid(theArgs, theResult, closedSolid, GeomCtrl)` (natively not
/// closed, with geometric controls): a null result is valid; the result alone without
/// argument face (unless it is an argument), else its new faces; then its shells;
/// `brep_algo::is_valid_result(&args, &result, false, true)?`.
pub fn is_valid_result(
    the_args: &List<Shape>,
    the_result: &Shape,
    closed_solid: bool,
    geom_ctrl: bool,
) -> Result<bool, Failure> {
    if the_result.is_null() {
        return Ok(true);
    }
    let (all_faces, validate) = argument_faces(the_args, the_result)?;
    if all_faces.is_empty() {
        if validate {
            return Ok(true);
        }
        if !BRepCheckAnalyzer::new(the_result, geom_ctrl, false, false)?.is_valid()? {
            return Ok(false);
        }
    } else if !validate
        && let Some(to_check) = new_faces(the_result, &all_faces)?
        && !check_new_faces(&to_check)?
    {
        return Ok(false);
    }
    check_shells(the_result, closed_solid)
}

/// Standard_NullObject of a missing check result (natively a crash).
fn null_result() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepAlgo::IsValid: null check result; expected a result",
        "BRepCheck_Analyzer",
    )
}
