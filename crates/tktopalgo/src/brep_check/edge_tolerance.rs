// Created on: 1995-12-11
// Created by: Jacques GOUSSARD
// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of BRepCheck_Edge::Tolerance and CheckPolygonOnTriangulation (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Edge.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Edge::Tolerance` (the spread of the edge representations) and
//! `CheckPolygonOnTriangulation`.
use super::edge::{BRepCheckEdge, curve_on_surface};
use super::errors::{null_object, out_of_range};
use super::status::BRepCheckStatus;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point3, Transform3, TransformForm};
use rustycad_tkmath::poly::{PolygonOnTriangulation, Triangulation};

/// `NCONTROL`: the number of control points of `Tolerance` and of the polygon check.
const NCONTROL: i32 = 23;

/// The `theRep` array of `Tolerance()`: the adaptors of the representations, the 3d
/// curve first (1-based natively).
struct Representations {
    items: Vec<Handle<dyn Adaptor3dCurve>>,
    count: i32,
}

impl Representations {
    fn get(&self, index: i32) -> Result<&dyn Adaptor3dCurve, Failure> {
        usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.items.get(i))
            .and_then(|h| h.get())
            .ok_or_else(|| {
                null_object(&format!(
                    "BRepCheck_Edge::Tolerance: no representation {index} of {}",
                    self.count
                ))
            })
    }

    fn set(&mut self, index: i32, value: Handle<dyn Adaptor3dCurve>) -> Result<(), Failure> {
        let size = self.items.len();
        let slot = usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.items.get_mut(i))
            .ok_or_else(|| {
                out_of_range(&format!(
                    "BRepCheck_Edge::Tolerance: representation {index} beyond {size}"
                ))
            })?;
        *slot = value;
        Ok(())
    }
}

impl BRepCheckEdge {
    /// `Tolerance()`: 1.05 times the largest distance, at 23 parameters of the
    /// reference range, between the 3d curve and the curves on surface of the edge;
    /// Precision::Confusion() for a single representation, Precision::Infinite() for an
    /// infinite point. The second pcurve of a closed surface is moved by the
    /// representation location only, as natively; `checker.tolerance()?`.
    pub fn tolerance(&self) -> Result<f64, Failure> {
        let shape = self.edge();
        let tedge = edge_tshape(&shape, "BRepCheck_Edge::Tolerance")?;
        let curves = tedge.curves();
        let nb: i32 = i32::try_from(curves.len()).unwrap_or(i32::MAX);
        if nb <= 1 {
            return Ok(precision::confusion());
        }
        let range = self.tolerance_range(&shape)?;
        let mut reps = Representations {
            items: vec![Handle::default(); 2 * curves.len()],
            count: nb,
        };
        let mut next: i32 = 1;
        for cr in &curves {
            let Some(r) = cr.get() else {
                reps.count -= 1;
                continue;
            };
            if r.is_curve3d() && !tedge.degenerated() {
                let adaptor = self.moved_curve3d(r, range)?;
                let mut slot = next;
                if next > 1 {
                    let first = reps.items[0].clone();
                    reps.set(next, first)?;
                    slot = 1;
                }
                reps.set(slot, adaptor)?;
                next += 1;
            } else if r.is_curve_on_surface() {
                next = self.add_curves_on_surface(r, range, &mut reps, next)?;
            } else {
                reps.count -= 1;
            }
        }
        spread_of(&reps, range)
    }

    /// The range of `Tolerance()`: the reference adaptor's, else the edge's.
    fn tolerance_range(&self, shape: &Edge) -> Result<(f64, f64), Failure> {
        let hcurve = self.fields.read().hcurve.clone();
        match hcurve.get() {
            Some(curve) => Ok((curve.first_parameter()?, curve.last_parameter()?)),
            None => BRepTool::range(shape),
        }
    }

