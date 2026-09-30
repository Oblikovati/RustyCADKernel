// Rust translation of the members that GeomEval_EllipsoidSurface, GeomEval_HyperboloidSurface, GeomEval_HypParaboloidSurface, GeomEval_ParaboloidSurface and GeomEval_CircularHelicoidSurface (ModelingData/TKG3d/GeomEval) write alike, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! What the five GeomEval elementary surfaces share: the local frame their
//! evaluators read from `pos`, the assembly of `ResD1..ResD3` from per-order
//! derivative formulas, the diagonal quadric `Coefficients` expansion and the
//! trait members that are identical in every class (the refused reversals, isos
//! and `Transform`, the inherited `Geom_ElementarySurface` continuity, `Copy`
//! and the `DumpJson` shape).
//!
//! Every class states its formulas once per order; `EvalD1`, `EvalD2` and
//! `EvalD3` natively repeat the lower orders verbatim, so assembling them from
//! the same per-order functions gives the same bits.
use crate::geom::quadric::frame_rows;
use crate::geom::{GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3};
use rustycad_tkmath::gp::{Axis3, Point3, Vector3, Xyz};

/// `pos.Location()`, `XDirection()`, `YDirection()` and `Direction()` as the
/// `gp_XYZ` values the evaluators combine; `EvalFrame::new(pos).z`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EvalFrame {
    pub(crate) origin: Xyz,
    pub(crate) x: Xyz,
    pub(crate) y: Xyz,
    pub(crate) z: Xyz,
}
impl EvalFrame {
    /// The frame of `pos`; `EvalFrame::new(Axis3::default())`.
    pub(crate) fn new(pos: Axis3) -> Self {
        Self {
            origin: pos.location().xyz(),
            x: pos.x_direction().xyz(),
            y: pos.y_direction().xyz(),
            z: pos.direction().xyz(),
        }
    }
}

/// The derivative formulas of one surface at one `(U, V)`, one method per order;
/// `patch.first()[0]` is `D1U`.
pub(crate) trait PatchDerivatives {
    /// `P`.
    fn point(&self) -> Xyz;
    /// `[D1U, D1V]`.
    fn first(&self) -> [Xyz; 2];
    /// `[D2U, D2V, D2UV]`.
    fn second(&self) -> [Xyz; 3];
    /// `[D3U, D3V, D3UUV, D3UVV]`.
    fn third(&self) -> [Xyz; 4];
}

/// `EvalD1` from the per-order formulas; `result_d1(&patch)`.
pub(crate) fn result_d1(patch: &impl PatchDerivatives) -> GeomSurfaceD1 {
    let [d1u, d1v] = patch.first().map(Vector3::from);
    GeomSurfaceD1 {
        point: Point3::from(patch.point()),
        d1u,
        d1v,
    }
}

/// `EvalD2` from the per-order formulas; `result_d2(&patch)`.
pub(crate) fn result_d2(patch: &impl PatchDerivatives) -> GeomSurfaceD2 {
    let [d1u, d1v] = patch.first().map(Vector3::from);
    let [d2u, d2v, d2uv] = patch.second().map(Vector3::from);
    GeomSurfaceD2 {
        point: Point3::from(patch.point()),
        d1u,
        d1v,
        d2u,
        d2v,
        d2uv,
    }
}

/// `EvalD3` from the per-order formulas; `result_d3(&patch)`.
pub(crate) fn result_d3(patch: &impl PatchDerivatives) -> GeomSurfaceD3 {
    let second: GeomSurfaceD2 = result_d2(patch);
    let [d3u, d3v, d3uuv, d3uvv] = patch.third().map(Vector3::from);
    GeomSurfaceD3 {
        point: second.point,
        d1u: second.d1u,
        d1v: second.d1v,
        d2u: second.d2u,
        d2v: second.d2v,
        d2uv: second.d2uv,
        d3u,
        d3v,
        d3uuv,
        d3uvv,
    }
}

/// The global `Coefficients` `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]` of the local
/// quadric `a1 X^2 + a2 Y^2 + a3 Z^2 + d = 0`, expanded through the rows of
/// `T.SetTransformation(pos)` in the native order (ellipsoid and hyperboloid).
///
/// The ellipsoid writes its last term `... - 1.0` and the hyperboloid `... + aDLoc`;
/// `x - 1.0` and `x + (-1.0)` round identically, so both pass `d`;
/// `diagonal_quadric_coefficients(pos, [1.0, 1.0, 1.0], -1.0)`.
pub(crate) fn diagonal_quadric_coefficients(
    pos: Axis3,
    [a1, a2, a3]: [f64; 3],
    d: f64,
) -> [f64; 10] {
    let [r1, r2, r3] = frame_rows(pos);
    // `a1 * Ti * Tj + a2 * ... + a3 * ...` for the columns (i, j) of each coefficient.
    let term = |i: usize, j: usize| a1 * r1[i] * r1[j] + a2 * r2[i] * r2[j] + a3 * r3[i] * r3[j];
    [
        term(0, 0),
        term(1, 1),
        term(2, 2),
        term(0, 1),
        term(0, 2),
        term(1, 2),
        term(0, 3),
        term(1, 3),
        term(2, 3),
        term(3, 3) + d,
    ]
}

