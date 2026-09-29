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
// Rust translation support for ModelingData/TKBRep/BRepAdaptor at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The inherited virtual members of the BRepAdaptor classes.
//!
//! `BRepAdaptor_Curve`, `BRepAdaptor_Curve2d` and `BRepAdaptor_Surface` derive from a
//! GeomAdaptor class and override only `ShallowCopy` (and `Trim` for the curve). In
//! Rust each one embeds its parent adaptor as the `parent` field; these macros write
//! the inherited trait members once, as calls on that field. The non-virtual `Value`,
//! `D0`..`DN` of the traits are provided methods over the `Eval*` members, so they
//! are not listed.

/// The `Adaptor3d_Curve` members of `GeomAdaptor_TransformedCurve` that
/// `BRepAdaptor_Curve` inherits (all but `ShallowCopy` and `Trim`).
macro_rules! inherited_curve_members {
    () => {
        fn first_parameter(&self) -> Result<f64, Failure> {
            self.parent.first_parameter()
        }
        fn last_parameter(&self) -> Result<f64, Failure> {
            self.parent.last_parameter()
        }
        fn continuity(&self) -> Result<Shape, Failure> {
            self.parent.continuity()
        }
        fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
            self.parent.nb_intervals(shape)
        }
        fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
            self.parent.intervals(bounds, shape)
        }
        fn is_closed(&self) -> Result<bool, Failure> {
            self.parent.is_closed()
        }
        fn is_periodic(&self) -> Result<bool, Failure> {
            self.parent.is_periodic()
        }
        fn period(&self) -> Result<f64, Failure> {
            self.parent.period()
        }
        fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
            self.parent.resolution(r3d)
        }
        fn get_type(&self) -> Result<CurveType, Failure> {
            self.parent.get_type()
        }
        fn line(&self) -> Result<Line3, Failure> {
            self.parent.line()
        }
        fn circle(&self) -> Result<Circle3, Failure> {
            self.parent.circle()
        }
        fn ellipse(&self) -> Result<Ellipse3, Failure> {
            self.parent.ellipse()
        }
        fn hyperbola(&self) -> Result<Hyperbola3, Failure> {
            self.parent.hyperbola()
        }
        fn parabola(&self) -> Result<Parabola3, Failure> {
            self.parent.parabola()
        }
        fn degree(&self) -> Result<i32, Failure> {
            self.parent.degree()
        }
        fn is_rational(&self) -> Result<bool, Failure> {
            self.parent.is_rational()
        }
        fn nb_poles(&self) -> Result<i32, Failure> {
            self.parent.nb_poles()
        }
        fn nb_knots(&self) -> Result<i32, Failure> {
            self.parent.nb_knots()
        }
        fn bezier(&self) -> Result<Handle<GeomBezierCurve>, Failure> {
            self.parent.bezier()
        }
        fn bspline(&self) -> Result<Handle<GeomBSplineCurve>, Failure> {
            self.parent.bspline()
        }
        fn offset_curve(&self) -> Result<Handle<GeomOffsetCurve>, Failure> {
            self.parent.offset_curve()
        }
        fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
            self.parent.eval_d0(u)
        }
        fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
            self.parent.eval_d1(u)
        }
        fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
            self.parent.eval_d2(u)
        }
        fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
            self.parent.eval_d3(u)
        }
        fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
            self.parent.eval_dn(u, n)
        }
    };
}

/// The `Adaptor2d_Curve2d` members of `Geom2dAdaptor_Curve` that
/// `BRepAdaptor_Curve2d` inherits (all but `ShallowCopy`).
macro_rules! inherited_curve2d_members {
    () => {
        fn first_parameter(&self) -> Result<f64, Failure> {
            self.parent.first_parameter()
        }
        fn last_parameter(&self) -> Result<f64, Failure> {
            self.parent.last_parameter()
        }
        fn continuity(&self) -> Result<Shape, Failure> {
            self.parent.continuity()
        }
        fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
            self.parent.nb_intervals(shape)
        }
        fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
            self.parent.intervals(bounds, shape)
        }
        fn trim(
            &self,
            first: f64,
            last: f64,
            tolerance: f64,
        ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
            self.parent.trim(first, last, tolerance)
        }
        fn is_closed(&self) -> Result<bool, Failure> {
            self.parent.is_closed()
        }
        fn is_periodic(&self) -> Result<bool, Failure> {
            self.parent.is_periodic()
        }
        fn period(&self) -> Result<f64, Failure> {
            self.parent.period()
        }
        fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
            self.parent.resolution(r3d)
        }
        fn get_type(&self) -> Result<CurveType, Failure> {
            self.parent.get_type()
        }
        fn line(&self) -> Result<Line2, Failure> {
            self.parent.line()
        }
        fn circle(&self) -> Result<Circle2, Failure> {
            self.parent.circle()
        }
        fn ellipse(&self) -> Result<Ellipse2, Failure> {
            self.parent.ellipse()
        }
        fn hyperbola(&self) -> Result<Hyperbola2, Failure> {
            self.parent.hyperbola()
        }
        fn parabola(&self) -> Result<Parabola2, Failure> {
            self.parent.parabola()
        }
        fn degree(&self) -> Result<i32, Failure> {
            self.parent.degree()
        }
        fn is_rational(&self) -> Result<bool, Failure> {
            self.parent.is_rational()
        }
        fn nb_poles(&self) -> Result<i32, Failure> {
            self.parent.nb_poles()
        }
        fn nb_knots(&self) -> Result<i32, Failure> {
            self.parent.nb_knots()
        }
        fn nb_samples(&self) -> Result<i32, Failure> {
            self.parent.nb_samples()
        }
        fn bezier(&self) -> Result<Handle<Geom2dBezierCurve>, Failure> {
            self.parent.bezier()
        }
        fn bspline(&self) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
            self.parent.bspline()
        }
        fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
            self.parent.eval_d0(u)
        }
        fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
            self.parent.eval_d1(u)
        }
        fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
            self.parent.eval_d2(u)
        }
        fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
            self.parent.eval_d3(u)
        }
        fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
            self.parent.eval_dn(u, n)
        }
    };
}

