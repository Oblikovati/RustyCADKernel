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
// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_Projector.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_Projector`: the root of the elementary projectors, which stores the
//! projected 2D curve (a gp conic, a line, or a Bezier or B-spline handle), its
//! kind, the periodic flag and the done flag.
use crate::native_failure::failure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Circle2, Circle3, Ellipse2, Ellipse3, Hyperbola2, Hyperbola3, Line2, Line3, Parabola2,
    Parabola3, Point2, Vector2,
};

/// `ProjLib_Projector`: the result of an elementary projection.
///
/// The native constructor documents the type as OtherCurve but sets
/// BSplineCurve; the port keeps the native value (docs/tkgeombase-progress.md).
///
/// ```
/// use rustycad_tkgeombase::proj_lib::Projector;
/// use rustycad_tkmath::geom_abs::CurveType;
/// let projector = Projector::new();
/// assert_eq!(projector.get_type(), CurveType::BSplineCurve);
/// assert!(!projector.is_done());
/// ```
#[derive(Clone)]
pub struct Projector {
    pub(super) my_type: CurveType,
    pub(super) lin: Line2,
    pub(super) circ: Circle2,
    pub(super) elips: Ellipse2,
    pub(super) hypr: Hyperbola2,
    pub(super) parab: Parabola2,
    pub(super) bspline: Handle<Geom2dBSplineCurve>,
    pub(super) bezier: Handle<Geom2dBezierCurve>,
    pub(super) is_periodic: bool,
    pub(super) is_done: bool,
}

impl Default for Projector {
    fn default() -> Self {
        Self {
            my_type: CurveType::BSplineCurve,
            lin: Line2::default(),
            circ: Circle2::default(),
            elips: Ellipse2::default(),
            hypr: Hyperbola2::default(),
            parab: Parabola2::default(),
            bspline: Handle::default(),
            bezier: Handle::default(),
            is_periodic: false,
            is_done: false,
        }
    }
}

impl Projector {
    /// `ProjLib_Projector()`: not done, type BSplineCurve; `Projector::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IsDone()`; `projector.is_done()`.
    pub fn is_done(&self) -> bool {
        self.is_done
    }
    /// `Done()`: sets the done flag; `projector.done()`.
    pub fn done(&mut self) {
        self.is_done = true;
    }
    /// `GetType()`; `projector.get_type()`.
    pub fn get_type(&self) -> CurveType {
        self.my_type
    }
    /// `SetType(Type)`; `projector.set_type(CurveType::Line)`.
    pub fn set_type(&mut self, curve_type: CurveType) {
        self.my_type = curve_type;
    }
    /// `IsPeriodic()`; `projector.is_periodic()`.
    pub fn is_periodic(&self) -> bool {
        self.is_periodic
    }
    /// `SetPeriodic()`: sets the periodic flag; `projector.set_periodic()`.
    pub fn set_periodic(&mut self) {
        self.is_periodic = true;
    }
    /// `SetBSpline(C)`; `projector.set_bspline(&curve)`.
    pub fn set_bspline(&mut self, curve: &Handle<Geom2dBSplineCurve>) {
        self.bspline = curve.clone();
    }
    /// `SetBezier(C)`; `projector.set_bezier(&curve)`.
    pub fn set_bezier(&mut self, curve: &Handle<Geom2dBezierCurve>) {
        self.bezier = curve.clone();
    }
    /// `BSpline()`: the stored handle, possibly null; `projector.bspline()`.
    pub fn bspline(&self) -> Handle<Geom2dBSplineCurve> {
        self.bspline.clone()
    }
    /// `Bezier()`: the stored handle, possibly null; `projector.bezier()`.
    pub fn bezier(&self) -> Handle<Geom2dBezierCurve> {
        self.bezier.clone()
    }
}

// The typed accessors raise Standard_NoSuchObject unless the type matches.
fn check_type(actual: CurveType, expected: CurveType, member: &str) -> Result<(), Failure> {
    if actual == expected {
        return Ok(());
    }
    Err(failure(
        "Standard_NoSuchObject",
        &format!(
            "ProjLib_Projector::{member}: the projection is {actual:?}; expected {expected:?}"
        ),
    ))
}

impl Projector {
    /// `Line()`: Standard_NoSuchObject unless the type is Line; `projector.line()?`.
    pub fn line(&self) -> Result<Line2, Failure> {
        check_type(self.my_type, CurveType::Line, "Line")?;
        Ok(self.lin)
    }
    /// `Circle()`: Standard_NoSuchObject unless the type is Circle; `projector.circle()?`.
    pub fn circle(&self) -> Result<Circle2, Failure> {
        check_type(self.my_type, CurveType::Circle, "Circle")?;
        Ok(self.circ)
    }
    /// `Ellipse()`: Standard_NoSuchObject unless the type is Ellipse; `projector.ellipse()?`.
    pub fn ellipse(&self) -> Result<Ellipse2, Failure> {
        check_type(self.my_type, CurveType::Ellipse, "Ellipse")?;
        Ok(self.elips)
    }
    /// `Hyperbola()`: Standard_NoSuchObject unless the type is Hyperbola;
    /// `projector.hyperbola()?`.
    pub fn hyperbola(&self) -> Result<Hyperbola2, Failure> {
        check_type(self.my_type, CurveType::Hyperbola, "Hyperbola")?;
        Ok(self.hypr)
    }
    /// `Parabola()`: Standard_NoSuchObject unless the type is Parabola;
    /// `projector.parabola()?`.
    pub fn parabola(&self) -> Result<Parabola2, Failure> {
        check_type(self.my_type, CurveType::Parabola, "Parabola")?;
        Ok(self.parab)
    }

