// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_SurfaceOfRevolution.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_SurfaceOfRevolution`: a basis curve rotated around an axis, U the
//! rotation angle and V the curve parameter.
use super::circle::GeomCircle;
use super::curve::GeomCurve;
use super::errors;
use super::geometry::GeomGeometry;
use super::parametric::v_affinity;
use super::revolution_utils;
use super::surface::{
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds, check_dn_orders,
};
use super::surface_of_linear_extrusion::copied_basis;
use super::swept_surface::{GeomSweptSurface, dump_swept_surface};
use crate::geom_eval::{RepSurfaceDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::cell::GeometryCell;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Direction3, GeneralTransform2, Line3, Point3, Transform3, Vector3, Xyz,
    resolution,
};
use std::f64::consts::PI;
use std::io;

struct RevolutionState {
    basis: Handle<dyn GeomCurve>,
    direction: Direction3,
    smooth: Shape,
    eval_rep: Handle<dyn RepSurfaceDesc>,
    loc: Point3,
}

/// A surface of revolution; the basis curve is a private copy;
/// `Handle::new(GeomSurfaceOfRevolution::new(&curve, axis)?)`.
pub struct GeomSurfaceOfRevolution {
    parent: StandardTransient,
    state: GeometryCell<RevolutionState>,
}
impl_transient!(
    GeomSurfaceOfRevolution,
    "Geom_SurfaceOfRevolution",
    dyn GeomSweptSurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomSweptSurface]
);

impl GeomSurfaceOfRevolution {
    /// Revolution of a copy of `curve` around `axis`; the continuity is the curve one
    /// (`Geom_SurfaceOfRevolution(C, A1)`); `GeomSurfaceOfRevolution::new(&curve, axis)?`.
    pub fn new(curve: &Handle<dyn GeomCurve>, axis: Axis1) -> Result<Self, Failure> {
        let (basis, smooth) = copied_basis(curve)?;
        Ok(Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(RevolutionState {
                basis,
                direction: axis.direction(),
                smooth,
                eval_rep: Handle::default(),
                loc: axis.location(),
            }),
        })
    }
    /// The native Copy: a new revolution of the basis (copied again) around `Axis()`
    /// with a deep copy of the evaluation representation;
    /// `GeomSurfaceOfRevolution::copy_of(&surface)?`.
    pub fn copy_of(other: &GeomSurfaceOfRevolution) -> Result<Self, Failure> {
        let (basis, axis, eval_rep) = {
            let source = other.state.read();
            let axis = Axis1::new(source.loc, source.direction);
            (source.basis.clone(), axis, source.eval_rep.clone())
        };
        let copy = Self::new(&basis, axis)?;
        copy.state.write().eval_rep = rep_utils::clone_surface_desc(&eval_rep)?;
        Ok(copy)
    }
    /// Whether an evaluation representation is set (`HasEvalRepresentation`);
    /// `surface.has_eval_representation()`.
    pub fn has_eval_representation(&self) -> bool {
        self.state.read().eval_rep.is_some()
    }
    /// The evaluation representation, possibly null (`EvalRepresentation`);
    /// `surface.eval_representation()`.
    pub fn eval_representation(&self) -> Handle<dyn RepSurfaceDesc> {
        self.state.read().eval_rep.clone()
    }
    /// Validate and set the evaluation representation (`SetEvalRepresentation`);
    /// `surface.set_eval_representation(desc)?`.
    pub fn set_eval_representation(&self, desc: Handle<dyn RepSurfaceDesc>) -> Result<(), Failure> {
        rep_utils::validate_surface_desc(&desc, std::ptr::from_ref(self).cast())?;
        self.state.write().eval_rep = desc;
        Ok(())
    }
    /// Remove the evaluation representation (`ClearEvalRepresentation`);
    /// `surface.clear_eval_representation()`.
    pub fn clear_eval_representation(&self) {
        self.state.write().eval_rep = Handle::default();
    }
    /// Replace the axis of revolution (`SetAxis`); `surface.set_axis(axis)`.
    pub fn set_axis(&self, axis: Axis1) {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.direction = axis.direction();
        state.loc = axis.location();
    }
    /// Replace the direction of the axis (`SetDirection`); `surface.set_direction(d)`.
    pub fn set_direction(&self, direction: Direction3) {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.direction = direction;
    }
    /// Replace the basis by a copy of `curve` (`SetBasisCurve`); `surface.set_basis_curve(&c)?`.
    pub fn set_basis_curve(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        self.clear_eval_representation();
        let (basis, smooth) = copied_basis(curve)?;
        let mut state = self.state.write();
        state.basis = basis;
        state.smooth = smooth;
        Ok(())
    }
    /// Replace the origin of the axis (`SetLocation`); `surface.set_location(p)`.
    pub fn set_location(&self, point: Point3) {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.loc = point;
    }
    /// The axis of revolution (`Axis`); `surface.axis()`.
    pub fn axis(&self) -> Axis1 {
        let state = self.state.read();
        Axis1::new(state.loc, state.direction)
    }
    /// The origin of the axis (`Location`); `surface.location()`.
    pub fn location(&self) -> Point3 {
        self.state.read().loc
    }
    /// Standard_NotImplemented, as natively (`ReferencePlane`); `surface.reference_plane()`.
    pub fn reference_plane(&self) -> Result<Axis2, Failure> {
        Err(errors::not_implemented(
            "Geom_SurfaceOfRevolution::ReferencePlane: not implemented in OCCT; expected no call",
        ))
    }
    fn parts(&self) -> (Handle<dyn GeomCurve>, Axis1) {
        let state = self.state.read();
        (state.basis.clone(), Axis1::new(state.loc, state.direction))
    }
}

