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
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_SurfaceOfLinearExtrusion.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAdaptor_SurfaceOfLinearExtrusion`: an adapted curve extruded along a
//! direction, `S(u, v) = C(u) + v * D`.
use super::extrusion_kind::{extrusion_cylinder, extrusion_plane, extrusion_type};
use super::surface::GeomAdaptorSurface;
use super::surface_state::{ExtrusionData, SurfaceData, SurfaceState, with_data};
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::errors::{array_bounds, domain, no_such_object, null_object};
use crate::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomSurface, GeomSurfaceD1, GeomSurfaceD2,
    GeomSurfaceD3, SurfaceBounds,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};

/// The own fields of the extrusion adaptor; the direction defaults to X as gp_Dir().
#[derive(Clone, Default)]
pub(super) struct ExtrusionFields {
    pub(super) basis_curve: Handle<dyn Adaptor3dCurve>,
    pub(super) direction: Direction3,
    pub(super) have_dir: bool,
}
impl ExtrusionFields {
    /// The extruded curve; natively a null one is dereferenced; `fields.curve()?`.
    pub(super) fn curve(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        self.basis_curve.get().ok_or_else(|| {
            null_object("GeomAdaptor_SurfaceOfLinearExtrusion: no basis curve; expected Load(C)")
        })
    }
}

/// An adapted curve extruded along a direction (`GeomAdaptor_SurfaceOfLinearExtrusion`).
///
/// It derives from `GeomAdaptor_Surface` natively: loading the direction records
/// the extrusion kind and data in the base state, whose evaluators it uses, while the
/// bounds, continuity, intervals and kind queries come from the basis curve;
/// `GeomAdaptorSurfaceOfLinearExtrusion::with_curve_and_direction(&curve, dir)`.
#[derive(Default)]
pub struct GeomAdaptorSurfaceOfLinearExtrusion {
    parent: GeomAdaptorSurface,
    fields: SharedState<ExtrusionFields>,
}
impl_transient!(
    GeomAdaptorSurfaceOfLinearExtrusion,
    "GeomAdaptor_SurfaceOfLinearExtrusion",
    GeomAdaptorSurface,
    parent,
    interfaces: [dyn Adaptor3dSurface]
);

impl GeomAdaptorSurfaceOfLinearExtrusion {
    /// An extrusion of `curve` without direction (`GeomAdaptor_SurfaceOfLinearExtrusion(C)`);
    /// `GeomAdaptorSurfaceOfLinearExtrusion::with_curve(&curve)`.
    pub fn with_curve(curve: &Handle<dyn Adaptor3dCurve>) -> Self {
        let adaptor = Self::default();
        adaptor.load_curve(curve);
        adaptor
    }
    /// An extrusion of `curve` along `direction` (`GeomAdaptor_SurfaceOfLinearExtrusion(C, V)`);
    /// `GeomAdaptorSurfaceOfLinearExtrusion::with_curve_and_direction(&curve, dir)`.
    pub fn with_curve_and_direction(
        curve: &Handle<dyn Adaptor3dCurve>,
        direction: Direction3,
    ) -> Self {
        let adaptor = Self::with_curve(curve);
        adaptor.load_direction(direction);
        adaptor
    }
    /// Change the curve, keeping a direction already loaded (`Load(C)`);
    /// `adaptor.load_curve(&curve)`.
    pub fn load_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) {
        let direction: Option<Direction3> = {
            let mut fields = self.fields.write();
            fields.basis_curve = curve.clone();
            fields.have_dir.then_some(fields.direction)
        };
        if let Some(direction) = direction {
            self.load_direction(direction);
        }
    }
    /// Set the direction and record the extrusion data in the base state (`Load(V)`);
    /// `adaptor.load_direction(dir)`.
    pub fn load_direction(&self, direction: Direction3) {
        let curve: Handle<dyn Adaptor3dCurve> = {
            let mut fields = self.fields.write();
            fields.have_dir = true;
            fields.direction = direction;
            fields.basis_curve.clone()
        };
        let mut state = self.parent.cell().write();
        state.surface_type = SurfaceType::SurfaceOfExtrusion;
        state.data = SurfaceData::Extrusion(ExtrusionData {
            basis_curve: curve,
            direction: direction.xyz(),
            eval_rep: Handle::default(),
        });
    }
    /// The base-class part (`GeomAdaptor_Surface`), whose own members (`Surface`,
    /// `ToleranceU`...) read the base state; `adaptor.as_geom_adaptor_surface()`.
    pub fn as_geom_adaptor_surface(&self) -> &GeomAdaptorSurface {
        &self.parent
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
    fn snapshot(&self) -> ExtrusionFields {
        self.fields.read().clone()
    }
    // A new extrusion of `curve` along the direction of this one.
    fn extruded(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Handle<dyn Adaptor3dSurface> {
        let direction: Direction3 = self.fields.read().direction;
        Handle::new(Self::with_curve_and_direction(curve, direction)).cast()
    }
    // ShallowCopy of the base part: the fields, and extrusion data (without its
    // representation) only when the base holds some.
    fn shallow_base(state: &SurfaceState) -> Result<SurfaceState, Failure> {
        let data: SurfaceData = match &state.data {
            SurfaceData::Extrusion(data) => SurfaceData::Extrusion(ExtrusionData {
                basis_curve: data.curve()?.shallow_copy()?,
                direction: data.direction,
                eval_rep: Handle::default(),
            }),
            _ => SurfaceData::None,
        };
        Ok(with_data(state, data))
    }
}

fn not_this_kind(member: &str) -> Failure {
    no_such_object(&format!(
        "GeomAdaptor_SurfaceOfLinearExtrusion::{member}: not defined on an extrusion"
    ))
}

impl Adaptor3dSurface for GeomAdaptorSurfaceOfLinearExtrusion {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let mut fields: ExtrusionFields = self.snapshot();
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
        self.snapshot().curve()?.first_parameter()
    }
    fn last_u_parameter(&self) -> Result<f64, Failure> {
        self.snapshot().curve()?.last_parameter()
    }
    fn first_v_parameter(&self) -> Result<f64, Failure> {
        Ok(-f64::MAX) // RealFirst()
    }
    fn last_v_parameter(&self) -> Result<f64, Failure> {
        Ok(f64::MAX) // RealLast()
    }
    fn u_continuity(&self) -> Result<Shape, Failure> {
        self.snapshot().curve()?.continuity()
    }
    fn v_continuity(&self) -> Result<Shape, Failure> {
        Ok(Shape::CN)
    }
    fn nb_u_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.snapshot().curve()?.nb_intervals(shape)
    }
    fn nb_v_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Ok(1)
    }
    fn u_intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.snapshot().curve()?.intervals(bounds, shape)
    }
    fn v_intervals(&self, bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        let lower: i32 = bounds.lower();
        bounds
            .set_value(lower, self.first_v_parameter()?)
            .map_err(array_bounds)?;
        bounds
            .set_value(lower + 1, self.last_v_parameter()?)
            .map_err(array_bounds)
    }
    // Natively the U range is ignored: the same extrusion.
    fn u_trim(
        &self,
        _first: f64,
        _last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        Ok(self.extruded(&self.snapshot().basis_curve))
    }
    // Natively the V range trims the basis curve, whose parameter is U.
    fn v_trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let trimmed: Handle<dyn Adaptor3dCurve> =
            self.basis_curve()?.trim(first, last, tolerance)?;
        Ok(self.extruded(&trimmed))
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        self.snapshot().curve()?.is_closed()
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn is_u_periodic(&self) -> Result<bool, Failure> {
        self.snapshot().curve()?.is_periodic()
    }
    fn u_period(&self) -> Result<f64, Failure> {
        self.snapshot().curve()?.period()
    }
    fn is_v_periodic(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    fn v_period(&self) -> Result<f64, Failure> {
        Err(domain(
            "GeomAdaptor_SurfaceOfLinearExtrusion::VPeriod: an extrusion is not V-periodic",
        ))
    }
    fn u_resolution(&self, r3d: f64) -> Result<f64, Failure> {
        self.snapshot().curve()?.resolution(r3d)
    }
    fn v_resolution(&self, r3d: f64) -> Result<f64, Failure> {
        Ok(r3d)
    }
    fn get_type(&self) -> Result<SurfaceType, Failure> {
        let fields = self.snapshot();
        extrusion_type(fields.curve()?, fields.direction)
    }
    fn plane(&self) -> Result<Plane3, Failure> {
        let fields = self.snapshot();
        if self.get_type()? != SurfaceType::Plane {
            return Err(not_this_kind("Plane"));
        }
        extrusion_plane(fields.curve()?, fields.direction)
    }
    fn cylinder(&self) -> Result<Cylinder3, Failure> {
        let fields = self.snapshot();
        if self.get_type()? != SurfaceType::Cylinder {
            return Err(not_this_kind("Cylinder"));
        }
        extrusion_cylinder(fields.curve()?, fields.direction)
    }
    fn cone(&self) -> Result<Cone3, Failure> {
        Err(not_this_kind("Cone"))
    }
    fn sphere(&self) -> Result<Sphere3, Failure> {
        Err(not_this_kind("Sphere"))
    }
    fn torus(&self) -> Result<Torus3, Failure> {
        Err(not_this_kind("Torus"))
    }
    fn u_degree(&self) -> Result<i32, Failure> {
        self.snapshot().curve()?.degree()
    }
    fn nb_u_poles(&self) -> Result<i32, Failure> {
        self.snapshot().curve()?.nb_poles()
    }
    fn v_degree(&self) -> Result<i32, Failure> {
        self.parent.v_degree()
    }
    fn nb_v_poles(&self) -> Result<i32, Failure> {
        self.parent.nb_v_poles()
    }
    fn nb_u_knots(&self) -> Result<i32, Failure> {
        self.parent.nb_u_knots()
    }
    fn nb_v_knots(&self) -> Result<i32, Failure> {
        self.parent.nb_v_knots()
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
    // The native message says "Axes".
    fn axe_of_revolution(&self) -> Result<Axis1, Failure> {
        Err(not_this_kind("Axes"))
    }
    fn direction(&self) -> Result<Direction3, Failure> {
        Ok(self.fields.read().direction)
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

// The members that GeomAdaptor_SurfaceOfLinearExtrusion inherits unchanged.
impl GeomAdaptorSurfaceOfLinearExtrusion {
    /// The base surface, null for an adaptor built from a curve (`Surface`);
    /// `adaptor.surface()`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.parent.surface()
    }
}
