// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_MultiLine.hxx/.cxx (the
// constructors, the accessors and the AppCont_Function overrides) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_MultiLine`: the 3d curve and the two p-curves of the intersection of two
//! faces of an evolved shape, followed along a bisector, as an AppCont_Function.
mod curves;
pub use curves::MultiLineCurves;
mod isos;
mod value_on_face;

use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::app_cont::{Function, FunctionBase};
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity, SurfaceType};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::cell::RefCell;

/// `5 * Precision::Confusion()`: the tolerance of the particular cases and of the
/// degenerated isos.
pub(super) fn eps() -> f64 {
    5. * precision::confusion()
}

/// The intersection line of two faces along a bisector (`BRepFill_MultiLine`), the
/// function of one 3d point and two 2d points the approximation fits;
/// `let ml = BRepFillMultiLine::new(&f1, &f2, &e1, &e2, false, false, &bis)?;`.
pub struct BRepFillMultiLine {
    base: FunctionBase,
    my_face1: Face,
    my_face2: Face,
    my_u1: Geom2dAdaptorCurve,
    my_v1: Geom2dAdaptorCurve,
    my_u2: Geom2dAdaptorCurve,
    my_v2: Geom2dAdaptorCurve,
    my_iso_u1: bool,
    my_iso_u2: bool,
    my_bis: Geom2dAdaptorCurve,
    my_k_part: i32,
    my_cont: Continuity,
    // The first exception of Value (an AppCont_Function cannot raise it through the
    // fit); the caller returns it after the approximation.
    my_error: RefCell<Option<Failure>>,
}

impl Default for BRepFillMultiLine {
    /// `BRepFill_MultiLine()`: one 3d point and two 2d points, no face.
    fn default() -> Self {
        let mut base = FunctionBase::new();
        base.set_number_of_points(1, 2);
        Self {
            base,
            my_face1: Face::default(),
            my_face2: Face::default(),
            my_u1: Geom2dAdaptorCurve::default(),
            my_v1: Geom2dAdaptorCurve::default(),
            my_u2: Geom2dAdaptorCurve::default(),
            my_v2: Geom2dAdaptorCurve::default(),
            my_iso_u1: false,
            my_iso_u2: false,
            my_bis: Geom2dAdaptorCurve::default(),
            my_k_part: 0,
            my_cont: Continuity::C0,
            my_error: RefCell::new(None),
        }
    }
}

impl BRepFillMultiLine {
    /// `BRepFill_MultiLine(Face1, Face2, Edge1, Edge2, Inv1, Inv2, Bissec)`: the isos of
    /// both faces at their minimal parameters (the maximal ones for an inverted or
    /// degenerated edge), projected in the plane, and the particular cases: a line
    /// bisector along an iso (1, G1) or between two planes (2).
    pub fn new(
        face1: &Face,
        face2: &Face,
        edge1: &Edge,
        edge2: &Edge,
        inv1: bool,
        inv2: bool,
        bissec: &Handle<dyn Geom2dCurve>,
    ) -> Result<Self, Failure> {
        let mut ml = Self {
            my_face1: face1.clone(),
            my_face2: face2.clone(),
            ..Self::default()
        };
        ml.my_bis = Geom2dAdaptorCurve::with_curve(bissec)?;
        ml.my_iso_u1 = isos::is_iso_u(face1, edge1)?;
        ml.my_iso_u2 = isos::is_iso_u(face2, edge2)?;
        let side1 = isos::face_isos(face1, edge1, inv1, ml.my_iso_u1)?;
        let side2 = isos::face_isos(face2, edge2, inv2, ml.my_iso_u2)?;
        (ml.my_u1, ml.my_v1) = (side1.u, side1.v);
        (ml.my_u2, ml.my_v2) = (side2.u, side2.v);
        ml.particular_case(
            side1.surface_type == SurfaceType::Plane && side2.surface_type == SurfaceType::Plane,
        )?;
        Ok(ml)
    }