    /// `UFrame(CFirst, CLast, UFirst, Period)`: translates a line result in U so that
    /// its point of parameter `c_first` lies in `[u_first, u_first + period[`; other
    /// kinds are left unchanged. `c_last` is unused, as natively;
    /// `projector.u_frame(0.0, 1.0, 0.0, 2.0 * PI)`.
    pub fn u_frame(&mut self, c_first: f64, _c_last: f64, u_first: f64, period: f64) {
        if self.my_type != CurveType::Line {
            return;
        }
        let u: f64 = elclib::value(c_first, &self.lin).x();
        let new_u: f64 = elclib::in_period(u, u_first, u_first + period);
        self.lin.translate(Vector2::new(new_u - u, 0.));
    }

    /// `VFrame(CFirst, CLast, VFirst, Period)`: the V twin of [`Self::u_frame`];
    /// `projector.v_frame(0.0, 1.0, 0.0, 2.0 * PI)`.
    pub fn v_frame(&mut self, c_first: f64, _c_last: f64, v_first: f64, period: f64) {
        if self.my_type != CurveType::Line {
            return;
        }
        let first: Point2 = elclib::value(c_first, &self.lin);
        let v: f64 = first.y();
        let new_v: f64 = elclib::in_period(v, v_first, v_first + period);
        self.lin.translate(Vector2::new(0., new_v - v));
    }
}

/// The virtual `Project` overloads of `ProjLib_Projector` and its elementary
/// subclasses. The base class sets the type to OtherCurve for every conic; each
/// subclass overrides the kinds it knows in closed form. A projection returns the
/// native exception (a null gp direction, for example) as a `Failure`.
///
/// ```
/// use rustycad_tkgeombase::proj_lib::{Project, Projector};
/// use rustycad_tkmath::geom_abs::CurveType;
/// use rustycad_tkmath::gp::Line3;
/// let mut projector = Projector::new();
/// projector.project_line(&Line3::default()).unwrap();
/// assert_eq!(projector.projector().get_type(), CurveType::OtherCurve);
/// ```
pub trait Project {
    /// The stored projection (the base class object); `projection.projector()`.
    fn projector(&self) -> &Projector;
    /// `Project(const gp_Lin&)`; `projection.project_line(&line)?`.
    fn project_line(&mut self, line: &Line3) -> Result<(), Failure>;
    /// `Project(const gp_Circ&)`; `projection.project_circle(&circle)?`.
    fn project_circle(&mut self, circle: &Circle3) -> Result<(), Failure>;
    /// `Project(const gp_Elips&)`; `projection.project_ellipse(&ellipse)?`.
    fn project_ellipse(&mut self, ellipse: &Ellipse3) -> Result<(), Failure>;
    /// `Project(const gp_Parab&)`; `projection.project_parabola(&parabola)?`.
    fn project_parabola(&mut self, parabola: &Parabola3) -> Result<(), Failure>;
    /// `Project(const gp_Hypr&)`; `projection.project_hyperbola(&hyperbola)?`.
    fn project_hyperbola(&mut self, hyperbola: &Hyperbola3) -> Result<(), Failure>;
}

impl Project for Projector {
    fn projector(&self) -> &Projector {
        self
    }
    fn project_line(&mut self, _line: &Line3) -> Result<(), Failure> {
        self.my_type = CurveType::OtherCurve;
        Ok(())
    }
    fn project_circle(&mut self, _circle: &Circle3) -> Result<(), Failure> {
        self.my_type = CurveType::OtherCurve;
        Ok(())
    }
    fn project_ellipse(&mut self, _ellipse: &Ellipse3) -> Result<(), Failure> {
        self.my_type = CurveType::OtherCurve;
        Ok(())
    }
    fn project_parabola(&mut self, _parabola: &Parabola3) -> Result<(), Failure> {
        self.my_type = CurveType::OtherCurve;
        Ok(())
    }
    fn project_hyperbola(&mut self, _hyperbola: &Hyperbola3) -> Result<(), Failure> {
        self.my_type = CurveType::OtherCurve;
        Ok(())
    }
}

/// Implements `Deref<Target = Projector>`, `DerefMut` and the base-class `Project`
/// members for the conics an elementary projector does not override.
macro_rules! projector_base {
    ($projector:ty) => {
        impl std::ops::Deref for $projector {
            type Target = $crate::proj_lib::Projector;
            fn deref(&self) -> &Self::Target {
                &self.base
            }
        }
        impl std::ops::DerefMut for $projector {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.base
            }
        }
        impl From<$projector> for $crate::proj_lib::Projector {
            // The native `myResult = P` slices the subclass to its base object.
            fn from(projector: $projector) -> Self {
                projector.base
            }
        }
    };
}
pub(super) use projector_base;