    /// The 3d curve moved by the edge location, on `range`.
    fn moved_curve3d(
        &self,
        r: &dyn BRepCurveRepresentation,
        (first, last): (f64, f64),
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let location = (self.edge().location() * &r.location())?;
        let c3d = r
            .curve3d()?
            .get()
            .ok_or_else(|| null_object("BRepCheck_Edge::Tolerance: null 3d curve"))?
            .transformed(location.transformation())?
            .cast::<dyn GeomCurve>();
        Ok(Handle::new(GeomAdaptorCurve::with_range(&c3d, first, last)?).cast())
    }

    /// The curve on surface (and the second one of a closed surface) at `next`;
    /// returns the next free index.
    fn add_curves_on_surface(
        &self,
        r: &dyn BRepCurveRepresentation,
        range: (f64, f64),
        reps: &mut Representations,
        mut next: i32,
    ) -> Result<i32, Failure> {
        let location = (self.edge().location() * &r.location())?;
        let sref = moved_surface(&r.surface()?, location.transformation())?;
        reps.set(next, curve_on_surface(&r.pcurve()?, range, &sref)?.cast())?;
        next += 1;
        if r.is_curve_on_closed_surface() {
            let sref2 = moved_surface(&r.surface()?, r.location().transformation())?;
            reps.set(next, curve_on_surface(&r.pcurve2()?, range, &sref2)?.cast())?;
            next += 1;
            reps.count += 1;
        }
        Ok(next)
    }

    /// `CheckPolygonOnTriangulation(theEdge)`: for an edge with a 3d curve and polygons
    /// on triangulations, `InvalidPolygonOnTriangulation` when a polygon node is beyond
    /// the polygon deflection plus the edge tolerance from the curve (at its parameter),
    /// or, without parameters, when a curve point is out of the enlarged box of the
    /// nodes. A polygon on a closed triangulation checks its second polygon, as
    /// natively; `checker.check_polygon_on_triangulation(&edge)?`.
    pub fn check_polygon_on_triangulation(&self, edge: &Edge) -> Result<BRepCheckStatus, Failure> {
        let curves = edge_tshape(edge, "BRepCheck_Edge::CheckPolygonOnTriangulation")?.curves();
        if !has_polygon_and_curve3d(&curves)? {
            return Ok(BRepCheckStatus::NoError);
        }
        let adaptor = BRepAdaptorCurve::with_edge(edge)?;
        if !adaptor.transformed_curve().is_3d_curve() {
            return Ok(BRepCheckStatus::NoError);
        }
        for cr in &curves {
            let Some(r) = cr.get().filter(|r| r.is_polygon_on_triangulation()) else {
                continue;
            };
            if !polygon_fits(edge, r, &adaptor)? {
                return Ok(BRepCheckStatus::InvalidPolygonOnTriangulation);
            }
        }
        Ok(BRepCheckStatus::NoError)
    }
}

/// The surface of a representation moved by `transform`.
fn moved_surface(
    surface: &Handle<dyn GeomSurface>,
    transform: Transform3,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(surface
        .get()
        .ok_or_else(|| null_object("BRepCheck_Edge::Tolerance: null surface"))?
        .transformed(transform)?
        .cast::<dyn GeomSurface>())
}

/// The largest distance from the first representation to the others at the control
/// parameters, times 1.05 (Precision::Infinite() at an infinite point).
fn spread_of(reps: &Representations, (first, last): (f64, f64)) -> Result<f64, Failure> {
    let mut tol_cal: f64 = 0.0;
    for i in 0..NCONTROL {
        let prm: f64 =
            (f64::from(NCONTROL - 1 - i) * first + f64::from(i) * last) / f64::from(NCONTROL - 1);
        let center: Point3 = reps.get(1)?.value(prm)?;
        if is_infinite_point(&center) {
            return Ok(precision::infinite());
        }
        for index in 2..=reps.count {
            let other: Point3 = reps.get(index)?.value(prm)?;
            if is_infinite_point(&other) {
                return Ok(precision::infinite());
            }
            let dist2: f64 = center.square_distance(&other);
            if dist2 > tol_cal {
                tol_cal = dist2;
            }
        }
    }
    // "On prend 5% de marge car au dessus on crontrole severement".
    Ok(tol_cal.sqrt() * 1.05)
}

