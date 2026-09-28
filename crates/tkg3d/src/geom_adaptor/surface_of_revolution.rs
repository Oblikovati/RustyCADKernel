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
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_SurfaceOfRevolution.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAdaptor_SurfaceOfRevolution`: an adapted curve revolved around an axis;
//! U is the angle, V the curve parameter.
use super::revolution_kind::{
    Revolved, axe_of_revolution, revolution_cone, revolution_cylinder, revolution_plane,
    revolution_sphere, revolution_torus, revolution_type,
};
use super::surface::GeomAdaptorSurface;
use super::surface_state::{RevolutionData, SurfaceData, SurfaceState, with_data};
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::errors::{array_bounds, no_such_object, null_object, out_of_range};
use crate::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomSurface, GeomSurfaceD1, GeomSurfaceD2,
    GeomSurfaceD3, SurfaceBounds,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Axis3, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};
use std::f64::consts::PI;

/// The own fields of the revolution adaptor.
#[derive(Clone, Default)]
struct RevolutionFields {
    basis_curve: Handle<dyn Adaptor3dCurve>,
    axis: Axis1,
    have_axis: bool,
    axe_rev: Axis3,
}
impl RevolutionFields {
    fn curve(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        self.basis_curve.get().ok_or_else(|| {
            null_object("GeomAdaptor_SurfaceOfRevolution: no basis curve; expected Load(C)")
        })
    }
}

/// An adapted curve revolved around an axis (`GeomAdaptor_SurfaceOfRevolution`).
///
/// It derives from `GeomAdaptor_Surface` natively: loading the axis records the
/// revolution kind and data in the base state, whose evaluators it uses, and builds
/// the auxiliary frame of the canonical kinds (`Axis`); the V bounds, continuity and
/// intervals come from the basis curve;
/// `GeomAdaptorSurfaceOfRevolution::with_curve_and_axis(&curve, axis)?`.
#[derive(Default)]
pub struct GeomAdaptorSurfaceOfRevolution {
    parent: GeomAdaptorSurface,
    fields: SharedState<RevolutionFields>,
}
impl_transient!(
    GeomAdaptorSurfaceOfRevolution,
    "GeomAdaptor_SurfaceOfRevolution",
    GeomAdaptorSurface,
    parent,
    interfaces: [dyn Adaptor3dSurface]
);

impl GeomAdaptorSurfaceOfRevolution {
    /// A revolution of `curve` without axis (`GeomAdaptor_SurfaceOfRevolution(C)`);
    /// `GeomAdaptorSurfaceOfRevolution::with_curve(&curve)`.
    pub fn with_curve(curve: &Handle<dyn Adaptor3dCurve>) -> Self {
        let adaptor = Self::default();
        adaptor.fields.write().basis_curve = curve.clone();
        adaptor
    }
    /// A revolution of `curve` around `axis` (`GeomAdaptor_SurfaceOfRevolution(C, V)`);
    /// Standard_ConstructionError when the curve lies on the axis;
    /// `GeomAdaptorSurfaceOfRevolution::with_curve_and_axis(&curve, axis)?`.
    pub fn with_curve_and_axis(
        curve: &Handle<dyn Adaptor3dCurve>,
        axis: Axis1,
    ) -> Result<Self, Failure> {
        let adaptor = Self::with_curve(curve);
        adaptor.load_axis(axis)?;
        Ok(adaptor)
    }
    /// Change the curve, recomputing the frame when an axis is loaded (`Load(C)`);
    /// `adaptor.load_curve(&curve)?`.
    pub fn load_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<(), Failure> {
        let axis: Option<Axis1> = {
            let mut fields = self.fields.write();
            fields.basis_curve = curve.clone();
            fields.have_axis.then_some(fields.axis)
        };
        match axis {
            // To evaluate the new myAxeRev.
            Some(axis) => self.load_axis(axis),
            None => Ok(()),
        }
    }
    /// Set the axis, record the revolution data in the base state, then compute the
    /// auxiliary frame (`Load(V)`); on a failure the axis and data stay loaded with
    /// the previous frame, as natively; `adaptor.load_axis(axis)?`.
    pub fn load_axis(&self, axis: Axis1) -> Result<(), Failure> {
        let fields: RevolutionFields = {
            let mut fields = self.fields.write();
            fields.have_axis = true;
            fields.axis = axis;
            fields.clone()
        };
        {
            let mut state = self.parent.cell().write();
            state.surface_type = SurfaceType::SurfaceOfRevolution;
            state.data = SurfaceData::Revolution(RevolutionData {
                basis_curve: fields.basis_curve.clone(),
                axis,
                eval_rep: Handle::default(),
            });
        }
        let frame: Axis3 = axe_of_revolution(&self.revolved(&fields)?)?;
        self.fields.write().axe_rev = frame;
        Ok(())
    }
    /// The auxiliary frame of the canonical kinds (`Axis`); `adaptor.axis()`.
    pub fn axis(&self) -> Axis3 {
        self.fields.read().axe_rev
    }
    /// The base-class part (`GeomAdaptor_Surface`); `adaptor.as_geom_adaptor_surface()`.
    pub fn as_geom_adaptor_surface(&self) -> &GeomAdaptorSurface {
        &self.parent
    }
    /// The base surface, null for an adaptor built from a curve (`Surface`);
    /// `adaptor.surface()`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.parent.surface()
    }
    /// `Bounds` through the overridden bounds; `let (u1, u2, v1, v2) = adaptor.bounds()?;`.
    pub fn bounds(&self) -> Result<SurfaceBounds, Failure> {
        Ok((
            self.first_u_parameter()?,
            self.last_u_parameter()?,
            self.first_v_parameter()?,
            self.last_v_parameter()?,
        ))
    }
    fn snapshot(&self) -> RevolutionFields {
        self.fields.read().clone()
    }
    fn revolved<'a>(&'a self, fields: &'a RevolutionFields) -> Result<Revolved<'a>, Failure> {
        Ok(Revolved {
            base: &self.parent,
            curve: fields.curve()?,
            axis: fields.axis,
            axe_rev: fields.axe_rev,
        })
    }
    // A canonical-kind query: NoSuchObject unless GetType() is `kind`.
    fn checked<T>(
        &self,
        kind: SurfaceType,
        member: &str,
        build: impl Fn(&Revolved<'_>) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let fields: RevolutionFields = self.snapshot();
        let revolved: Revolved<'_> = self.revolved(&fields)?;
        if revolution_type(&revolved)? != kind {
            return Err(no_such_object(&format!(
                "GeomAdaptor_SurfaceOfRevolution:{member}"
            )));
        }
        build(&revolved)
    }
    fn revolved_copy(
        &self,
        curve: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let axis: Axis1 = self.fields.read().axis;
        Ok(Handle::new(Self::with_curve_and_axis(curve, axis)?).cast())
    }
    // ShallowCopy of the base part: the fields, and revolution data (without its
    // representation) only when the base holds some.
    fn shallow_base(state: &SurfaceState) -> Result<SurfaceState, Failure> {
        let data: SurfaceData = match &state.data {
            SurfaceData::Revolution(data) => SurfaceData::Revolution(RevolutionData {
                basis_curve: data.curve()?.shallow_copy()?,
                axis: data.axis,
                eval_rep: Handle::default(),
            }),
            _ => SurfaceData::None,
        };
        Ok(with_data(state, data))
    }
}

fn not_this_kind(member: &str) -> Failure {
    no_such_object(&format!(
        "GeomAdaptor_SurfaceOfRevolution::{member}: not defined on a surface of revolution"
    ))
}

impl Adaptor3dSurface for GeomAdaptorSurfaceOfRevolution {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let mut fields: RevolutionFields = self.snapshot();
        if let Some(curve) = fields.basis_curve.get() {
            fields.basis_curve = curve.shallow_copy()?;
        }
        let base: SurfaceState = Self::shallow_base(&self.parent.cell().read())?;
        let copy = Self {
            parent: GeomAdaptorSurface::from_state(base),
            fields: SharedState::new(fields),
        };
        Ok(Handle::new(copy).cast())
    }
    fn first_u_parameter(&self) -> Result<f64, Failure> {
        Ok(0.)
    }
    fn last_u_parameter(&self) -> Result<f64, Failure> {
        Ok(2. * PI)
    }
    fn first_v_parameter(&self) -> Result<f64, Failure> {
        self.snapshot().curve()?.first_parameter()
    }
    fn last_v_parameter(&self) -> Result<f64, Failure> {
        self.snapshot().curve()?.last_parameter()
    }
    fn u_continuity(&self) -> Result<Shape, Failure> {
        Ok(Shape::CN)
    }
    fn v_continuity(&self) -> Result<Shape, Failure> {
        self.snapshot().curve()?.continuity()
    }
    fn nb_u_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Ok(1)
    }
    fn nb_v_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.snapshot().curve()?.nb_intervals(shape)
    }
    fn u_intervals(&self, bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        let lower: i32 = bounds.lower();
        bounds.set_value(lower, 0.).map_err(array_bounds)?;
        bounds.set_value(lower + 1, 2. * PI).map_err(array_bounds)
    }
    fn v_intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.snapshot().curve()?.intervals(bounds, shape)
    }
    // Only the whole turn is accepted: the same surface.
    fn u_trim(
        &self,
        first: f64,
        last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let eps: f64 = precision::p_confusion();
        if first.abs() > eps || (last - 2. * PI).abs() > eps {
            return Err(out_of_range(&format!(
                "GeomAdaptor_SurfaceOfRevolution : UTrim : Parameters out of range: [{first}, {last}]; expected [0, 2*PI]"
            )));
        }
        self.revolved_copy(&self.snapshot().basis_curve)
    }
    fn v_trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let trimmed: Handle<dyn Adaptor3dCurve> =
            self.basis_curve()?.trim(first, last, tolerance)?;
        self.revolved_copy(&trimmed)
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        self.snapshot().curve()?.is_closed()
    }
    fn is_u_periodic(&self) -> Result<bool, Failure> {
        Ok(true)
    }
    fn u_period(&self) -> Result<f64, Failure> {
        Ok(2. * PI)
    }
    fn is_v_periodic(&self) -> Result<bool, Failure> {
        self.snapshot().curve()?.is_periodic()
    }
    fn v_period(&self) -> Result<f64, Failure> {
        self.snapshot().curve()?.period()
    }
    fn u_resolution(&self, r3d: f64) -> Result<f64, Failure> {
        Ok(precision::parametric(r3d))
    }
    fn v_resolution(&self, r3d: f64) -> Result<f64, Failure> {
        self.snapshot().curve()?.resolution(r3d)
    }
    fn get_type(&self) -> Result<SurfaceType, Failure> {
        let fields: RevolutionFields = self.snapshot();
        revolution_type(&self.revolved(&fields)?)
    }
    fn plane(&self) -> Result<Plane3, Failure> {
        self.checked(SurfaceType::Plane, "Plane", revolution_plane)
    }
    fn cylinder(&self) -> Result<Cylinder3, Failure> {
        self.checked(SurfaceType::Cylinder, ":Cylinder", revolution_cylinder)
    }
    fn cone(&self) -> Result<Cone3, Failure> {
        self.checked(SurfaceType::Cone, "Cone", revolution_cone)
    }
    fn sphere(&self) -> Result<Sphere3, Failure> {
        self.checked(SurfaceType::Sphere, "Sphere", revolution_sphere)
    }
    fn torus(&self) -> Result<Torus3, Failure> {
        self.checked(SurfaceType::Torus, "Torus", revolution_torus)
    }
    fn u_degree(&self) -> Result<i32, Failure> {
        self.parent.u_degree()
    }
    fn nb_u_poles(&self) -> Result<i32, Failure> {
        self.parent.nb_u_poles()
    }
    fn v_degree(&self) -> Result<i32, Failure> {
        self.snapshot().curve()?.degree()
    }
    fn nb_v_poles(&self) -> Result<i32, Failure> {
        self.snapshot().curve()?.nb_poles()
    }
    fn nb_u_knots(&self) -> Result<i32, Failure> {
        self.parent.nb_u_knots()
    }
    fn nb_v_knots(&self) -> Result<i32, Failure> {
        Err(not_this_kind("NbVKnots"))
    }
    fn is_u_rational(&self) -> Result<bool, Failure> {
        Err(not_this_kind("IsURational"))
    }
    fn is_v_rational(&self) -> Result<bool, Failure> {
        Err(not_this_kind("IsVRational"))
    }
    fn bezier(&self) -> Result<Handle<GeomBezierSurface>, Failure> {
        Err(not_this_kind("Bezier"))
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
        Err(not_this_kind("BSpline"))
    }
    fn axe_of_revolution(&self) -> Result<Axis1, Failure> {
        Ok(self.fields.read().axis)
    }
    fn direction(&self) -> Result<Direction3, Failure> {
        self.parent.direction()
    }
    fn basis_curve(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        Ok(self.fields.read().basis_curve.clone())
    }
    fn basis_surface(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        self.parent.basis_surface()
    }
    fn offset_value(&self) -> Result<f64, Failure> {
        self.parent.offset_value()
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.parent.eval_d0(u, v)
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        self.parent.eval_d1(u, v)
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        self.parent.eval_d2(u, v)
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        self.parent.eval_d3(u, v)
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        self.parent.eval_dn(u, v, nu, nv)
    }
}
