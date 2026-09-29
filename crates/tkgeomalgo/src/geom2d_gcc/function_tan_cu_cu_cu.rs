// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of Geom2dGcc_FunctionTanCuCuCu.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_FunctionTanCuCuCu`: the system of a circle tangent to three curves.
use super::{CurveTool, EvalFailure, Type1, null_object};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Circle2, Line2, Point2, Vector2, Xy};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// The points, tangents and second derivatives of the three arguments
/// (`InitDerivative` outputs), each as `(Point, Tan, D2)`.
pub type TanCuCuCuDerivatives = [(Point2, Vector2, Vector2); 3];

/// `Geom2dGcc_FunctionTanCuCuCu`: the three equations
/// `s_i n_i . p_ij + s_j n_j . p_ij = 0` on the unit normals `n_i` of the arguments
/// at `X(i)` and the unit chords `p_ij`, signed towards the centroid of the three
/// points; a `math_FunctionSetWithDerivatives` of 3 variables and 3 equations.
///
/// Natively `Derivatives` and `Values` return true without writing anything, so a
/// solver on this function keeps the Jacobian and residuals it started with.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::FunctionTanCuCuCu;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Line2, Point2};
/// use rustycad_tkmath::math::{FunctionSet, Vector};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 4.0), Direction2::new(1.0, 0.0)?)).cast();
/// let adaptor = Geom2dAdaptorCurve::with_curve(&line)?;
/// let l1 = Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let l2 = Line2::new(Point2::new(0.0, 0.0), Direction2::new(0.0, 1.0)?);
/// let mut f = FunctionTanCuCuCu::lin_lin_curve(l1, l2, &adaptor);
/// assert_eq!(f.variables(), 3);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct FunctionTanCuCuCu<'a> {
    curv1: Option<&'a Geom2dAdaptorCurve>,
    curv2: Option<&'a Geom2dAdaptorCurve>,
    curv3: &'a Geom2dAdaptorCurve,
    circ1: Circle2,
    circ2: Circle2,
    lin1: Line2,
    lin2: Line2,
    the_type: Type1,
    failure: EvalFailure,
}

impl<'a> FunctionTanCuCuCu<'a> {
    fn with_type(the_type: Type1, curv3: &'a Geom2dAdaptorCurve) -> Self {
        Self {
            curv1: None,
            curv2: None,
            curv3,
            circ1: Circle2::default(),
            circ2: Circle2::default(),
            lin1: Line2::default(),
            lin2: Line2::default(),
            the_type,
            failure: EvalFailure::default(),
        }
    }

    /// `Geom2dGcc_FunctionTanCuCuCu(C1, C2, C3)` on three curves;
    /// `FunctionTanCuCuCu::new(&a1, &a2, &a3)`.
    pub fn new(
        c1: &'a Geom2dAdaptorCurve,
        c2: &'a Geom2dAdaptorCurve,
        c3: &'a Geom2dAdaptorCurve,
    ) -> Self {
        let mut f: Self = Self::with_type(Type1::CuCuCu, c3);
        (f.curv1, f.curv2) = (Some(c1), Some(c2));
        f
    }

    /// `Geom2dGcc_FunctionTanCuCuCu(gp_Circ2d C1, C2, C3)`;
    /// `FunctionTanCuCuCu::circ_curve_curve(circle, &a2, &a3)`.
    pub fn circ_curve_curve(
        c1: Circle2,
        c2: &'a Geom2dAdaptorCurve,
        c3: &'a Geom2dAdaptorCurve,
    ) -> Self {
        let mut f: Self = Self::with_type(Type1::CiCuCu, c3);
        (f.circ1, f.curv2) = (c1, Some(c2));
        f
    }

    /// `Geom2dGcc_FunctionTanCuCuCu(gp_Circ2d C1, gp_Circ2d C2, C3)`;
    /// `FunctionTanCuCuCu::circ_circ_curve(c1, c2, &a3)`.
    pub fn circ_circ_curve(c1: Circle2, c2: Circle2, c3: &'a Geom2dAdaptorCurve) -> Self {
        let mut f: Self = Self::with_type(Type1::CiCiCu, c3);
        (f.circ1, f.circ2) = (c1, c2);
        f
    }

    /// `Geom2dGcc_FunctionTanCuCuCu(gp_Circ2d C1, gp_Lin2d L2, C3)`;
    /// `FunctionTanCuCuCu::circ_lin_curve(c1, l2, &a3)`.
    pub fn circ_lin_curve(c1: Circle2, l2: Line2, c3: &'a Geom2dAdaptorCurve) -> Self {
        let mut f: Self = Self::with_type(Type1::CiLiCu, c3);
        (f.circ1, f.lin2) = (c1, l2);
        f
    }

    /// `Geom2dGcc_FunctionTanCuCuCu(gp_Lin2d L1, gp_Lin2d L2, C3)`;
    /// `FunctionTanCuCuCu::lin_lin_curve(l1, l2, &a3)`.
    pub fn lin_lin_curve(l1: Line2, l2: Line2, c3: &'a Geom2dAdaptorCurve) -> Self {
        let mut f: Self = Self::with_type(Type1::LiLiCu, c3);
        (f.lin1, f.lin2) = (l1, l2);
        f
    }