// The frame of the V iso circle: centered on the projection of `pc` on the axis,
// with its X direction towards `pc`, or the automatic X for a point on the axis.
fn iso_frame(pc: Point3, loc: Point3, direction: Direction3, rad: f64) -> Result<Axis2, Failure> {
    if rad <= resolution() {
        return Axis2::new(pc, direction);
    }
    let mut p: Xyz = pc.xyz();
    let mut c = Xyz::default();
    c.set_scaled_sum(
        (p - loc.xyz()).dot(direction.xyz()),
        direction.xyz(),
        loc.xyz(),
    );
    p = p - c;
    if p.modulus() > resolution() {
        let d: Direction3 = Direction3::from_xyz(p.normalized()?)?;
        return Axis2::with_x_direction(Point3::from(c), direction, d);
    }
    Axis2::new(Point3::from(c), direction)
}

impl GeomGeometry for GeomSurfaceOfRevolution {
    /// Moves the axis and the basis; a transformation that changes the orientation
    /// (`ScaleFactor * det(HVectorialPart) < 0`) also reverses U.
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        {
            let mut state = self.state.write();
            state.eval_rep = Handle::default();
            state.loc.transform(transform);
            state.direction.transform(transform);
            state.basis.transform(transform)?;
        }
        if transform.scale_factor() * transform.matrix().determinant() < 0. {
            self.u_reverse()?;
        }
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom_SurfaceOfRevolution")?;
        dump_base(stream, depth, |s, d| {
            dump_swept_surface(s, d, &state.basis, state.direction, state.smooth)
        })?;
        let loc: Point3 = state.loc;
        dump_field(stream, depth, "loc", |nested, _| loc.dump_json(nested))
    }
}
impl GeomSweptSurface for GeomSurfaceOfRevolution {
    fn direction(&self) -> Direction3 {
        self.state.read().direction
    }
    fn basis_curve(&self) -> Handle<dyn GeomCurve> {
        self.state.read().basis.clone()
    }
}
impl GeomSurface for GeomSurfaceOfRevolution {
    fn u_reverse(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.direction.reverse();
        Ok(())
    }
    fn u_reversed_parameter(&self, u: f64) -> f64 {
        2. * PI - u
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.basis.reverse()
    }
    fn v_reversed_parameter(&self, v: f64) -> f64 {
        self.basis_curve().reversed_parameter(v)
    }
    fn transform_parameters(&self, u: f64, v: f64, transform: Transform3) -> (f64, f64) {
        (u, self.basis_curve().transformed_parameter(v, transform))
    }
    fn parametric_transformation(
        &self,
        transform: Transform3,
    ) -> Result<GeneralTransform2, Failure> {
        let ratio: f64 = self.basis_curve().parametric_transformation(transform);
        Ok(v_affinity(ratio))
    }
    fn bounds(&self) -> SurfaceBounds {
        let basis: Handle<dyn GeomCurve> = self.basis_curve();
        (
            0.0,
            2.0 * PI,
            basis.first_parameter(),
            basis.last_parameter(),
        )
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        self.basis_curve().is_closed()
    }
    fn is_u_periodic(&self) -> bool {
        true
    }
    fn is_v_periodic(&self) -> bool {
        self.basis_curve().is_periodic()
    }
    /// A copy of the basis rotated by `u` around the axis.
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let (basis, axis) = self.parts();
        let copy: Handle<dyn GeomCurve> = basis.copy()?.cast();
        copy.rotate(axis, u)?;
        Ok(copy)
    }
    /// The parallel circle through the basis point of parameter `v`.
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let (basis, axis) = self.parts();
        let pc: Point3 = basis.value(v)?;
        let rad: f64 = Line3::from_axis(axis).distance(pc);
        let frame: Axis2 = iso_frame(pc, axis.location(), axis.direction(), rad)?;
        Ok(Handle::new(GeomCircle::new(frame, rad)?).cast())
    }
    fn continuity(&self) -> Shape {
        self.state.read().smooth
    }
    fn is_cn_u(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_cn_v(&self, n: i32) -> Result<bool, Failure> {
        if n < 0 {
            return Err(errors::range(&format!(
                "Geom_SurfaceOfRevolution::IsCNv: N = {n}; expected N >= 0"
            )));
        }
        self.basis_curve().is_cn(n)
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let rep = self.eval_representation();
        if let Some(point) = rep_utils::try_eval_surface_d0(&rep, u, v)? {
            return Ok(point);
        }
        let (basis, axis) = self.parts();
        revolution_utils::d0(u, v, &*basis, axis)
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let rep = self.eval_representation();
        if let Some(result) = rep_utils::try_eval_surface_d1(&rep, u, v)? {
            return Ok(result);
        }
        let (basis, axis) = self.parts();
        revolution_utils::d1(u, v, &*basis, axis)
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let rep = self.eval_representation();
        if let Some(result) = rep_utils::try_eval_surface_d2(&rep, u, v)? {
            return Ok(result);
        }
        let (basis, axis) = self.parts();
        revolution_utils::d2(u, v, &*basis, axis)
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        let rep = self.eval_representation();
        if let Some(result) = rep_utils::try_eval_surface_d3(&rep, u, v)? {
            return Ok(result);
        }
        let (basis, axis) = self.parts();
        revolution_utils::d3(u, v, &*basis, axis)
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_SurfaceOfRevolution", nu, nv)?;
        let rep = self.eval_representation();
        if let Some(result) = rep_utils::try_eval_surface_dn(&rep, u, v, nu, nv)? {
            return Ok(result);
        }
        let (basis, axis) = self.parts();
        revolution_utils::dn(u, v, &*basis, axis, (nu, nv))
    }
}
