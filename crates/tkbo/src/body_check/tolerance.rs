// SPDX-License-Identifier: GPL-2.0-only
//! Tolerance consistency, which no mapped checker covers (OCCT never emits
//! `InvalidToleranceValue`): valid values, vertex >= edge >= face nesting, the caller's
//! cap, and the certified planar curve/p-curve residual of ADR 0014 where it applies.
use super::census::children;
use super::naming::TopologyNames;
use super::report::{CheckMode, Finding, FindingKind, Severity, TopologyRef};
use crate::sketch_region_validation::inspect_planar_edge_residual;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

/// Root isolation work policy of the residual certificates (an approximation width,
/// not a modelling tolerance).
const RESIDUAL_WIDTH: f64 = 0.01;

/// The tolerance findings of every indexed entity and whether all residuals certified.
/// Example: `let (findings, mode) = tolerance_findings(&names, Some(1e-3))?;`.
pub(super) fn tolerance_findings(
    names: &TopologyNames,
    budget: Option<f64>,
) -> Result<(Vec<Finding>, CheckMode), Failure> {
    let mut findings = Vec::new();
    for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
        for shape in names.all(kind) {
            value_findings(&shape, names.name(&shape), budget, &mut findings)?;
        }
    }
    nesting_findings(names, &mut findings)?;
    let mode = residual_findings(names, &mut findings)?;
    Ok((findings, mode))
}

/// The stored tolerance of a vertex, edge or face; `stored_tolerance(&edge)?`.
fn stored_tolerance(shape: &Shape) -> Result<f64, Failure> {
    match shape.shape_type()? {
        ShapeEnum::Vertex => BRepTool::vertex_tolerance(&topods::vertex(shape)?),
        ShapeEnum::Edge => BRepTool::edge_tolerance(&topods::edge(shape)?),
        _ => BRepTool::face_tolerance(&topods::face(shape)?),
    }
}

fn value_findings(
    shape: &Shape,
    name: TopologyRef,
    budget: Option<f64>,
    findings: &mut Vec<Finding>,
) -> Result<(), Failure> {
    let tolerance = stored_tolerance(shape)?;
    if !tolerance.is_finite() || tolerance < 0.0 {
        findings.push(defect(
            FindingKind::InvalidTolerance { tolerance },
            name,
            None,
        ));
        return Ok(());
    }
    if let Some(budget) = budget.filter(|budget| tolerance > *budget) {
        let kind = FindingKind::ToleranceAboveBudget { tolerance, budget };
        findings.push(defect(kind, name, None));
    }
    Ok(())
}

/// Each edge within its vertices' tolerance, each face within its edges'.
fn nesting_findings(names: &TopologyNames, findings: &mut Vec<Finding>) -> Result<(), Failure> {
    for (inner, outer) in [
        (ShapeEnum::Edge, ShapeEnum::Vertex),
        (ShapeEnum::Face, ShapeEnum::Edge),
    ] {
        for shape in names.all(inner) {
            let tolerance = stored_tolerance(&shape)?;
            for bound in children(&shape, outer, ShapeEnum::Shape)? {
                let bounding = stored_tolerance(&bound)?;
                if bounding < tolerance {
                    let kind = FindingKind::ToleranceOrder {
                        inner: tolerance,
                        outer: bounding,
                    };
                    findings.push(defect(kind, names.name(&shape), Some(names.name(&bound))));
                }
            }
        }
    }
    Ok(())
}

/// Certify `|curve - pcurve| <= edge tolerance` on planar faces; Exact when every edge
/// use of every face was decided exactly, Sampled otherwise (the geometry stage sampled it).
fn residual_findings(
    names: &TopologyNames,
    findings: &mut Vec<Finding>,
) -> Result<CheckMode, Failure> {
    let mut all_decided = true;
    for face in names.all(ShapeEnum::Face) {
        all_decided &= face_residuals(&topods::face(&face)?, names, findings)?;
    }
    Ok(match all_decided {
        true => CheckMode::Exact,
        false => CheckMode::Sampled,
    })
}

/// The residual findings of every edge use of `face`; whether all were decided exactly.
fn face_residuals(
    face: &Face,
    names: &TopologyNames,
    findings: &mut Vec<Finding>,
) -> Result<bool, Failure> {
    let planar = BRepAdaptorSurface::with_face(face, false)?.get_type()? == SurfaceType::Plane;
    let mut all_decided = true;
    for edge in children(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&edge)?;
        let residual = if planar {
            certify(&edge, face)?
        } else {
            Residual::Undecided
        };
        all_decided &= residual != Residual::Undecided;
        if let Residual::Exceeds(kind) = residual {
            findings.push(defect(kind, names.name(&edge), Some(names.name(face))));
        }
    }
    Ok(all_decided)
}

/// The exact residual decision of one planar edge use.
#[derive(Debug, PartialEq)]
enum Residual {
    Within,
    Exceeds(FindingKind),
    /// No certificate applies (unsupported curve kinds, degenerate edge, missing p-curve).
    Undecided,
}

fn certify(edge: &Edge, face: &Face) -> Result<Residual, Failure> {
    if BRepTool::degenerated(edge)? {
        return Ok(Residual::Undecided);
    }
    let tolerance = BRepTool::edge_tolerance(edge)?;
    let policy = RootIsolationOptions::new(RESIDUAL_WIDTH);
    let inspection = inspect_planar_edge_residual(edge, face, &[], tolerance, policy, || false);
    Ok(match inspection.decision {
        Ok(ResidualDecision::Within { .. }) => Residual::Within,
        Ok(ResidualDecision::Exceeds { .. }) => {
            Residual::Exceeds(FindingKind::CurveResidualExceedsTolerance { tolerance })
        }
        Err(_) => Residual::Undecided,
    })
}

fn defect(kind: FindingKind, subject: TopologyRef, context: Option<TopologyRef>) -> Finding {
    Finding {
        severity: Severity::Defect,
        kind,
        subject,
        context,
    }
}