/// The `Adaptor3d_Surface` members of `GeomAdaptor_TransformedSurface` that
/// `BRepAdaptor_Surface` inherits (all but `ShallowCopy`).
macro_rules! inherited_surface_members {
    () => {
        fn first_u_parameter(&self) -> Result<f64, Failure> {
            self.parent.first_u_parameter()
        }
        fn last_u_parameter(&self) -> Result<f64, Failure> {
            self.parent.last_u_parameter()
        }
        fn first_v_parameter(&self) -> Result<f64, Failure> {
            self.parent.first_v_parameter()
        }
        fn last_v_parameter(&self) -> Result<f64, Failure> {
            self.parent.last_v_parameter()
        }
        fn u_continuity(&self) -> Result<Shape, Failure> {
            self.parent.u_continuity()
        }
        fn v_continuity(&self) -> Result<Shape, Failure> {
            self.parent.v_continuity()
        }
        fn nb_u_intervals(&self, shape: Shape) -> Result<i32, Failure> {
            self.parent.nb_u_intervals(shape)
        }
        fn nb_v_intervals(&self, shape: Shape) -> Result<i32, Failure> {
            self.parent.nb_v_intervals(shape)
        }
        fn u_intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
            self.parent.u_intervals(bounds, shape)
        }
        fn v_intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
            self.parent.v_intervals(bounds, shape)
        }
        fn u_trim(
            &self,
            first: f64,
            last: f64,
            tolerance: f64,
        ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
            self.parent.u_trim(first, last, tolerance)
        }
        fn v_trim(
            &self,
            first: f64,
            last: f64,
            tolerance: f64,
        ) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
            self.parent.v_trim(first, last, tolerance)
        }
        fn is_u_closed(&self) -> Result<bool, Failure> {
            self.parent.is_u_closed()
        }
        fn is_v_closed(&self) -> Result<bool, Failure> {
            self.parent.is_v_closed()
        }
        fn is_u_periodic(&self) -> Result<bool, Failure> {
            self.parent.is_u_periodic()
        }
        fn u_period(&self) -> Result<f64, Failure> {
            self.parent.u_period()
        }
        fn is_v_periodic(&self) -> Result<bool, Failure> {
            self.parent.is_v_periodic()
        }
        fn v_period(&self) -> Result<f64, Failure> {
            self.parent.v_period()
        }
        fn u_resolution(&self, r3d: f64) -> Result<f64, Failure> {
            self.parent.u_resolution(r3d)
        }
        fn v_resolution(&self, r3d: f64) -> Result<f64, Failure> {
            self.parent.v_resolution(r3d)
        }
        fn get_type(&self) -> Result<SurfaceType, Failure> {
            self.parent.get_type()
        }
        fn plane(&self) -> Result<Plane3, Failure> {
            self.parent.plane()
        }
        fn cylinder(&self) -> Result<Cylinder3, Failure> {
            self.parent.cylinder()
        }
        fn cone(&self) -> Result<Cone3, Failure> {
            self.parent.cone()
        }
        fn sphere(&self) -> Result<Sphere3, Failure> {
            self.parent.sphere()
        }
        fn torus(&self) -> Result<Torus3, Failure> {
            self.parent.torus()
        }
        fn u_degree(&self) -> Result<i32, Failure> {
            self.parent.u_degree()
        }
        fn nb_u_poles(&self) -> Result<i32, Failure> {
            self.parent.nb_u_poles()
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
            self.parent.is_u_rational()
        }
        fn is_v_rational(&self) -> Result<bool, Failure> {
            self.parent.is_v_rational()
        }
        fn bezier(&self) -> Result<Handle<GeomBezierSurface>, Failure> {
            self.parent.bezier()
        }
        fn bspline(&self) -> Result<Handle<GeomBSplineSurface>, Failure> {
            self.parent.bspline()
        }
        fn axe_of_revolution(&self) -> Result<Axis1, Failure> {
            self.parent.axe_of_revolution()
        }
        fn direction(&self) -> Result<Direction3, Failure> {
            self.parent.direction()
        }
        fn basis_curve(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
            self.parent.basis_curve()
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
    };
}

pub(crate) use {inherited_curve_members, inherited_curve2d_members, inherited_surface_members};
