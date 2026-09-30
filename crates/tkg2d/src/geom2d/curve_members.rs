// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Curve.hxx (the constant overrides that Geom2d, Geom, Geom2dEval and GeomEval curves repeat) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The one-line virtual overrides that many curve classes write identically:
//! the parameter range of an open curve and the infinite continuity of an
//! analytic one. The macros expand to members of a `Geom2dCurve` or `GeomCurve`
//! implementation; the signatures of both traits are the same for these members.

/// `FirstParameter`, `LastParameter` and `IsClosed` of an open curve on
/// `[first, last]`: `IsClosed` is always false;
/// `rustycad_tkg2d::open_curve_range!(-precision::infinite(), precision::infinite());`.
#[macro_export]
macro_rules! open_curve_range {
    ($first:expr, $last:expr) => {
        fn first_parameter(&self) -> f64 {
            $first
        }
        fn last_parameter(&self) -> f64 {
            $last
        }
        fn is_closed(&self) -> ::core::result::Result<bool, ::rustycad_tkernel::standard::Failure> {
            Ok(false)
        }
    };
}

/// `IsPeriodic`, `Continuity` and `IsCN` of a non-periodic curve that is CN
/// everywhere (lines, parabolas, hyperbolas, Bezier and eval curves);
/// `rustycad_tkg2d::non_periodic_cn_curve!();`.
#[macro_export]
macro_rules! non_periodic_cn_curve {
    () => {
        $crate::cn_curve_continuity!(false);
    };
}

/// `IsPeriodic` (returning `$periodic`), `Continuity` and `IsCN` of a curve that is
/// CN everywhere; `rustycad_tkg2d::cn_curve_continuity!(true);`.
#[macro_export]
macro_rules! cn_curve_continuity {
    ($periodic:literal) => {
        fn is_periodic(&self) -> bool {
            $periodic
        }
        fn continuity(&self) -> ::rustycad_tkmath::geom_abs::Shape {
            ::rustycad_tkmath::geom_abs::Shape::CN
        }
        fn is_cn(
            &self,
            _n: i32,
        ) -> ::core::result::Result<bool, ::rustycad_tkernel::standard::Failure> {
            Ok(true)
        }
    };
}

/// `Reverse` and `ReversedParameter` of an eval curve, which keeps its defining
/// parameterization: both raise the package's Standard_NotImplemented through
/// `$unsupported(member, detail)`;
/// `rustycad_tkg2d::unsupported_curve_reversal!(unsupported, "Geom2dEval_SineWaveCurve");`.
#[macro_export]
macro_rules! unsupported_curve_reversal {
    ($unsupported:path, $class:literal) => {
        fn reverse(&self) -> ::core::result::Result<(), ::rustycad_tkernel::standard::Failure> {
            Err($unsupported(concat!($class, "::Reverse"), ""))
        }
        fn reversed_parameter(
            &self,
            u: f64,
        ) -> ::core::result::Result<f64, ::rustycad_tkernel::standard::Failure> {
            let detail: String = format!("(U = {u})");
            Err($unsupported(
                concat!($class, "::ReversedParameter"),
                &detail,
            ))
        }
    };
}