    /// The particular cases of a line bisector: along an iso of the first face (1,
    /// continuity G1), between two planes (2).
    fn particular_case(&mut self, both_planes: bool) -> Result<(), Failure> {
        self.my_cont = Continuity::C0;
        if self.my_bis.get_type()? != CurveType::Line {
            return Ok(());
        }
        let (first, last) = (
            self.my_bis.first_parameter()?,
            self.my_bis.last_parameter()?,
        );
        let delta_u: f64 = last - first;
        let a_pnt1: Point2 = self.value_on_f1(first + 0.1 * delta_u)?;
        let a_pnt2: Point2 = self.value_on_f1(first + 0.9 * delta_u)?;
        let along: bool = if self.my_iso_u1 {
            (a_pnt1.y() - a_pnt2.y()).abs() < eps()
        } else {
            (a_pnt1.x() - a_pnt2.x()).abs() < eps()
        };
        if along {
            self.my_k_part = 1;
            self.my_cont = Continuity::G1;
        }
        if both_planes {
            self.my_k_part = 2;
        }
        Ok(())
    }

    /// `IsParticularCase()`: whether the curves are computed exactly; `ml.is_particular_case()`.
    pub fn is_particular_case(&self) -> bool {
        self.my_k_part != 0
    }

    /// `Continuity()`: G1 along an iso, else C0; `ml.continuity()`.
    pub fn continuity(&self) -> Continuity {
        self.my_cont
    }

    /// `FirstParameter()`: of the bisector; `ml.first_parameter_of()?`.
    pub fn first_parameter_of(&self) -> Result<f64, Failure> {
        self.my_bis.first_parameter()
    }

    /// `LastParameter()`: of the bisector; `ml.last_parameter_of()?`.
    pub fn last_parameter_of(&self) -> Result<f64, Failure> {
        self.my_bis.last_parameter()
    }

    /// The first exception raised by `Value` through the AppCont_Function interface,
    /// taken; `ml.take_error()`.
    pub fn take_error(&self) -> Option<Failure> {
        self.my_error.borrow_mut().take()
    }

    /// Records the first exception of Value.
    fn record(&self, error: Failure) {
        let mut slot = self.my_error.borrow_mut();
        if slot.is_none() {
            *slot = Some(error);
        }
    }
}

impl Function for BRepFillMultiLine {
    fn base(&self) -> &FunctionBase {
        &self.base
    }

    /// `FirstParameter()` (a null bisector gives 0, natively a crash).
    fn first_parameter(&self) -> f64 {
        self.first_parameter_of().unwrap_or_else(|e| {
            self.record(e);
            0.
        })
    }

    /// `LastParameter()` (a null bisector gives 0, natively a crash).
    fn last_parameter(&self) -> f64 {
        self.last_parameter_of().unwrap_or_else(|e| {
            self.record(e);
            0.
        })
    }

    /// `Value(theU, thePnt2d, thePnt)`: the 3d point on the first face and the points
    /// on both faces; an exception is recorded (see `take_error`) and gives false.
    fn value(&self, u: f64, pnt2d: &mut Array1<'_, Point2>, pnt: &mut Array1<'_, Point3>) -> bool {
        let values = (|| -> Result<(Point3, Point2, Point2), Failure> {
            Ok((
                self.value_3d(u)?,
                self.value_on_f1(u)?,
                self.value_on_f2(u)?,
            ))
        })();
        match values {
            Ok((p3d, p1, p2)) => {
                pnt[1] = p3d;
                pnt2d[1] = p1;
                pnt2d[2] = p2;
                true
            }
            Err(error) => {
                self.record(error);
                false
            }
        }
    }

    /// `D1(theU, theVec2d, theVec)`: not given (false).
    fn d1(
        &self,
        _u: f64,
        _vec2d: &mut Array1<'_, Vector2>,
        _vec: &mut Array1<'_, Vector3>,
    ) -> bool {
        false
    }
}
