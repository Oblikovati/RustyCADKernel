// Created on: 1994-12-06
// Created by: Jacques GOUSSARD
// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of BRepTopAdaptor_HVertex.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTopAdaptor_HVertex`: a vertex of a face edge as the `Adaptor3d_HVertex` that
//! `BRepTopAdaptor_TopolTool::Vertex` hands out.
use super::errors::null_object;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::real::real_first;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::{Adaptor3dHVertex, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

/// `BRepTopAdaptor_HVertex` (docs/topalgo-design.md section 4): a TopoDS vertex and
/// the pcurve adaptor it was found on. Its parent field is a default
/// `Adaptor3dHVertex` (the RTTI chain); the overrides are inherent methods.
///
/// ```
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorHVertex;
/// let vertex = BRepTopAdaptorHVertex::new(&Vertex::default(), &Handle::default());
/// assert!(vertex.vertex().is_null());
/// ```
pub struct BRepTopAdaptorHVertex {
    parent: Adaptor3dHVertex,
    my_vtx: Vertex,
    // Kept as natively, although no member reads it.
    _my_curve: Handle<BRepAdaptorCurve2d>,
}
impl_transient!(
    BRepTopAdaptorHVertex,
    "BRepTopAdaptor_HVertex",
    Adaptor3dHVertex,
    parent
);

/// The first-order data `Resolution` samples: the curve, the surface and the vertex.
struct ResolutionContext<'a> {
    curve: &'a dyn Adaptor2dCurve2d,
    surface: BRepAdaptorSurface,
    tv: f64,
    p: f64,
    point: Point3,
    reversed: bool,
    bounds: (f64, f64),
}

impl BRepTopAdaptorHVertex {
    /// `BRepTopAdaptor_HVertex(Vtx, Curve)`;
    /// `BRepTopAdaptorHVertex::new(&vertex, &curve)`.
    pub fn new(vertex: &Vertex, curve: &Handle<BRepAdaptorCurve2d>) -> Self {
        Self {
            parent: Adaptor3dHVertex::default(),
            my_vtx: vertex.clone(),
            _my_curve: curve.clone(),
        }
    }

    /// `Vertex()`; `hvertex.vertex()`.
    pub fn vertex(&self) -> &Vertex {
        &self.my_vtx
    }

    /// `ChangeVertex()`; `*hvertex.change_vertex() = other;`.
    pub fn change_vertex(&mut self) -> &mut Vertex {
        &mut self.my_vtx
    }

    /// `Value()`: always `(RealFirst, RealFirst)` (the native body does nothing);
    /// `hvertex.value()`.
    pub fn value(&self) -> Point2 {
        Point2::new(real_first(), real_first())
    }

    /// `Parameter(C)`: the parameter of the vertex on the edge of `curve` on its face;
    /// Standard_NullObject when `curve` is not a BRepAdaptor_Curve2d (a null
    /// dereference natively); `hvertex.parameter(&curve)?`.
    pub fn parameter(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure> {
        let brhc = brep_curve(curve, "BRepTopAdaptor_HVertex::Parameter")?;
        let pcurve = brhc.get().expect("checked BRepAdaptor_Curve2d");
        BRepTool::parameter_on_face(&self.my_vtx, &pcurve.edge(), &pcurve.face())
    }

    /// `Resolution(C)`: the parametric resolution of the vertex tolerance on `curve`,
    /// refined by two chord estimates on the face surface (at most
    /// `max(C.Resolution(URes), C.Resolution(VRes))`); Standard_NullObject when `curve`
    /// is not a BRepAdaptor_Curve2d; `hvertex.resolution(&curve)?`.
    pub fn resolution(&self, curve: &Handle<dyn Adaptor2dCurve2d>) -> Result<f64, Failure> {
        let brhc = brep_curve(curve, "BRepTopAdaptor_HVertex::Resolution")?;
        let pcurve = brhc.get().expect("checked BRepAdaptor_Curve2d");
        let surface = BRepAdaptorSurface::with_face(&pcurve.face(), false)?;
        let tv: f64 = BRepTool::vertex_tolerance(&self.my_vtx)?;
        let p: f64 = BRepTool::parameter_on_face(&self.my_vtx, &pcurve.edge(), &pcurve.face())?;
        let reversed: bool = self.orientation() == Orientation::Reversed;
        let c: &dyn Adaptor2dCurve2d = curve.get().expect("checked curve");
        let (p2d, v2d) = c.d1(p)?;
        let d1 = surface.d1(p2d.x(), p2d.y())?;
        let mag: f64 = linear_form(v2d.x(), d1.d1u, v2d.y(), d1.d1v).magnitude();
        let t_u_res: f64 = c.resolution(surface.u_resolution(tv)?)?;
        let t_v_res: f64 = c.resolution(surface.v_resolution(tv)?)?;
        let res_uv1: f64 = std_max(t_u_res, t_v_res);
        if mag < 1e-12 {
            return Ok(res_uv1);
        }
        let bounds = (c.first_parameter()?, c.last_parameter()?);
        let context = ResolutionContext {
            curve: c,
            surface,
            tv,
            p,
            point: d1.point,
            reversed,
            bounds,
        };
        Ok(std_min(context.refined(mag)?, res_uv1))
    }

    /// `Orientation()`: the orientation of the vertex; `hvertex.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.my_vtx.orientation()
    }