/// The class-specific parts of a GeomEval elementary surface that
/// `impl_eval_elementary_surface!` wires into the Geom traits.
pub(crate) trait EvalElementarySurface {
    /// The native class name (`GeomEval_EllipsoidSurface`).
    const CLASS: &'static str;
    /// `Bounds`.
    const BOUNDS: (f64, f64, f64, f64);
    /// `IsUClosed` and `IsUPeriodic` (equal in every class).
    const U_CLOSED: bool;
    /// The derivative formulas at one parameter pair.
    type Patch: PatchDerivatives;
    /// The formulas of the current state at `(u, v)`.
    fn patch(&self, u: f64, v: f64) -> Self::Patch;
    /// `EvalDN` after the order check.
    fn derivative_n(&self, u: f64, v: f64, nu: i32, nv: i32) -> Vector3;
    /// The `OCCT_DUMP_FIELD_VALUE_NUMERICAL` fields after the base class.
    fn dump_parameters(
        &self,
        stream: &mut rustycad_tkernel::standard::dump::DumpStream,
    ) -> std::io::Result<()>;
}

/// Implement `GeomGeometry`, `GeomSurface` and `GeomElementarySurface` for a
/// GeomEval elementary surface whose struct has a `state` cell with a `pos`
/// field and implements `Clone` and `EvalElementarySurface`.
///
/// Every class refuses `Transform`, `UReverse`, `VReverse`, the reversed parameters
/// and both isos with Standard_NotImplemented; `IsVClosed` and `IsVPeriodic` are
/// false; `Continuity` and `IsCNu`/`IsCNv` are the inherited
/// `Geom_ElementarySurface` members (CN, always true).
macro_rules! impl_eval_elementary_surface {
    ($type:ty) => {
        impl $crate::geom::GeomGeometry for $type {
            fn transform(
                &self,
                _transform: rustycad_tkmath::gp::Transform3,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                Err($crate::geom_eval::elementary_eval::refused::<Self>(
                    "Transform",
                    "",
                ))
            }
            fn copy(
                &self,
            ) -> Result<
                rustycad_tkernel::standard::Handle<dyn $crate::geom::GeomGeometry>,
                rustycad_tkernel::standard::Failure,
            > {
                Ok(rustycad_tkernel::standard::Handle::new(self.clone()).cast())
            }
            fn dump_json(
                &self,
                stream: &mut rustycad_tkernel::standard::dump::DumpStream,
                depth: i32,
            ) -> std::io::Result<()> {
                use $crate::geom::GeomElementarySurface;
                use $crate::geom_eval::elementary_eval::EvalElementarySurface;
                let pos: rustycad_tkmath::gp::Axis3 = self.position();
                rustycad_tkg2d::geom2d::dump_class_name(
                    stream,
                    <Self as EvalElementarySurface>::CLASS,
                )?;
                rustycad_tkg2d::geom2d::dump_base(stream, depth, |s, d| {
                    $crate::geom::dump_elementary_surface(s, d, pos)
                })?;
                self.dump_parameters(stream)
            }
        }
        impl $crate::geom::GeomSurface for $type {
            $crate::geom_eval::elementary_eval::eval_elementary_surface_members!();
        }
        impl $crate::geom::GeomElementarySurface for $type {
            $crate::geom::impl_elementary_position!();
        }
    };
}
pub(crate) use impl_eval_elementary_surface;

