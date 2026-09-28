// Copyright (c) 2020 OPEN CASCADE SAS
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
// Rust translation of Extrema_GlobOptFuncCQuadric.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GlobOptFuncCQuadric`: `F(t)`, the square distance from `C(t)` to a
//! trimmed quadric (its projection when inside the trimming, and its corners), false
//! outside the curve range or for another surface.
use crate::extrema::failures::null_object;
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Plane3, Point3, Sphere3, Torus3};
use rustycad_tkmath::math::{MultipleVarFunction, Vector};
use std::f64::consts::PI;

/// The quadric of a function, `None` for another surface type (`mySType`).
#[derive(Clone, Copy, Debug)]
enum Quadric {
    Plane(Plane3),
    Cylinder(Cylinder3),
    Cone(Cone3),
    Sphere(Sphere3),
    Torus(Torus3),
}

impl Quadric {
    // `ElSLib::Parameters(quadric, P, u, v)`.
    fn parameters(&self, point: Point3) -> (f64, f64) {
        match self {
            Quadric::Plane(s) => elslib::parameters(s, point),
            Quadric::Cylinder(s) => elslib::parameters(s, point),
            Quadric::Cone(s) => elslib::parameters(s, point),
            Quadric::Sphere(s) => elslib::parameters(s, point),
            Quadric::Torus(s) => elslib::parameters(s, point),
        }
    }
}

/// The trimmed quadric of LoadQuad.
struct LoadedQuadric<'a> {
    s: &'a dyn Adaptor3dSurface,
    quadric: Option<Quadric>,
    p_trim: [Point3; 4],
    uf: f64,
    ul: f64,
    vf: f64,
    vl: f64,
}

/// `Extrema_GlobOptFuncCQuadric`; `GlobOptFuncCQuadric::with_surface(&curve, &surface)?`.
pub struct GlobOptFuncCQuadric<'a> {
    c: &'a dyn Adaptor3dCurve,
    loaded: Option<LoadedQuadric<'a>>,
    tf: f64,
    tl: f64,
    failure: FailureCapture,
}

// The periodic range brought into `[0, 2 PI]` as LoadQuad does.
fn adjust_range(periodic: bool, (first, last): (f64, f64)) -> (f64, f64) {
    if !periodic {
        return (first, last);
    }
    let t_max: f64 = 2. * PI + precision::p_confusion();
    if first > t_max || first < -precision::p_confusion() || (last - first).abs() > t_max {
        let tolerance: f64 = std_min((last - first).abs() / 2., precision::p_confusion());
        return elclib::adjust_periodic(0., 2. * PI, tolerance, first, last);
    }
    (first, last)
}