/// The `HasEvalRepresentation`, `EvalRepresentation`, `SetEvalRepresentation` and
/// `ClearEvalRepresentation` members that every class with an evaluation
/// representation writes identically over its locked `state.eval_rep`; `$validate`
/// is the package RepUtils check (`ValidateCurveDesc` or `ValidateSurfaceDesc`);
/// `rustycad_tkg2d::eval_representation_members!(dyn RepCurveDesc, rep_utils::validate_curve_desc, "curve");`.
#[macro_export]
macro_rules! eval_representation_members {
    ($desc:ty, $validate:path, $object:literal) => {
        #[doc = concat!(
                    "Whether an evaluation representation is set (`HasEvalRepresentation`);\n`",
                    $object,
                    ".has_eval_representation()`."
                )]
        pub fn has_eval_representation(&self) -> bool {
            self.state.read().eval_rep.is_some()
        }
        #[doc = concat!(
                    "The evaluation representation, possibly null (`EvalRepresentation`); `",
                    $object,
                    ".eval_representation()`."
                )]
        pub fn eval_representation(&self) -> ::rustycad_tkernel::standard::Handle<$desc> {
            self.state.read().eval_rep.clone()
        }
        #[doc = concat!(
                    "Validate and set the evaluation representation (`SetEvalRepresentation`);\n`",
                    $object,
                    ".set_eval_representation(desc)?`."
                )]
        pub fn set_eval_representation(
            &self,
            desc: ::rustycad_tkernel::standard::Handle<$desc>,
        ) -> ::core::result::Result<(), ::rustycad_tkernel::standard::Failure> {
            $validate(&desc, std::ptr::from_ref(self).cast())?;
            self.state.write().eval_rep = desc;
            Ok(())
        }
        #[doc = concat!(
                    "Remove the evaluation representation (`ClearEvalRepresentation`); `",
                    $object,
                    ".clear_eval_representation()`."
                )]
        pub fn clear_eval_representation(&self) {
            self.state.write().eval_rep = ::rustycad_tkernel::standard::Handle::default();
        }
    };
}

/// `EvalD0`..`EvalDN` of a spline curve class that forward to its locked shared
/// state, whose evaluators return tuples; `$d1`..`$d3` are the package result
/// structs; `rustycad_tkg2d::spline_state_evaluators!(Point3, Vector3, GeomCurveD1, GeomCurveD2, GeomCurveD3);`.
#[macro_export]
macro_rules! spline_state_evaluators {
    ($point:ty, $vector:ty, $d1:ident, $d2:ident, $d3:ident) => {
        fn eval_d0(
            &self,
            u: f64,
        ) -> ::core::result::Result<$point, ::rustycad_tkernel::standard::Failure> {
            self.state.read().eval_d0(u)
        }
        fn eval_d1(
            &self,
            u: f64,
        ) -> ::core::result::Result<$d1, ::rustycad_tkernel::standard::Failure> {
            let (point, d1) = self.state.read().eval_d1(u)?;
            Ok($d1 { point, d1 })
        }
        fn eval_d2(
            &self,
            u: f64,
        ) -> ::core::result::Result<$d2, ::rustycad_tkernel::standard::Failure> {
            let (point, d1, d2) = self.state.read().eval_d2(u)?;
            Ok($d2 { point, d1, d2 })
        }
        fn eval_d3(
            &self,
            u: f64,
        ) -> ::core::result::Result<$d3, ::rustycad_tkernel::standard::Failure> {
            let (point, d1, d2, d3) = self.state.read().eval_d3(u)?;
            Ok($d3 { point, d1, d2, d3 })
        }
        fn eval_dn(
            &self,
            u: f64,
            n: i32,
        ) -> ::core::result::Result<$vector, ::rustycad_tkernel::standard::Failure> {
            self.state.read().eval_dn(u, n)
        }
    };
}

/// The `Geometry` members of a spline curve class: `Transform` moves every pole,
/// `Copy` runs the copy constructor `copy_of`, and `DumpJson` writes the class name,
/// the bounded-curve base (`$dump_base`) and the state fields;
/// `rustycad_tkg2d::spline_geometry_members!(Transform3, dyn GeomGeometry, "Geom_BezierCurve", dump_bounded_curve);`.
#[macro_export]
macro_rules! spline_geometry_members {
    ($transform:ty, $geometry:ty, $class:literal, $dump_base:path) => {
        fn transform(
            &self,
            transform: $transform,
        ) -> ::core::result::Result<(), ::rustycad_tkernel::standard::Failure> {
            self.state
                .write()
                .transform_poles(|pole| pole.transform(transform));
            Ok(())
        }
        fn copy(
            &self,
        ) -> ::core::result::Result<
            ::rustycad_tkernel::standard::Handle<$geometry>,
            ::rustycad_tkernel::standard::Failure,
        > {
            Ok(::rustycad_tkernel::standard::Handle::new(Self::copy_of(self)?).cast())
        }
        fn dump_json(
            &self,
            stream: &mut ::rustycad_tkernel::standard::dump::DumpStream,
            depth: i32,
        ) -> ::std::io::Result<()> {
            let state = self.state.read();
            $crate::geom2d::dump_class_name(stream, $class)?;
            $crate::geom2d::dump_base(stream, depth, $dump_base)?;
            state.dump_fields(stream)
        }
    };
}

