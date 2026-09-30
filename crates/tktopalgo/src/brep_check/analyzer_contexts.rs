// Rust translation of BRepCheck_ParallelAnalyzer (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Analyzer.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_ParallelAnalyzer`: the checks of each shape of the analyzer in its
//! context. A failure of a check is caught and recorded as `CheckFail`, as the native
//! `catch (Standard_Failure)` blocks do; a missing checker leaves the analysis.
use super::analyzer::CheckerMap;
use super::edge::BRepCheckEdge;
use super::explore::sub_shapes;
use super::face::BRepCheckFace;
use super::result::BRepCheckResult;
use super::status::BRepCheckStatus;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::osd::Parallel;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Mutex;

/// The checker of `shape` in the map (Standard_NoSuchObject when it is missing).
fn checker_of(map: &CheckerMap, shape: &Shape) -> Result<Handle<dyn BRepCheckResult>, Failure> {
    map.find_from_key(shape).cloned()
}

/// `InContext(context)` on the checker of `sub`; a failure marks the context shape's
/// checker and the sub-shape's checker (on both shapes). True when it succeeded.
fn sub_in_context(
    map: &CheckerMap,
    sub: &Shape,
    context: &Shape,
    context_checker: &Handle<dyn BRepCheckResult>,
) -> Result<bool, Failure> {
    let checker = checker_of(map, sub)?;
    let outcome = match checker.get() {
        Some(c) => c.in_context(context),
        None => Err(super::errors::null_object(
            "BRepCheck_Analyzer: null checker",
        )),
    };
    if outcome.is_err() {
        mark_failed(context_checker, context);
        if let Some(c) = checker.get() {
            c.set_fail_status(sub);
            c.set_fail_status(context);
        }
    }
    Ok(outcome.is_ok())
}

fn mark_failed(checker: &Handle<dyn BRepCheckResult>, shape: &Shape) {
    if let Some(c) = checker.get() {
        c.set_fail_status(shape);
    }
}

/// The distinct (IsSame) sub-shapes of `kind` of `shape`, each in context of `shape`.
fn subs_in_context(
    map: &CheckerMap,
    shape: &Shape,
    kind: ShapeEnum,
    checker: &Handle<dyn BRepCheckResult>,
) -> Result<(), Failure> {
    let mut seen = MapOfShape::default();
    for sub in sub_shapes(shape, kind)? {
        if seen.add(sub.clone()) {
            sub_in_context(map, &sub, shape, checker)?;
        }
    }
    Ok(())
}

/// An edge: its polygons on triangulations, then its vertices in its context.
fn check_edge(
    map: &CheckerMap,
    shape: &Shape,
    checker: &Handle<dyn BRepCheckResult>,
) -> Result<(), Failure> {
    let polygon = (|| -> Result<(), Failure> {
        let edge_checker = checker.cast::<BRepCheckEdge>();
        let e = edge_checker
            .get()
            .ok_or_else(|| super::errors::null_object("BRepCheck_Analyzer: not an edge checker"))?;
        let status = e.check_polygon_on_triangulation(&Edge::unchecked(shape.clone()))?;
        if status != BRepCheckStatus::NoError {
            e.set_status(status)?;
        }
        Ok(())
    })();
    if polygon.is_err() {
        mark_failed(checker, shape);
    }
    subs_in_context(map, shape, ShapeEnum::Vertex, checker)
}

/// Whether the edge statuses in the face forbid the wire checks.
fn blocks_wires(statuses: &[BRepCheckStatus]) -> bool {
    statuses.iter().any(|s| {
        matches!(
            s,
            BRepCheckStatus::NoCurveOnSurface
                | BRepCheckStatus::InvalidCurveOnSurface
                | BRepCheckStatus::InvalidRange
                | BRepCheckStatus::InvalidCurveOnClosedSurface
        )
    })
}

