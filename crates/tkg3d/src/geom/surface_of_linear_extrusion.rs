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
// Rust translation of ModelingData/TKG3d/Geom/Geom_SurfaceOfLinearExtrusion.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_SurfaceOfLinearExtrusion`: a basis curve swept along a direction,
//! `P(u, v) = C(u) + v * Dir`.
use super::curve::GeomCurve;
use super::errors;
use super::extrusion_utils;
use super::geometry::GeomGeometry;
use super::line::GeomLine;
use super::parametric::{scaled_parameter, u_affinity, v_affinity};
use super::surface::{
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds, check_dn_orders,
};
use super::swept_surface::{GeomSweptSurface, dump_swept_surface};
use crate::geom_eval::{RepSurfaceDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Direction3, GeneralTransform2, Point3, Transform3, Vector3, Xyz};
use std::io;

struct ExtrusionState {
    basis: Handle<dyn GeomCurve>,
    direction: Direction3,
    smooth: Shape,
    eval_rep: Handle<dyn RepSurfaceDesc>,
}

/// A surface of linear extrusion; the basis curve is a private copy;
/// `Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, dir)?)`.
pub struct GeomSurfaceOfLinearExtrusion {
    parent: StandardTransient,
    state: SharedState<ExtrusionState>,
}
impl_transient!(
    GeomSurfaceOfLinearExtrusion,
    "Geom_SurfaceOfLinearExtrusion",
    dyn GeomSweptSurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomSweptSurface]
);

// `occ::down_cast<Geom_Curve>(C->Copy())` and `C->Continuity()`, with the native null
// dereference turned into an error.
pub(crate) fn copied_basis(
    curve: &Handle<dyn GeomCurve>,
) -> Result<(Handle<dyn GeomCurve>, Shape), Failure> {
    let Some(source) = curve.get() else {
        return Err(errors::construction(
            "swept surface: the basis curve is null; expected a curve",
        ));
    };
    Ok((source.copy()?.cast(), source.continuity()))
}