/// The `GeomSurface` members of `impl_eval_elementary_surface!`.
macro_rules! eval_elementary_surface_members {
    () => {
        fn u_reverse(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
            Err($crate::geom_eval::elementary_eval::refused::<Self>(
                "UReverse", "",
            ))
        }
        fn u_reversed_parameter(&self, u: f64) -> Result<f64, rustycad_tkernel::standard::Failure> {
            let detail: String = format!("(U = {u})");
            Err($crate::geom_eval::elementary_eval::refused::<Self>(
                "UReversedParameter",
                &detail,
            ))
        }
        fn v_reverse(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
            Err($crate::geom_eval::elementary_eval::refused::<Self>(
                "VReverse", "",
            ))
        }
        fn v_reversed_parameter(&self, v: f64) -> Result<f64, rustycad_tkernel::standard::Failure> {
            let detail: String = format!("(V = {v})");
            Err($crate::geom_eval::elementary_eval::refused::<Self>(
                "VReversedParameter",
                &detail,
            ))
        }
        fn bounds(&self) -> $crate::geom::SurfaceBounds {
            <Self as $crate::geom_eval::elementary_eval::EvalElementarySurface>::BOUNDS
        }
        fn is_u_closed(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
            Ok(<Self as $crate::geom_eval::elementary_eval::EvalElementarySurface>::U_CLOSED)
        }
        fn is_v_closed(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
            Ok(false)
        }
        fn is_u_periodic(&self) -> bool {
            <Self as $crate::geom_eval::elementary_eval::EvalElementarySurface>::U_CLOSED
        }
        fn is_v_periodic(&self) -> bool {
            false
        }
        fn u_iso(
            &self,
            u: f64,
        ) -> Result<
            rustycad_tkernel::standard::Handle<dyn $crate::geom::GeomCurve>,
            rustycad_tkernel::standard::Failure,
        > {
            let detail: String = format!("(U = {u})");
            Err($crate::geom_eval::elementary_eval::refused::<Self>(
                "UIso", &detail,
            ))
        }
        fn v_iso(
            &self,
            v: f64,
        ) -> Result<
            rustycad_tkernel::standard::Handle<dyn $crate::geom::GeomCurve>,
            rustycad_tkernel::standard::Failure,
        > {
            let detail: String = format!("(V = {v})");
            Err($crate::geom_eval::elementary_eval::refused::<Self>(
                "VIso", &detail,
            ))
        }
        fn continuity(&self) -> rustycad_tkmath::geom_abs::Shape {
            rustycad_tkmath::geom_abs::Shape::CN
        }
        fn is_cn_u(&self, _n: i32) -> Result<bool, rustycad_tkernel::standard::Failure> {
            Ok(true)
        }
        fn is_cn_v(&self, _n: i32) -> Result<bool, rustycad_tkernel::standard::Failure> {
            Ok(true)
        }
        fn eval_d0(
            &self,
            u: f64,
            v: f64,
        ) -> Result<rustycad_tkmath::gp::Point3, rustycad_tkernel::standard::Failure> {
            use $crate::geom_eval::elementary_eval::{EvalElementarySurface, PatchDerivatives};
            Ok(rustycad_tkmath::gp::Point3::from(self.patch(u, v).point()))
        }
        fn eval_d1(
            &self,
            u: f64,
            v: f64,
        ) -> Result<$crate::geom::GeomSurfaceD1, rustycad_tkernel::standard::Failure> {
            use $crate::geom_eval::elementary_eval::EvalElementarySurface;
            Ok($crate::geom_eval::elementary_eval::result_d1(
                &self.patch(u, v),
            ))
        }
        fn eval_d2(
            &self,
            u: f64,
            v: f64,
        ) -> Result<$crate::geom::GeomSurfaceD2, rustycad_tkernel::standard::Failure> {
            use $crate::geom_eval::elementary_eval::EvalElementarySurface;
            Ok($crate::geom_eval::elementary_eval::result_d2(
                &self.patch(u, v),
            ))
        }
        fn eval_d3(
            &self,
            u: f64,
            v: f64,
        ) -> Result<$crate::geom::GeomSurfaceD3, rustycad_tkernel::standard::Failure> {
            use $crate::geom_eval::elementary_eval::EvalElementarySurface;
            Ok($crate::geom_eval::elementary_eval::result_d3(
                &self.patch(u, v),
            ))
        }
        fn eval_dn(
            &self,
            u: f64,
            v: f64,
            nu: i32,
            nv: i32,
        ) -> Result<rustycad_tkmath::gp::Vector3, rustycad_tkernel::standard::Failure> {
            use $crate::geom_eval::elementary_eval::EvalElementarySurface;
            $crate::geom::check_dn_orders(Self::CLASS, nu, nv)?;
            Ok(self.derivative_n(u, v, nu, nv))
        }
    };
}
pub(crate) use eval_elementary_surface_members;

/// Standard_NotImplemented of `Class::member`; `refused::<GeomEvalParaboloidSurface>("UIso", "(U = 1)")`.
pub(crate) fn refused<T: EvalElementarySurface>(
    member: &str,
    detail: &str,
) -> rustycad_tkernel::standard::Failure {
    super::eval_failures::unsupported(&format!("{}::{member}", T::CLASS), detail)
}

/// The Standard_ConstructionError of a parameter that must be strictly positive;
/// `positive(2.0, "GeomEval_ParaboloidSurface: focal")?`.
pub(crate) fn positive(value: f64, what: &str) -> Result<f64, rustycad_tkernel::standard::Failure> {
    if value <= 0.0 {
        return Err(crate::geom::errors::construction(&format!(
            "{what} = {value}; expected > 0"
        )));
    }
    Ok(value)
}