/// The statuses of `sub`'s checker in the context `shape`, if any.
fn statuses_in(
    map: &CheckerMap,
    sub: &Shape,
    shape: &Shape,
) -> Result<Vec<BRepCheckStatus>, Failure> {
    let checker = checker_of(map, sub)?;
    let Some(c) = checker.get() else {
        return Ok(Vec::new());
    };
    if !c.is_status_on_shape(shape) {
        return Ok(Vec::new());
    }
    Ok(c.status_on_shape_of(shape)?.iter().copied().collect())
}

/// A face: its vertices, edges and wires in its context, then the orientation of its
/// wires when no edge or wire check failed there, else `UnorientableShape`.
fn check_face(
    map: &CheckerMap,
    shape: &Shape,
    checker: &Handle<dyn BRepCheckResult>,
) -> Result<(), Failure> {
    subs_in_context(map, shape, ShapeEnum::Vertex, checker)?;
    let mut perform_wire = true;
    let mut seen = MapOfShape::default();
    for sub in sub_shapes(shape, ShapeEnum::Edge)? {
        if !seen.add(sub.clone()) {
            continue;
        }
        let checked = sub_in_context(map, &sub, shape, checker)?;
        if checked && perform_wire && blocks_wires(&statuses_in(map, &sub, shape)?) {
            perform_wire = false;
        }
    }
    let mut orient_wires = perform_wire;
    let wires = sub_shapes(shape, ShapeEnum::Wire)?;
    for wire in &wires {
        let checked = sub_in_context(map, wire, shape, checker)?;
        if checked
            && orient_wires
            && statuses_in(map, wire, shape)?
                .iter()
                .any(|s| *s != BRepCheckStatus::NoError)
        {
            orient_wires = false;
        }
    }
    if face_orientation(checker, perform_wire && orient_wires).is_err() {
        mark_failed(checker, shape);
        for wire in &wires {
            let wire_checker = checker_of(map, wire)?;
            if let Some(c) = wire_checker.get() {
                c.set_fail_status(wire);
                c.set_fail_status(shape);
                mark_failed(checker, wire);
            }
        }
    }
    Ok(())
}

/// The face's `OrientationOfWires(true)`, or `SetUnorientable()`.
fn face_orientation(checker: &Handle<dyn BRepCheckResult>, orient: bool) -> Result<(), Failure> {
    let face_checker = checker.cast::<BRepCheckFace>();
    let f = face_checker
        .get()
        .ok_or_else(|| super::errors::null_object("BRepCheck_Analyzer: not a face checker"))?;
    if orient {
        f.orientation_of_wires(true)?;
        return Ok(());
    }
    f.set_unorientable()
}

/// The checks of one shape by its type (vertices, wires, shells and compounds have none).
fn check_shape(map: &CheckerMap, shape: &Shape) -> Result<(), Failure> {
    let checker = checker_of(map, shape)?;
    match shape.shape_type()? {
        ShapeEnum::Edge => check_edge(map, shape, &checker),
        ShapeEnum::Face => check_face(map, shape, &checker),
        ShapeEnum::Solid => {
            for sub in sub_shapes(shape, ShapeEnum::Shell)? {
                sub_in_context(map, &sub, shape, &checker)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// `OSD_Parallel::For` over the tasks: in order without the parallel flag, else on the
/// default pool; the first failure outside the catch blocks is returned.
pub(super) fn check_in_contexts(
    arrays: &[Vec<Shape>],
    map: &CheckerMap,
    parallel: bool,
) -> Result<(), Failure> {
    let failure: Mutex<Option<Failure>> = Mutex::new(None);
    let task = |index: i32| {
        for shape in &arrays[index as usize] {
            if let Err(error) = check_shape(map, shape) {
                failure
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get_or_insert(error);
                return;
            }
        }
    };
    if parallel {
        Parallel::for_range(0, arrays.len() as i32, task, false)?;
    } else {
        for index in 0..arrays.len() as i32 {
            task(index);
            if failure.lock().unwrap_or_else(|e| e.into_inner()).is_some() {
                break;
            }
        }
    }
    match failure.into_inner().unwrap_or_else(|e| e.into_inner()) {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