    /// `Geom2dGcc_FunctionTanCuCuCu(gp_Lin2d L1, C2, C3)`;
    /// `FunctionTanCuCuCu::lin_curve_curve(l1, &a2, &a3)`.
    pub fn lin_curve_curve(
        l1: Line2,
        c2: &'a Geom2dAdaptorCurve,
        c3: &'a Geom2dAdaptorCurve,
    ) -> Self {
        let mut f: Self = Self::with_type(Type1::LiCuCu, c3);
        (f.lin1, f.curv2) = (l1, Some(c2));
        f
    }

    /// The exception a curve evaluation raised natively, after a member answered
    /// `false`; `f.raised()?`.
    pub fn raised(&self) -> Result<(), Failure> {
        self.failure.check()
    }

    /// `InitDerivative(X, Point1, Point2, Point3, Tan1, Tan2, Tan3, D21, D22, D23)`:
    /// a line has a null second derivative; `let [a1, a2, a3] = f.init_derivative(&x)?;`.
    pub fn init_derivative(&self, x: &Vector<'_>) -> Result<TanCuCuCuDerivatives, Failure> {
        let (u1, u2, u3) = (*x.value(1)?, *x.value(2)?, *x.value(3)?);
        let first = match self.the_type {
            Type1::CiCuCu | Type1::CiCiCu | Type1::CiLiCu => elclib::d2(u1, &self.circ1),
            Type1::LiCuCu | Type1::LiLiCu => line_d2(u1, &self.lin1),
            Type1::CuCuCu => curve_d2(self.curv1, u1)?,
        };
        let second = match self.the_type {
            Type1::CiCiCu => elclib::d2(u2, &self.circ2),
            Type1::CiLiCu | Type1::LiLiCu => line_d2(u2, &self.lin2),
            _ => curve_d2(self.curv2, u2)?,
        };
        Ok([first, second, CurveTool::d2(self.curv3, u3)?])
    }

    // Fval(1..3).
    fn write_values(&self, x: &Vector<'_>, fval: &mut Vector<'_>) -> Result<(), Failure> {
        let [(point1, tan1, _), (point2, tan2, _), (point3, tan3, _)] = self.init_derivative(x)?;
        // pipj (normes) et PiPj (non Normes).
        let p1p2: Xy = unit_or_zero(Vector2::between(point1, point2).xy());
        let p2p3: Xy = unit_or_zero(Vector2::between(point2, point3).xy());
        let p3p1: Xy = unit_or_zero(Vector2::between(point3, point1).xy());
        // normales aux courbes.
        let nor1: Xy = unit_or_zero(Xy::new(-tan1.y(), tan1.x()));
        let nor2: Xy = unit_or_zero(Xy::new(-tan2.y(), tan2.x()));
        let nor3: Xy = unit_or_zero(Xy::new(-tan3.y(), tan3.x()));
        // determination des signes pour les produits scalaires.
        let pcenter: Xy = (point1.xy() + point2.xy() + point3.xy()) / 3.0;
        let signe1: f64 = sign_towards(nor1, pcenter - point1.xy());
        let signe2: f64 = sign_towards(nor2, pcenter - point2.xy());
        let signe3: f64 = sign_towards(nor3, pcenter - point3.xy());
        *fval.change_value(1)? = signe1 * nor1.dot(p1p2) + signe2 * nor2.dot(p1p2);
        *fval.change_value(2)? = signe2 * nor2.dot(p2p3) + signe3 * nor3.dot(p2p3);
        *fval.change_value(3)? = signe3 * nor3.dot(p3p1) + signe1 * nor1.dot(p3p1);
        Ok(())
    }
}

// `ElCLib::D1(U, L, P, T)` with the second derivative set to (0, 0).
pub(super) fn line_d2(u: f64, line: &Line2) -> (Point2, Vector2, Vector2) {
    let (point, tan) = elclib::d1(u, line);
    (point, tan, Vector2::new(0.0, 0.0))
}

// `Geom2dGcc_CurveTool::D2` of a curve slot; a slot the type does not fill is the
// native null handle.
pub(super) fn curve_d2(
    curve: Option<&Geom2dAdaptorCurve>,
    u: f64,
) -> Result<(Point2, Vector2, Vector2), Failure> {
    let curve = curve.ok_or_else(|| null_object("Geom2dGcc_CurveTool::D2"))?;
    CurveTool::d2(curve, u)
}

// `v / |v|`, or (0, 0) below gp::Resolution().
fn unit_or_zero(v: Xy) -> Xy {
    let norm: f64 = v.modulus();
    if norm >= gp::resolution() {
        return v / norm;
    }
    Xy::new(0.0, 0.0)
}

// -1 when the normal points away from (or across) the centroid, else 1.
fn sign_towards(normal: Xy, fic: Xy) -> f64 {
    if normal.dot(fic) <= 0.0 { -1.0 } else { 1.0 }
}

impl FunctionSet for FunctionTanCuCuCu<'_> {
    fn variables(&self) -> i32 {
        3
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        let result = self.write_values(x, f);
        self.failure.keep(result).is_some()
    }
}

impl FunctionSetWithDerivatives for FunctionTanCuCuCu<'_> {
    // Natively empty: returns true and writes nothing.
    fn derivatives(&mut self, _x: &Vector<'_>, _deriv: &mut Matrix<'_>) -> bool {
        true
    }
    // Natively empty: returns true and writes nothing.
    fn values(&mut self, _x: &Vector<'_>, _f: &mut Vector<'_>, _deriv: &mut Matrix<'_>) -> bool {
        true
    }
}
