// Rust translation of Blend_Function.hxx/.cxx, Blend_SurfRstFunction.hxx/.cxx and
// Blend_RstRstFunction.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The blend functions between two surfaces, a surface and a restriction, and two
//! restrictions, with the member bodies their abstract classes define.
use super::{BlendAppFunction, DecrochStatus};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::Vector;

/// The tangents and normals at both ends of a section (the `Tangent` output arguments).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SectionEndFrames {
    /// `TgFirst`: the tangent at the first extremity.
    pub tg_first: Vector3,
    /// `TgLast`: the tangent at the last extremity.
    pub tg_last: Vector3,
    /// `NormFirst`: the normal of the first surface at the first extremity.
    pub norm_first: Vector3,
    /// `NormLast`: the normal of the second surface at the last extremity.
    pub norm_last: Vector3,
}

/// `Blend_Function`: a blend function between two surfaces, whose variables are
/// `(U1, V1, U2, V2)`.
///
/// The members `Blend_Function` defines over `Blend_AppFunction` are the free functions of
/// this module ([`function_nb_variables`], [`function_pnt1`], [`function_pnt2`],
/// [`function_section_d2`]), which the implementations call.
pub trait BlendFunction: BlendAppFunction {
    /// `PointOnS1()`: the point on the first surface at the last solution.
    fn point_on_s1(&self) -> Point3;
    /// `PointOnS2()`: the point on the second surface at the last solution.
    fn point_on_s2(&self) -> Point3;
    /// `IsTangencyPoint()`: true when the tangents cannot be computed.
    fn is_tangency_point(&self) -> bool;
    /// `TangentOnS1()`: the 3d tangent at the first point.
    fn tangent_on_s1(&self) -> Result<Vector3, Failure>;
    /// `Tangent2dOnS1()`: the tangent in the parametric space of the first surface.
    fn tangent2d_on_s1(&self) -> Result<Vector2, Failure>;
    /// `TangentOnS2()`: the 3d tangent at the second point.
    fn tangent_on_s2(&self) -> Result<Vector3, Failure>;
    /// `Tangent2dOnS2()`: the tangent in the parametric space of the second surface.
    fn tangent2d_on_s2(&self) -> Result<Vector2, Failure>;
    /// `Tangent(U1, V1, U2, V2, TgFirst, TgLast, NormFirst, NormLast)`: the tangents of
    /// the section at both ends and the normals of the surfaces there.
    fn tangent(&self, u1: f64, v1: f64, u2: f64, v2: f64) -> Result<SectionEndFrames, Failure>;
    /// `TwistOnS1()`: false unless overridden; `function.twist_on_s1()?`.
    fn twist_on_s1(&self) -> Result<bool, Failure> {
        Ok(false)
    }
    /// `TwistOnS2()`: false unless overridden; `function.twist_on_s2()?`.
    fn twist_on_s2(&self) -> Result<bool, Failure> {
        Ok(false)
    }
}

/// `Blend_Function::NbVariables()`: 4; `function_nb_variables() == 4`.
pub fn function_nb_variables() -> i32 {
    4
}

/// `Blend_Function::Pnt1()`: `PointOnS1()`; `function_pnt1(&f)`.
pub fn function_pnt1<F: BlendFunction + ?Sized>(function: &F) -> Point3 {
    function.point_on_s1()
}

/// `Blend_Function::Pnt2()`: `PointOnS2()`; `function_pnt2(&f)`.
pub fn function_pnt2<F: BlendFunction + ?Sized>(function: &F) -> Point3 {
    function.point_on_s2()
}

/// `Blend_Function::Section(P, ..., D2Weigths)`: the second derivatives are not computed;
/// `function_section_d2() == false`.
pub fn function_section_d2() -> bool {
    false
}