fn is_infinite_point(point: &Point3) -> bool {
    precision::is_infinite(point.x())
        || precision::is_infinite(point.y())
        || precision::is_infinite(point.z())
}

/// Whether the edge has a polygon on triangulation and a non-null 3d curve.
fn has_polygon_and_curve3d(
    curves: &[Handle<dyn BRepCurveRepresentation>],
) -> Result<bool, Failure> {
    let (mut polygon, mut curve3d) = (false, false);
    for cr in curves {
        let Some(r) = cr.get() else {
            continue;
        };
        if r.is_polygon_on_triangulation() {
            polygon = true;
        }
        if r.is_curve3d() && r.curve3d()?.is_some() {
            curve3d = true;
        }
        if polygon && curve3d {
            break;
        }
    }
    Ok(polygon && curve3d)
}

/// The check of one polygon on triangulation against the edge curve.
fn polygon_fits(
    edge: &Edge,
    r: &dyn BRepCurveRepresentation,
    adaptor: &BRepAdaptorCurve,
) -> Result<bool, Failure> {
    let transform: Transform3 = (edge.location() * &r.location())?.transformation();
    let triangulation = r.triangulation()?;
    let polygon = if r.is_polygon_on_closed_triangulation() {
        r.polygon_on_triangulation2()?
    } else {
        r.polygon_on_triangulation()?
    };
    let polygon = polygon
        .get()
        .ok_or_else(|| null_object("BRepCheck_Edge: null polygon on triangulation"))?;
    let triangulation = triangulation
        .get()
        .ok_or_else(|| null_object("BRepCheck_Edge: null triangulation"))?;
    let tol: f64 = polygon.deflection() + BRepTool::edge_tolerance(edge)?;
    if polygon.has_parameters() {
        return nodes_near_curve(polygon, triangulation, transform, adaptor, tol);
    }
    curve_in_node_box(polygon, triangulation, transform, adaptor, tol)
}

/// The polygon with parameters: every node within `tol` of the curve point.
fn nodes_near_curve(
    polygon: &PolygonOnTriangulation,
    triangulation: &Triangulation,
    transform: Transform3,
    adaptor: &BRepAdaptorCurve,
    tol: f64,
) -> Result<bool, Failure> {
    let parameters = polygon.parameters();
    let parameters = parameters
        .get()
        .ok_or_else(|| null_object("BRepCheck_Edge: null polygon parameters"))?;
    let indices = polygon.nodes();
    for i in parameters.lower()..=parameters.upper() {
        let param: f64 = parameters.value(i)?;
        let on_edge: Point3 = adaptor.value(param)?;
        let index: i32 = *indices
            .value(i)
            .map_err(|e| out_of_range(&format!("{e:?}")))?;
        let node: Point3 = triangulation.node(index)?.transformed(transform);
        if on_edge.square_distance(&node) > tol * tol {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The polygon without parameters: the curve points at 23 steps and its end inside the
/// box of the nodes enlarged by `tol`.
fn curve_in_node_box(
    polygon: &PolygonOnTriangulation,
    triangulation: &Triangulation,
    transform: Transform3,
    adaptor: &BRepAdaptorCurve,
    tol: f64,
) -> Result<bool, Failure> {
    let indices = polygon.nodes();
    let mut bounds = BoundingBox3::default();
    for i in 1..=indices.length() {
        let index: i32 = *indices
            .value(i)
            .map_err(|e| out_of_range(&format!("{e:?}")))?;
        let node: Point3 = triangulation.node(index)?;
        if transform.form() == TransformForm::Identity {
            bounds.add_point(node);
        } else {
            bounds.add_point(node.transformed(transform));
        }
    }
    bounds.enlarge(tol);
    let (fp, lp) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
    let step: f64 = (lp - fp) / f64::from(NCONTROL);
    let mut par: f64 = fp;
    for _ in 1..NCONTROL {
        if bounds.is_out_point(adaptor.d0(par)?) {
            return Ok(false);
        }
        par += step;
    }
    Ok(!bounds.is_out_point(adaptor.d0(lp)?))
}