/// The members of a closed periodic conic on `[0, 2 Pi]` (circle, ellipse):
/// `ReversedParameter` is `2 Pi - U`, the curve is closed, periodic and CN;
/// `rustycad_tkg2d::full_turn_curve_members!();`.
#[macro_export]
macro_rules! full_turn_curve_members {
    () => {
        fn reversed_parameter(
            &self,
            u: f64,
        ) -> ::core::result::Result<f64, ::rustycad_tkernel::standard::Failure> {
            Ok(2. * ::std::f64::consts::PI - u)
        }
        fn first_parameter(&self) -> f64 {
            0.0
        }
        fn last_parameter(&self) -> f64 {
            2.0 * ::std::f64::consts::PI
        }
        fn is_closed(&self) -> ::core::result::Result<bool, ::rustycad_tkernel::standard::Failure> {
            Ok(true)
        }
        $crate::cn_curve_continuity!(true);
    };
}

/// `TransformedParameter` and `ParametricTransformation` of a curve whose parameter
/// scales with the transformation (line, parabola): `U * |Scale|`, an infinite `U`
/// unchanged; `rustycad_tkg2d::scaled_parameter_members!(Transform3);`.
#[macro_export]
macro_rules! scaled_parameter_members {
    ($transform:ty) => {
        fn transformed_parameter(&self, u: f64, transform: $transform) -> f64 {
            if ::rustycad_tkernel::precision::is_infinite(u) {
                return u;
            }
            u * transform.scale_factor().abs()
        }
        fn parametric_transformation(&self, transform: $transform) -> f64 {
            transform.scale_factor().abs()
        }
    };
}

/// `TransformedParameter` and `ParametricTransformation` of a curve that keeps a
/// basis curve (offset, trimmed): both forward to the basis over the locked
/// `state.basis`; `rustycad_tkg2d::basis_parameter_members!(Transform3);`.
#[macro_export]
macro_rules! basis_parameter_members {
    ($transform:ty) => {
        fn transformed_parameter(&self, u: f64, transform: $transform) -> f64 {
            self.state.read().basis.transformed_parameter(u, transform)
        }
        fn parametric_transformation(&self, transform: $transform) -> f64 {
            self.state.read().basis.parametric_transformation(transform)
        }
    };
}

/// `FirstParameter` and `LastParameter` of an offset curve, which are its basis
/// curve's; `rustycad_tkg2d::basis_range_members!();`.
#[macro_export]
macro_rules! basis_range_members {
    () => {
        fn first_parameter(&self) -> f64 {
            self.state.read().basis.first_parameter()
        }
        fn last_parameter(&self) -> f64 {
            self.state.read().basis.last_parameter()
        }
    };
}

/// `EvalDN` of an offset curve: the evaluation representation first, orders 1 to 3
/// through the explicit evaluators and higher orders from the basis, as natively;
/// `errors`, `rep_utils` and the curve trait must be in scope;
/// `rustycad_tkg2d::offset_curve_eval_dn!(Vector3, "Geom_OffsetCurve");`.
#[macro_export]
macro_rules! offset_curve_eval_dn {
    ($vector:ty, $class:literal) => {
        fn eval_dn(&self, u: f64, n: i32) -> Result<$vector, Failure> {
            if n < 1 {
                return Err(errors::undefined_derivative(&format!(
                    concat!($class, "::EvalDN: N = {}; expected N >= 1"),
                    n
                )));
            }
            if let Some(vector) = rep_utils::try_eval_curve_dn(&self.state.read().eval_rep, u, n)? {
                return Ok(vector);
            }
            match n {
                1 => Ok(self.eval_d1(u)?.d1),
                2 => Ok(self.eval_d2(u)?.d2),
                3 => Ok(self.eval_d3(u)?.d3),
                _ => self.state.read().basis.eval_dn(u, n),
            }
        }
    };
}