/// `Blend_SurfRstFunction`: a blend function between a surface and a restriction (a
/// curve on another surface).
pub trait BlendSurfRstFunction: BlendAppFunction {
    /// `PointOnS()`: the point on the surface.
    fn point_on_s(&self) -> Point3;
    /// `PointOnRst()`: the point on the restriction.
    fn point_on_rst(&self) -> Point3;
    /// `Pnt2dOnS()`: the parameters of the point on the surface.
    fn pnt2d_on_s(&self) -> Point2;
    /// `Pnt2dOnRst()`: the parameters of the point on the surface of the restriction.
    fn pnt2d_on_rst(&self) -> Point2;
    /// `ParameterOnRst()`: the parameter on the restriction.
    fn parameter_on_rst(&self) -> f64;
    /// `IsTangencyPoint()`.
    fn is_tangency_point(&self) -> bool;
    /// `TangentOnS()`.
    fn tangent_on_s(&self) -> Result<Vector3, Failure>;
    /// `Tangent2dOnS()`.
    fn tangent2d_on_s(&self) -> Result<Vector2, Failure>;
    /// `TangentOnRst()`.
    fn tangent_on_rst(&self) -> Result<Vector3, Failure>;
    /// `Tangent2dOnRst()`.
    fn tangent2d_on_rst(&self) -> Result<Vector2, Failure>;
    /// `Decroch(Sol, NS, TgS)`: whether the section leaves the surface at `sol`, with the
    /// normal and the tangent there.
    fn decroch(&self, sol: &Vector<'_>) -> Result<(bool, Vector3, Vector3), Failure>;
}

/// `Blend_SurfRstFunction::Pnt1()`: `PointOnS()`; `surf_rst_pnt1(&f)`.
pub fn surf_rst_pnt1<F: BlendSurfRstFunction + ?Sized>(function: &F) -> Point3 {
    function.point_on_s()
}

/// `Blend_SurfRstFunction::Pnt2()`: `PointOnRst()`; `surf_rst_pnt2(&f)`.
pub fn surf_rst_pnt2<F: BlendSurfRstFunction + ?Sized>(function: &F) -> Point3 {
    function.point_on_rst()
}

/// `Blend_SurfRstFunction::GetMinimalDistance()`: Standard_NotImplemented;
/// `surf_rst_minimal_distance()`.
pub fn surf_rst_minimal_distance() -> Result<f64, Failure> {
    Err(not_implemented("Blend_SurfRstFunction::GetMinimalDistance"))
}

/// `Blend_RstRstFunction`: a blend function between two restrictions.
pub trait BlendRstRstFunction: BlendAppFunction {
    /// `PointOnRst1()`.
    fn point_on_rst1(&self) -> Point3;
    /// `PointOnRst2()`.
    fn point_on_rst2(&self) -> Point3;
    /// `Pnt2dOnRst1()`.
    fn pnt2d_on_rst1(&self) -> Point2;
    /// `Pnt2dOnRst2()`.
    fn pnt2d_on_rst2(&self) -> Point2;
    /// `ParameterOnRst1()`.
    fn parameter_on_rst1(&self) -> f64;
    /// `ParameterOnRst2()`.
    fn parameter_on_rst2(&self) -> f64;
    /// `IsTangencyPoint()`.
    fn is_tangency_point(&self) -> bool;
    /// `TangentOnRst1()`.
    fn tangent_on_rst1(&self) -> Result<Vector3, Failure>;
    /// `Tangent2dOnRst1()`.
    fn tangent2d_on_rst1(&self) -> Result<Vector2, Failure>;
    /// `TangentOnRst2()`.
    fn tangent_on_rst2(&self) -> Result<Vector3, Failure>;
    /// `Tangent2dOnRst2()`.
    fn tangent2d_on_rst2(&self) -> Result<Vector2, Failure>;
    /// `Decroch(Sol, NRst1, TgRst1, NRst2, TgRst2)`: which restrictions the section leaves
    /// at `sol`, with the normals and tangents on both.
    fn decroch(&self, sol: &Vector<'_>) -> Result<(DecrochStatus, [Vector3; 4]), Failure>;
}

/// `Blend_RstRstFunction::Pnt1()`: `PointOnRst1()`; `rst_rst_pnt1(&f)`.
pub fn rst_rst_pnt1<F: BlendRstRstFunction + ?Sized>(function: &F) -> Point3 {
    function.point_on_rst1()
}

/// `Blend_RstRstFunction::Pnt2()`: `PointOnRst2()`; `rst_rst_pnt2(&f)`.
pub fn rst_rst_pnt2<F: BlendRstRstFunction + ?Sized>(function: &F) -> Point3 {
    function.point_on_rst2()
}

/// `Blend_RstRstFunction::GetMinimalDistance()`: Standard_NotImplemented;
/// `rst_rst_minimal_distance()`.
pub fn rst_rst_minimal_distance() -> Result<f64, Failure> {
    Err(not_implemented("Blend_RstRstFunction::GetMinimalDistance"))
}

fn not_implemented(member: &str) -> Failure {
    Failure::with_message("Standard_NotImplemented", member)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_bodies_answer_as_natively() {
        assert_eq!(function_nb_variables(), 4);
        assert!(!function_section_d2());
        let failure = surf_rst_minimal_distance().unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_NotImplemented");
        assert_eq!(
            rst_rst_minimal_distance().unwrap_err().message(),
            "Blend_RstRstFunction::GetMinimalDistance"
        );
    }
}
