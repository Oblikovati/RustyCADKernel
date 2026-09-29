// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_TransformedCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAdaptor_TransformedCurve`: a 3D curve adaptor, or a curve on a surface when
//! one is loaded, seen through a transformation.
use super::curve::GeomAdaptorCurve;
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use crate::geom::errors::{no_such_object, null_object};
use crate::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3,
    GeomGeometry, GeomOffsetCurve,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, SharedState, StandardTransient};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{
    Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Transform3, TransformForm, Vector3,
};

#[derive(Clone, Default)]
struct TransformedCurveState {
    con_surf: Handle<Adaptor3dCurveOnSurface>,
    trsf: Transform3,
}

/// A curve adaptor with a transformation (`GeomAdaptor_TransformedCurve`): the 3D
/// adaptor, or the loaded curve on a surface, with every evaluation and every
/// canonical kind transformed; `GeomAdaptorTransformedCurve::with_curve(&curve, trsf)?`.
pub struct GeomAdaptorTransformedCurve {
    parent: StandardTransient,
    curve: Handle<GeomAdaptorCurve>,
    state: SharedState<TransformedCurveState>,
}
impl Default for GeomAdaptorTransformedCurve {
    fn default() -> Self {
        Self::from_parts(
            Handle::new(GeomAdaptorCurve::default()),
            TransformedCurveState::default(),
        )
    }
}
impl_transient!(
    GeomAdaptorTransformedCurve,
    "GeomAdaptor_TransformedCurve",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

impl GeomAdaptorTransformedCurve {
    fn from_parts(curve: Handle<GeomAdaptorCurve>, state: TransformedCurveState) -> Self {
        Self {
            parent: StandardTransient::default(),
            curve,
            state: SharedState::new(state),
        }
    }
    /// An adaptor on `curve` over its bounds with `trsf`
    /// (`GeomAdaptor_TransformedCurve(C, Trsf)`); `with_curve(&curve, trsf)?`.
    pub fn with_curve(curve: &Handle<dyn GeomCurve>, trsf: Transform3) -> Result<Self, Failure> {
        let adaptor = GeomAdaptorCurve::with_curve(curve)?;
        Ok(Self::from_parts(
            Handle::new(adaptor),
            TransformedCurveState {
                trsf,
                ..Default::default()
            },
        ))
    }
    /// An adaptor on `curve` over `[first, last]` with `trsf`
    /// (`GeomAdaptor_TransformedCurve(C, First, Last, Trsf)`); `with_range(&c, 0.0, 1.0, trsf)?`.
    pub fn with_range(
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
        trsf: Transform3,
    ) -> Result<Self, Failure> {
        let adaptor = GeomAdaptorCurve::with_range(curve, first, last)?;
        Ok(Self::from_parts(
            Handle::new(adaptor),
            TransformedCurveState {
                trsf,
                ..Default::default()
            },
        ))
    }
    /// A copy with shallow copies of the 3D adaptor and of the curve on a surface
    /// (`ShallowCopy`, as a value, which the derived BRepAdaptor_Curve embeds);
    /// `adaptor.shallow_adaptor()?`.
    pub fn shallow_adaptor(&self) -> Result<Self, Failure> {
        let curve = Handle::new(self.curve.shallow_adaptor()?);
        let source = self.state.read().clone();
        let con_surf: Handle<Adaptor3dCurveOnSurface> = match source.con_surf.get() {
            Some(con_surf) => con_surf.shallow_copy()?.cast(),
            None => Handle::default(),
        };
        let state = TransformedCurveState {
            con_surf,
            trsf: source.trsf,
        };
        Ok(Self::from_parts(curve, state))
    }
    /// Load `curve` into the 3D adaptor (`Load(C)`); `adaptor.load(&curve)?`.
    pub fn load(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        self.curve.load(curve)
    }
    /// Load `curve` over `[first, last]` into the 3D adaptor (`Load(C, First, Last)`);
    /// `adaptor.load_range(&curve, 0.0, 1.0)?`.
    pub fn load_range(
        &self,
        curve: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        self.curve.load_range(curve, first, last)
    }
    /// Use a curve on a surface instead of the 3D adaptor (`LoadCurveOnSurface`); a null
    /// handle goes back to the 3D adaptor; `adaptor.load_curve_on_surface(&cos)`.
    pub fn load_curve_on_surface(&self, curve_on_surface: &Handle<Adaptor3dCurveOnSurface>) {
        self.state.write().con_surf = curve_on_surface.clone();
    }
    /// Replace the transformation (`SetTrsf`); `adaptor.set_trsf(trsf)`.
    pub fn set_trsf(&self, trsf: Transform3) {
        self.state.write().trsf = trsf;
    }
    /// The transformation (`Trsf`); `adaptor.trsf()`.
    pub fn trsf(&self) -> Transform3 {
        self.state.read().trsf
    }
    /// Whether the 3D adaptor is used (`Is3DCurve`); `adaptor.is_3d_curve()`.
    pub fn is_3d_curve(&self) -> bool {
        self.state.read().con_surf.is_null()
    }
    /// Whether a curve on a surface is used (`IsCurveOnSurface`); `adaptor.is_curve_on_surface()`.
    pub fn is_curve_on_surface(&self) -> bool {
        !self.is_3d_curve()
    }
    /// The 3D adaptor (`Curve`); `adaptor.curve()`.
    pub fn curve(&self) -> Handle<GeomAdaptorCurve> {
        self.curve.clone()
    }
    /// The 3D adaptor, for edition through its `&self` members (`ChangeCurve`);
    /// `adaptor.change_curve().load(&curve)?`.
    pub fn change_curve(&self) -> Handle<GeomAdaptorCurve> {
        self.curve.clone()
    }
    /// The curve on a surface; Standard_NullObject when none is loaded (natively a null
    /// dereference) (`CurveOnSurface`); `adaptor.curve_on_surface()?`.
    pub fn curve_on_surface(&self) -> Result<Handle<Adaptor3dCurveOnSurface>, Failure> {
        let con_surf = self.state.read().con_surf.clone();
        if con_surf.is_null() {
            return Err(null_object(
                "GeomAdaptor_TransformedCurve::CurveOnSurface: none is loaded",
            ));
        }
        Ok(con_surf)
    }
    /// The Geom curve of the 3D adaptor (`GeomCurve`); `adaptor.geom_curve()`.
    pub fn geom_curve(&self) -> Handle<dyn GeomCurve> {
        self.curve.curve()
    }
    // The adapted curve: the curve on a surface when loaded, else the 3D adaptor.
    fn source(&self) -> Handle<dyn Adaptor3dCurve> {
        let con_surf = self.state.read().con_surf.clone();
        if con_surf.is_null() {
            return self.curve.clone().cast();
        }
        con_surf.cast()
    }
    fn identity(&self) -> bool {
        self.trsf().form() == TransformForm::Identity
    }
    // A spline of the source, transformed (a copy) unless the transformation is the
    // identity; natively a null spline is dereferenced.
    fn spline<T: RuntimeTyped + GeomGeometry>(
        &self,
        spline: Handle<T>,
        member: &str,
    ) -> Result<Handle<T>, Failure> {
        if self.identity() {
            return Ok(spline);
        }
        let Some(geometry) = spline.get() else {
            return Err(null_object(&format!(
                "GeomAdaptor_TransformedCurve::{member}: null curve"
            )));
        };
        Ok(geometry.transformed(self.trsf())?.cast())
    }
}

impl Adaptor3dCurve for GeomAdaptorTransformedCurve {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        Ok(Handle::new(self.shallow_adaptor()?).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        self.source().first_parameter()
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        self.source().last_parameter()
    }
    fn continuity(&self) -> Result<Shape, Failure> {
        self.source().continuity()
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.source().nb_intervals(shape)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.source().intervals(bounds, shape)
    }
    // A new adaptor with the same transformation: the 3D curve reloaded on the range,
    // or the trimmed curve on a surface.
    fn trim(
        &self,
        first: f64,
        last: f64,
        tolerance: f64,
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let source = self.state.read().clone();
        let copy = Self::default();
        match source.con_surf.get() {
            None => copy.curve.load_range(&self.curve.curve(), first, last)?,
            Some(con_surf) => {
                copy.state.write().con_surf = con_surf.trim(first, last, tolerance)?.cast()
            }
        }
        copy.set_trsf(source.trsf);
        Ok(Handle::new(copy).cast())
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        self.source().is_closed()
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        self.source().is_periodic()
    }
    fn period(&self) -> Result<f64, Failure> {
        self.source().period()
    }
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        self.source().resolution(r3d)
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        self.source().get_type()
    }
    fn line(&self) -> Result<Line3, Failure> {
        Ok(self.source().line()?.transformed(self.trsf()))
    }
    fn circle(&self) -> Result<Circle3, Failure> {
        self.source().circle()?.transformed(self.trsf())
    }
    fn ellipse(&self) -> Result<Ellipse3, Failure> {
        self.source().ellipse()?.transformed(self.trsf())
    }
    fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
        self.source().hyperbola()?.transformed(self.trsf())
    }
    fn parabola(&self) -> Result<Parabola3, Failure> {
        self.source().parabola()?.transformed(self.trsf())
    }
    fn degree(&self) -> Result<i32, Failure> {
        self.source().degree()
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        self.source().is_rational()
    }
    fn nb_poles(&self) -> Result<i32, Failure> {
        self.source().nb_poles()
    }
    fn nb_knots(&self) -> Result<i32, Failure> {
        self.source().nb_knots()
    }
    fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
        self.spline(self.source().bezier()?, "Bezier")
    }
    fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
        self.spline(self.source().bspline()?, "BSpline")
    }
    fn offset_curve(&self) -> Result<Handle<GeomOffsetCurve>, Failure> {
        if !self.is_3d_curve() || self.curve.get_type()? != CurveType::OffsetCurve {
            return Err(no_such_object("GeomAdaptor_TransformedCurve::OffsetCurve"));
        }
        self.spline(self.curve.offset_curve()?, "OffsetCurve")
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        Ok(self.source().eval_d0(u)?.transformed(self.trsf()))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let trsf: Transform3 = self.trsf();
        let result: GeomCurveD1 = self.source().eval_d1(u)?;
        Ok(GeomCurveD1 {
            point: result.point.transformed(trsf),
            d1: result.d1.transformed(trsf),
        })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let trsf: Transform3 = self.trsf();
        let r: GeomCurveD2 = self.source().eval_d2(u)?;
        Ok(GeomCurveD2 {
            point: r.point.transformed(trsf),
            d1: r.d1.transformed(trsf),
            d2: r.d2.transformed(trsf),
        })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let trsf: Transform3 = self.trsf();
        let r: GeomCurveD3 = self.source().eval_d3(u)?;
        Ok(GeomCurveD3 {
            point: r.point.transformed(trsf),
            d1: r.d1.transformed(trsf),
            d2: r.d2.transformed(trsf),
            d3: r.d3.transformed(trsf),
        })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        Ok(self.source().eval_dn(u, n)?.transformed(self.trsf()))
    }
}