impl<'a> GlobOptFuncCQuadric<'a> {
    /// `Extrema_GlobOptFuncCQuadric(C, theTf, theTl)`: no quadric until LoadQuad
    /// (the native members then read an unset pointer);
    /// `GlobOptFuncCQuadric::with_range(&curve, 0.0, 1.0)`.
    pub fn with_range(curve: &'a dyn Adaptor3dCurve, tf: f64, tl: f64) -> Self {
        Self {
            c: curve,
            loaded: None,
            tf,
            tl,
            failure: FailureCapture::default(),
        }
    }
    /// `Extrema_GlobOptFuncCQuadric(C)`: the curve range, no quadric;
    /// `GlobOptFuncCQuadric::new(&curve)?`.
    pub fn new(curve: &'a dyn Adaptor3dCurve) -> Result<Self, Failure> {
        Ok(Self::with_range(curve, curve.first_parameter()?, curve.last_parameter()?))
    }
    /// `Extrema_GlobOptFuncCQuadric(C, S)`: the curve range and the whole quadric;
    /// `GlobOptFuncCQuadric::with_surface(&curve, &surface)?`.
    pub fn with_surface(curve: &'a dyn Adaptor3dCurve, surface: &'a dyn Adaptor3dSurface) -> Result<Self, Failure> {
        let mut function: Self = Self::new(curve)?;
        let u = (surface.first_u_parameter()?, surface.last_u_parameter()?);
        let v = (surface.first_v_parameter()?, surface.last_v_parameter()?);
        function.load_quad(surface, u, v)?;
        Ok(function)
    }
    /// `LoadQuad(S, theUf, theUl, theVf, theVl)`: the bounds (brought into
    /// `[0, 2 PI]` in a periodic direction), the corners and the quadric;
    /// `function.load_quad(&surface, (0., 1.), (0., 1.))?`.
    pub fn load_quad(&mut self, surface: &'a dyn Adaptor3dSurface, u: (f64, f64), v: (f64, f64)) -> Result<(), Failure> {
        let (uf, ul) = adjust_range(surface.is_u_periodic()?, u);
        let (vf, vl) = adjust_range(surface.is_v_periodic()?, v);
        let p_trim: [Point3; 4] = [
            surface.value(uf, vf)?,
            surface.value(ul, vf)?,
            surface.value(ul, vl)?,
            surface.value(uf, vl)?,
        ];
        let quadric: Option<Quadric> = match surface.get_type()? {
            SurfaceType::Plane => Some(Quadric::Plane(surface.plane()?)),
            SurfaceType::Cylinder => Some(Quadric::Cylinder(surface.cylinder()?)),
            SurfaceType::Cone => Some(Quadric::Cone(surface.cone()?)),
            SurfaceType::Sphere => Some(Quadric::Sphere(surface.sphere()?)),
            SurfaceType::Torus => Some(Quadric::Torus(surface.torus()?)),
            _ => None,
        };
        self.loaded = Some(LoadedQuadric { s: surface, quadric, p_trim, uf, ul, vf, vl });
        Ok(())
    }
    /// `NbVariables()`: 1; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        1
    }
    fn loaded(&self) -> Result<&LoadedQuadric<'a>, Failure> {
        self.loaded.as_ref().ok_or_else(|| null_object("Extrema_GlobOptFuncCQuadric - no quadric loaded"))
    }
    // The quadric parameters of `cp`, shifted by 2 PI beyond a 2 PI bound; None for
    // another surface.
    fn projection(&self, cp: Point3) -> Result<Option<(f64, f64)>, Failure> {
        let loaded = self.loaded()?;
        let Some(quadric) = loaded.quadric else {
            return Ok(None);
        };
        let (mut u, mut v) = quadric.parameters(cp);
        if !matches!(quadric, Quadric::Plane(_)) && loaded.ul > 2. * PI + precision::p_confusion() {
            u += 2. * PI;
        }
        if matches!(quadric, Quadric::Torus(_)) && loaded.vl > 2. * PI + precision::p_confusion() {
            v += 2. * PI;
        }
        Ok(Some((u, v)))
    }
    // The square distance to the projection when inside the bounds, else RealLast.
    fn inside_distance(&self, cp: Point3, (u, v): (f64, f64)) -> Result<Option<f64>, Failure> {
        let loaded = self.loaded()?;
        if u >= loaded.uf && u <= loaded.ul && v >= loaded.vf && v <= loaded.vl {
            return Ok(Some(cp.square_distance(&loaded.s.value(u, v)?)));
        }
        Ok(None)
    }
    // `value(ct, F)`.
    fn value_at(&self, ct: f64) -> Result<f64, Failure> {
        let cp: Point3 = self.c.value(ct)?;
        let Some(uv) = self.projection(cp)? else {
            return Ok(precision::infinite());
        };
        let mut f: f64 = f64::MAX;
        if let Some(d) = self.inside_distance(cp, uv)? {
            f = std_min(f, d);
        }
        for corner in &self.loaded()?.p_trim {
            f = std_min(f, cp.square_distance(corner));
        }
        Ok(f)
    }
    /// `Value(X, F)`: false outside the curve range or for an infinite value;
    /// `function.value(&x, &mut f)?`.
    pub fn value(&self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
        let ct: f64 = x[x.lower()];
        if !(ct >= self.tf && ct <= self.tl) {
            return Ok(false);
        }
        *f = self.value_at(ct)?;
        Ok(!precision::is_infinite(*f))
    }
    /// `QuadricParameters(theCT, theUV)`: the parameters of the point of the trimmed
    /// quadric nearest to `C(t)` (a corner when it is nearer); for another surface
    /// `(Uf, Ul)` as natively; `function.quadric_parameters(&t, &mut uv)?`.
    pub fn quadric_parameters(&self, ct: &Vector<'_>, uv: &mut Vector<'_>) -> Result<(), Failure> {
        let loaded = self.loaded()?;
        let uext: [f64; 4] = [loaded.uf, loaded.ul, loaded.ul, loaded.uf];
        let vext: [f64; 4] = [loaded.vf, loaded.vf, loaded.vl, loaded.vl];
        let cp: Point3 = self.c.value(ct[1])?;
        let Some((mut u, mut v)) = self.projection(cp)? else {
            (uv[1], uv[2]) = (loaded.uf, loaded.ul);
            return Ok(());
        };
        let mut f: f64 = f64::MAX;
        if let Some(d) = self.inside_distance(cp, (u, v))? {
            f = d;
        }
        for i in 0..4 {
            let fi: f64 = cp.square_distance(&loaded.p_trim[i]);
            if fi < f {
                f = fi;
                (u, v) = (uext[i], vext[i]);
            }
        }
        (uv[1], uv[2]) = (u, v);
        Ok(())
    }
    /// The first failure a math callback kept since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

impl MultipleVarFunction for GlobOptFuncCQuadric<'_> {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let result = GlobOptFuncCQuadric::value(self, point, value);
        self.failure.keep(result).unwrap_or(false)
    }
}
