// Rust translation of ModelingAlgorithms/TKShHealing/ShapeExtend/ShapeExtend_ComplexCurve.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeExtend_ComplexCurve`: the abstract curve made of several segments
//! (docs/shhealing-design.md section 2).
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

/// `ShapeExtend_ComplexCurve`: a curve of several segments on the global range `[0, 1]`.
///
/// The abstract members are the required methods; the protected `myClosed` is stored in
/// each concrete class and reached through [`ShapeExtendComplexCurve::complex_closed`] and
/// [`ShapeExtendComplexCurve::set_complex_closed`]. The `Geom_Curve` members the class
/// overrides are the provided `complex_*` methods, and the macro
/// [`shape_extend_complex_curve_members!`](crate::shape_extend_complex_curve_members) puts
/// them in the concrete class's `impl GeomCurve` (its `impl GeomGeometry` calls
/// [`ShapeExtendComplexCurve::complex_transform`] from `transform`), as the native
/// inheritance does. No OCCT class derives from it; ShapeAnalysis_Curve recognizes it.
///
/// ```
/// use rustycad_tkshhealing::shape_extend::ShapeExtendComplexCurve;
/// fn global_range(curve: &dyn ShapeExtendComplexCurve) -> (f64, f64) {
///     (curve.complex_first_parameter(), curve.complex_last_parameter())
/// }
/// ```
pub trait ShapeExtendComplexCurve: GeomCurve {
    /// `NbCurves()` (pure virtual): the number of segments; `curve.nb_curves()`.
    fn nb_curves(&self) -> i32;

    /// `Curve(index)` (pure virtual): the segment of rank `index`; `curve.curve(1)?`.
    fn curve(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure>;

    /// `LocateParameter(U, UOut)` (pure virtual): the rank of the segment of the global
    /// parameter `u` and the local parameter on it; `let (i, local) = curve.locate_parameter(u)?;`.
    fn locate_parameter(&self, u: f64) -> Result<(i32, f64), Failure>;

    /// `LocalToGlobal(index, Ulocal)` (pure virtual): the global parameter of a local one;
    /// `curve.local_to_global(1, 0.5)?`.
    fn local_to_global(&self, index: i32, u_local: f64) -> Result<f64, Failure>;

    /// `GetScaleFactor(ind)` (pure virtual): the factor of the derivatives of segment
    /// `ind` (local over global parameter); `curve.get_scale_factor(1)?`.
    fn get_scale_factor(&self, ind: i32) -> Result<f64, Failure>;

    /// The protected `myClosed` (false after the protected constructor).
    fn complex_closed(&self) -> bool;

    /// Writes the protected `myClosed`.
    fn set_complex_closed(&self, closed: bool);

    /// `Transform(T)`: transforms each segment; `curve.complex_transform(t)?`.
    fn complex_transform(&self, transform: Transform3) -> Result<(), Failure> {
        for i in 1..=self.nb_curves() {
            live(&self.curve(i)?, i)?.transform(transform)?;
        }
        Ok(())
    }

    /// `ReversedParameter(U)`: `1 - U`; `curve.complex_reversed_parameter(0.25)`.
    fn complex_reversed_parameter(&self, u: f64) -> f64 {
        1.0 - u
    }

    /// `FirstParameter()`: 0.
    fn complex_first_parameter(&self) -> f64 {
        0.0
    }

    /// `LastParameter()`: 1.
    fn complex_last_parameter(&self) -> f64 {
        1.0
    }

    /// `IsClosed()`: the flag `CheckConnectivity` sets.
    fn complex_is_closed(&self) -> bool {
        self.complex_closed()
    }

    /// `IsCN(N)`: `N <= 0` (the continuity is C0).
    fn complex_is_cn(&self, n: i32) -> bool {
        n <= 0
    }

    /// `EvalD0(U)`: the point of the segment that holds `u`, at the local parameter.
    fn complex_eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let (index, local) = self.locate_parameter(u)?;
        live(&self.curve(index)?, index)?.eval_d0(local)
    }