impl GeomSurfaceOfLinearExtrusion {
    /// Extrusion of a copy of `curve` along `direction`; the continuity is the curve
    /// one (`Geom_SurfaceOfLinearExtrusion(C, V)`);
    /// `GeomSurfaceOfLinearExtrusion::new(&curve, dir)?`.
    pub fn new(curve: &Handle<dyn GeomCurve>, direction: Direction3) -> Result<Self, Failure> {
        let (basis, smooth) = copied_basis(curve)?;
        Ok(Self {
            parent: StandardTransient::default(),
            state: SharedState::new(ExtrusionState {
                basis,
                direction,
                smooth,
                eval_rep: Handle::default(),
            }),
        })
    }
    /// The native Copy: a new extrusion of the basis (copied again, continuity read
    /// again) with a deep copy of the evaluation representation;
    /// `GeomSurfaceOfLinearExtrusion::copy_of(&surface)?`.
    pub fn copy_of(other: &GeomSurfaceOfLinearExtrusion) -> Result<Self, Failure> {
        let (basis, direction, eval_rep) = {
            let source = other.state.read();
            (
                source.basis.clone(),
                source.direction,
                source.eval_rep.clone(),
            )
        };
        let copy = Self::new(&basis, direction)?;
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
    /// Replace the extrusion direction (`SetDirection`); `surface.set_direction(d)`.
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
        state.smooth = smooth;
        state.basis = basis;
        Ok(())
    }
    fn parts(&self) -> (Handle<dyn GeomCurve>, Xyz) {
        let state = self.state.read();
        (state.basis.clone(), state.direction.xyz())
    }
}
impl GeomGeometry for GeomSurfaceOfLinearExtrusion {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.direction.transform(transform);
        state.basis.transform(transform)
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom_SurfaceOfLinearExtrusion")?;
        dump_base(stream, depth, |s, d| {
            dump_swept_surface(s, d, &state.basis, state.direction, state.smooth)
        })
    }
}
impl GeomSweptSurface for GeomSurfaceOfLinearExtrusion {
    fn direction(&self) -> Direction3 {
        self.state.read().direction
    }
    fn basis_curve(&self) -> Handle<dyn GeomCurve> {
        self.state.read().basis.clone()
    }
}
impl GeomSurface for GeomSurfaceOfLinearExtrusion {
    fn u_reverse(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.basis.reverse()
    }
    fn u_reversed_parameter(&self, u: f64) -> f64 {
        self.state.read().basis.reversed_parameter(u)
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.direction.reverse();
        Ok(())
    }
    fn v_reversed_parameter(&self, v: f64) -> f64 {
        -v
    }
    fn transform_parameters(&self, u: f64, v: f64, transform: Transform3) -> (f64, f64) {
        let u: f64 = self.state.read().basis.transformed_parameter(u, transform);
        (u, scaled_parameter(v, transform))
    }
    /// `TU * TV`: the basis ratio on U times the scale on V.
    fn parametric_transformation(
        &self,
        transform: Transform3,
    ) -> Result<GeneralTransform2, Failure> {
        let tv: GeneralTransform2 = v_affinity(transform.scale_factor().abs());
        let ratio: f64 = self.state.read().basis.parametric_transformation(transform);
        let tu: GeneralTransform2 = u_affinity(ratio);
        tu * tv
    }
    fn bounds(&self) -> SurfaceBounds {
        let basis: Handle<dyn GeomCurve> = self.basis_curve();
        let infinite: f64 = precision::infinite();
        (
            basis.first_parameter(),
            basis.last_parameter(),
            -infinite,
            infinite,
        )
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        self.basis_curve().is_closed()
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_u_periodic(&self) -> bool {
        self.basis_curve().is_periodic()
    }
    fn is_v_periodic(&self) -> bool {
        false
    }
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let point: Point3 = self.basis_curve().value(u)?;
        let line = GeomLine::from_point(point, self.direction());
        Ok(Handle::new(line).cast())
    }
    /// A copy of the basis translated by `v * Dir`.
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let mut vdir = Vector3::from(self.direction());
        vdir *= v;
        Ok(self.basis_curve().translated(vdir)?.cast())
    }
    fn continuity(&self) -> Shape {
        self.state.read().smooth
    }
    fn is_cn_u(&self, n: i32) -> Result<bool, Failure> {
        if n < 0 {
            return Err(errors::range(&format!(
                "Geom_SurfaceOfLinearExtrusion::IsCNu: N = {n}; expected N >= 0"
            )));
        }
        self.basis_curve().is_cn(n)
    }
    fn is_cn_v(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let rep = self.eval_representation();
        if let Some(point) = rep_utils::try_eval_surface_d0(&rep, u, v)? {
            return Ok(point);
        }
        let (basis, dir) = self.parts();
        extrusion_utils::d0(u, v, &*basis, dir)
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let rep = self.eval_representation();
        if let Some(result) = rep_utils::try_eval_surface_d1(&rep, u, v)? {
            return Ok(result);
        }
        let (basis, dir) = self.parts();
        extrusion_utils::d1(u, v, &*basis, dir)
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let rep = self.eval_representation();
        if let Some(result) = rep_utils::try_eval_surface_d2(&rep, u, v)? {
            return Ok(result);
        }
        let (basis, dir) = self.parts();
        extrusion_utils::d2(u, v, &*basis, dir)
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        let rep = self.eval_representation();
        if let Some(result) = rep_utils::try_eval_surface_d3(&rep, u, v)? {
            return Ok(result);
        }
        let (basis, dir) = self.parts();
        extrusion_utils::d3(u, v, &*basis, dir)
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_SurfaceOfLinearExtrusion", nu, nv)?;
        let rep = self.eval_representation();
        if let Some(result) = rep_utils::try_eval_surface_dn(&rep, u, v, nu, nv)? {
            return Ok(result);
        }
        let (basis, dir) = self.parts();
        extrusion_utils::dn(u, &*basis, dir, nu, nv)
    }
}