    /// `IsSame(Other)`: whether the vertex of `other` is the same TopoDS vertex;
    /// Standard_NullObject when `other` is not a BRepTopAdaptor_HVertex (a null
    /// dereference natively); `hvertex.is_same(&other)?`.
    pub fn is_same(&self, other: &Handle<Adaptor3dHVertex>) -> Result<bool, Failure> {
        let brhv: Handle<BRepTopAdaptorHVertex> = other.cast();
        let Some(other) = brhv.get() else {
            return Err(null_object(
                "BRepTopAdaptor_HVertex::IsSame: the other vertex is not a BRepTopAdaptor_HVertex; expected one",
            ));
        };
        Ok(self.my_vtx.is_same(&other.my_vtx))
    }
}

impl ResolutionContext<'_> {
    // The parameter `p -+ step` (towards the inside of the edge) clamped to the range.
    fn stepped(&self, step: f64) -> f64 {
        let mut pp: f64 = if self.reversed {
            self.p + step
        } else {
            self.p - step
        };
        if pp > self.bounds.1 {
            pp = self.bounds.1;
        }
        if pp < self.bounds.0 {
            pp = self.bounds.0;
        }
        pp
    }

    // The distance from the vertex point to the surface point of the curve at `pp`.
    fn distance_at(&self, pp: f64) -> Result<f64, Failure> {
        let p2d: Point2 = self.curve.d0(pp)?;
        let p1: Point3 = self.surface.d0(p2d.x(), p2d.y())?;
        Ok(self.point.distance(&p1))
    }

    // The body of `Resolution` after the tangent magnitude check: the first estimate
    // `tv / mag`, then the two refinements when the chord misses the tolerance.
    fn refined(&self, mag: f64) -> Result<f64, Failure> {
        let tv: f64 = self.tv;
        // for lack of better options limit the parametric solution to
        // 10 million*tolerance of the point
        let mut res_uv: f64 = if tv > 1.0e7 * mag { 1.0e7 } else { tv / mag };
        let mut dist: f64 = self.distance_at(self.stepped(res_uv))?;
        if !((dist > 1e-12) && ((dist > 1.1 * tv) || (dist < 0.8 * tv))) {
            return Ok(res_uv);
        }
        // Refine if possible
        let pp: f64 = self.stepped(tv / dist);
        let (p2d, v2d) = self.curve.d1(pp)?;
        let d1 = self.surface.d1(p2d.x(), p2d.y())?;
        let dc: Vector3 = linear_form(v2d.x(), d1.d1u, v2d.y(), d1.d1v);
        let dist1: f64 = self.point.distance(&d1.point);
        if (dist1 - tv).abs() < (dist - tv).abs() {
            // Take the result of interpolation
            res_uv = tv / dist;
            dist = dist1;
        }
        let mut mag: f64 = dc.magnitude();
        if tv > 1.0e7 * mag {
            mag = tv * 1.0e-7;
        }
        let dist1: f64 = self.distance_at(self.stepped(tv / mag))?;
        if (dist1 - tv).abs() < (dist - tv).abs() {
            // Take the new estimation
            res_uv = tv / mag;
        }
        Ok(res_uv)
    }
}

// `DC.SetLinearForm(a, DU, b, DV)`.
fn linear_form(a: f64, du: Vector3, b: f64, dv: Vector3) -> Vector3 {
    let mut dc = Vector3::default();
    dc.set_linear_form(a, du, b, dv);
    dc
}

// `occ::down_cast<BRepAdaptor_Curve2d>(C)`, where the native members dereference it.
fn brep_curve(
    curve: &Handle<dyn Adaptor2dCurve2d>,
    member: &str,
) -> Result<Handle<BRepAdaptorCurve2d>, Failure> {
    let brhc: Handle<BRepAdaptorCurve2d> = curve.cast();
    if brhc.is_null() {
        return Err(null_object(&format!(
            "{member}: the curve is not a BRepAdaptor_Curve2d; expected one"
        )));
    }
    Ok(brhc)
}