    /// `EvalD1(U)`: the point and the derivative scaled by the segment's factor.
    fn complex_eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let (index, local) = self.locate_parameter(u)?;
        let mut result = live(&self.curve(index)?, index)?.eval_d1(local)?;
        self.transform_dn(&mut result.d1, index, 1)?;
        Ok(result)
    }

    /// `EvalD2(U)`: as `EvalD1`, each derivative of order k scaled k times.
    fn complex_eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let (index, local) = self.locate_parameter(u)?;
        let mut result = live(&self.curve(index)?, index)?.eval_d2(local)?;
        self.transform_dn(&mut result.d1, index, 1)?;
        self.transform_dn(&mut result.d2, index, 2)?;
        Ok(result)
    }

    /// `EvalD3(U)`: as `EvalD2` with the third derivative.
    fn complex_eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let (index, local) = self.locate_parameter(u)?;
        let mut result = live(&self.curve(index)?, index)?.eval_d3(local)?;
        self.transform_dn(&mut result.d1, index, 1)?;
        self.transform_dn(&mut result.d2, index, 2)?;
        self.transform_dn(&mut result.d3, index, 3)?;
        Ok(result)
    }

    /// `EvalDN(U, N)`: the derivative of order `n` of the segment, scaled unless `n` is 0.
    fn complex_eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        let (index, local) = self.locate_parameter(u)?;
        let mut result = live(&self.curve(index)?, index)?.eval_dn(local, n)?;
        if n != 0 {
            self.transform_dn(&mut result, index, n)?;
        }
        Ok(result)
    }

    /// `CheckConnectivity(Preci)`: whether each segment ends where the next one starts
    /// (within `preci`); sets the closed flag from the ends of the whole curve. The flag
    /// is set inside the loop over the joints, so a curve of one segment keeps it, as
    /// natively; `curve.check_connectivity(1e-7)?`.
    fn check_connectivity(&self, preci: f64) -> Result<bool, Failure> {
        let nb = self.nb_curves();
        let mut ok = true;
        for i in 1..nb {
            if i == 1 {
                let first = self.value(self.first_parameter())?;
                let last = self.value(self.last_parameter())?;
                self.set_complex_closed(first.is_equal(&last, preci));
            }
            let (current, next) = (self.curve(i)?, self.curve(i + 1)?);
            let (current, next) = (live(&current, i)?, live(&next, i + 1)?);
            let end = current.value(current.last_parameter())?;
            ok &= end.is_equal(&next.value(next.first_parameter())?, preci);
        }
        Ok(ok)
    }

    /// `TransformDN(V, ind, N)` (protected): multiplies `v` by the scale factor of segment
    /// `ind`, `n` times (one product per order, not a power, as natively).
    fn transform_dn(&self, v: &mut Vector3, ind: i32, n: i32) -> Result<(), Failure> {
        let factor = self.get_scale_factor(ind)?;
        for _ in 1..=n {
            *v *= factor;
        }
        Ok(())
    }
}
impl_transient_interface!(
    dyn ShapeExtendComplexCurve,
    "ShapeExtend_ComplexCurve",
    dyn GeomCurve
);

/// The segment of rank `index`, Standard_NullObject when it is null (a null dereference
/// natively).
fn live(curve: &Handle<dyn GeomCurve>, index: i32) -> Result<&dyn GeomCurve, Failure> {
    curve.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            &format!("ShapeExtend_ComplexCurve: segment {index} is a null curve; expected a curve"),
        )
    })
}

/// The `Geom_Curve` members that `ShapeExtend_ComplexCurve` overrides, for the
/// `impl GeomCurve` of a concrete complex curve: `reversed_parameter`, the parameter range,
/// `is_closed`, `is_periodic`, `continuity`, `is_cn` and the evaluators. The concrete class
/// writes `reverse` (pure virtual in `Geom_Curve`) itself.
///
/// ```ignore
/// impl GeomCurve for MyComplexCurve {
///     fn reverse(&self) -> Result<(), Failure> { /* ... */ Ok(()) }
///     rustycad_tkshhealing::shape_extend_complex_curve_members!();
/// }
/// ```
#[macro_export]
macro_rules! shape_extend_complex_curve_members {
    () => {
        fn reversed_parameter(&self, u: f64) -> Result<f64, rustycad_tkernel::standard::Failure> {
            Ok($crate::shape_extend::ShapeExtendComplexCurve::complex_reversed_parameter(self, u))
        }
        fn first_parameter(&self) -> f64 {
            $crate::shape_extend::ShapeExtendComplexCurve::complex_first_parameter(self)
        }
        fn last_parameter(&self) -> f64 {
            $crate::shape_extend::ShapeExtendComplexCurve::complex_last_parameter(self)
        }
        fn is_closed(&self) -> Result<bool, rustycad_tkernel::standard::Failure> {
            Ok($crate::shape_extend::ShapeExtendComplexCurve::complex_is_closed(self))
        }
        fn is_periodic(&self) -> bool {
            false
        }
        fn continuity(&self) -> rustycad_tkmath::geom_abs::Shape {
            rustycad_tkmath::geom_abs::Shape::C0
        }
        fn is_cn(&self, n: i32) -> Result<bool, rustycad_tkernel::standard::Failure> {
            Ok($crate::shape_extend::ShapeExtendComplexCurve::complex_is_cn(self, n))
        }
        fn eval_d0(
            &self,
            u: f64,
        ) -> Result<rustycad_tkmath::gp::Point3, rustycad_tkernel::standard::Failure> {
            $crate::shape_extend::ShapeExtendComplexCurve::complex_eval_d0(self, u)
        }
        fn eval_d1(
            &self,
            u: f64,
        ) -> Result<rustycad_tkg3d::geom::GeomCurveD1, rustycad_tkernel::standard::Failure> {
            $crate::shape_extend::ShapeExtendComplexCurve::complex_eval_d1(self, u)
        }
        fn eval_d2(
            &self,
            u: f64,
        ) -> Result<rustycad_tkg3d::geom::GeomCurveD2, rustycad_tkernel::standard::Failure> {
            $crate::shape_extend::ShapeExtendComplexCurve::complex_eval_d2(self, u)
        }
        fn eval_d3(
            &self,
            u: f64,
        ) -> Result<rustycad_tkg3d::geom::GeomCurveD3, rustycad_tkernel::standard::Failure> {
            $crate::shape_extend::ShapeExtendComplexCurve::complex_eval_d3(self, u)
        }
        fn eval_dn(
            &self,
            u: f64,
            n: i32,
        ) -> Result<rustycad_tkmath::gp::Vector3, rustycad_tkernel::standard::Failure> {
            $crate::shape_extend::ShapeExtendComplexCurve::complex_eval_dn(self, u, n)
        }
    };
}
