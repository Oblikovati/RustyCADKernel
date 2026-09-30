// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_MultiCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_MultiCurve`: a set of Bezier curves of one degree, stored as one
//! MultiPoint per pole.
use super::multi_point::MultiPoint;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::fmt;

/// The virtual members of `AppParCurves_MultiCurve`, overridden by
/// `AppParCurves_MultiBSpCurve`: a caller holding either type through this trait
/// gets the native dynamic dispatch.
pub trait MultiCurveEval {
    /// `NbPoles()`; `curve.nb_poles()`.
    fn nb_poles(&self) -> i32;
    /// `Degree()`; `curve.degree()?`.
    fn degree(&self) -> Result<i32, Failure>;
    /// `Value(CuIndex, U, Pt)` on a 3D curve; `curve.value_3d(1, 0.5)?`.
    fn value_3d(&self, cu_index: i32, u: f64) -> Result<Point3, Failure>;
    /// `Value(CuIndex, U, Pt)` on a 2D curve; `curve.value_2d(2, 0.5)?`.
    fn value_2d(&self, cu_index: i32, u: f64) -> Result<Point2, Failure>;
    /// `D1(CuIndex, U, Pt, V1)` on a 3D curve; `curve.d1_3d(1, 0.5)?`.
    fn d1_3d(&self, cu_index: i32, u: f64) -> Result<(Point3, Vector3), Failure>;
    /// `D1(CuIndex, U, Pt, V1)` on a 2D curve; `curve.d1_2d(2, 0.5)?`.
    fn d1_2d(&self, cu_index: i32, u: f64) -> Result<(Point2, Vector2), Failure>;
    /// `D2(CuIndex, U, Pt, V1, V2)` on a 3D curve; `curve.d2_3d(1, 0.5)?`.
    fn d2_3d(&self, cu_index: i32, u: f64) -> Result<(Point3, Vector3, Vector3), Failure>;
    /// `D2(CuIndex, U, Pt, V1, V2)` on a 2D curve; `curve.d2_2d(2, 0.5)?`.
    fn d2_2d(&self, cu_index: i32, u: f64) -> Result<(Point2, Vector2, Vector2), Failure>;
    /// `Dump(o)`; `curve.dump(&mut text)?`.
    fn dump(&self, stream: &mut dyn fmt::Write) -> fmt::Result;
}

/// `AppParCurves_MultiCurve`: pole `i` of curve `k` is `Value(i).Point(k)` (or
/// `Point2d`). The poles live in a shared array: a copy (`clone`) shares it, as the
/// native copy shares its handle.
///
/// ```
/// use rustycad_tkgeombase::app_par_curves::{MultiCurve, MultiCurveEval, MultiPoint};
/// use rustycad_tkmath::gp::Point3;
/// let curve = MultiCurve::with_nb_poles(2)?;
/// for (i, x) in [(1, 0.0), (2, 2.0)] {
///     let pole = MultiPoint::with_counts(1, 0)?;
///     pole.set_point(1, Point3::new(x, 0.0, 0.0))?;
///     curve.set_value(i, &pole)?;
/// }
/// assert_eq!(curve.value_3d(1, 0.5)?, Point3::new(1.0, 0.0, 0.0));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct MultiCurve {
    tab_point: Handle<HArray1<MultiPoint>>,
}

pub(super) fn null_poles(member: &str) -> Failure {
    failure(
        "Standard_NullObject",
        &format!("AppParCurves_MultiCurve::{member}: the curve has no pole array"),
    )
}

impl MultiCurve {
    /// `AppParCurves_MultiCurve()`: no pole array; `MultiCurve::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `AppParCurves_MultiCurve(NbPol)`: `NbPol` empty MultiPoints;
    /// `MultiCurve::with_nb_poles(4)?`.
    pub fn with_nb_poles(nb_poles: i32) -> Result<Self, Failure> {
        let mut curve: Self = Self::new();
        curve.set_nb_poles(nb_poles)?;
        Ok(curve)
    }

    /// `AppParCurves_MultiCurve(tabMU)`: the MultiPoints of `tab_mu` (sharing their
    /// points), reindexed from 1; `MultiCurve::from_multi_points(&poles)`.
    pub fn from_multi_points(tab_mu: &Array1<'_, MultiPoint>) -> Self {
        let mut poles: Array1<'static, MultiPoint> = Array1::with_size(tab_mu.len());
        poles.update_lower_bound(1);
        poles
            .iter_mut()
            .zip(tab_mu.iter())
            .for_each(|(slot, pole)| *slot = pole.clone());
        Self {
            tab_point: Handle::new(HArray1::from_owned(poles)),
        }
    }

    /// `SetNbPoles(nbPoles)`: a new array of empty MultiPoints; `curve.set_nb_poles(3)?`.
    pub fn set_nb_poles(&mut self, nb_poles: i32) -> Result<(), Failure> {
        let poles: HArray1<MultiPoint> = HArray1::with_bounds(1, nb_poles)
            .map_err(|error| array_failure(error, "AppParCurves_MultiCurve"))?;
        self.tab_point = Handle::new(poles);
        Ok(())
    }

    // The pole array, Standard_NullObject for a curve without one (a crash natively).
    pub(super) fn poles(&self, member: &str) -> Result<&HArray1<MultiPoint>, Failure> {
        self.tab_point.get().ok_or_else(|| null_poles(member))
    }

    /// `SetValue(Index, MPoint)`: stores a copy sharing the points of `mpoint`;
    /// Standard_OutOfRange outside `1..=NbPoles()`; `curve.set_value(1, &pole)?`.
    pub fn set_value(&self, index: i32, mpoint: &MultiPoint) -> Result<(), Failure> {
        let poles: &HArray1<MultiPoint> = self.poles("SetValue")?;
        let length: i32 = poles.array1().length();
        if index <= 0 || index > length {
            return Err(Failure::out_of_range(&format!(
                "AppParCurves_MultiCurve::SetValue: index {index}; expected 1..={length}"
            )));
        }
        poles
            .change_array1()
            .set_value(index, mpoint.clone())
            .map_err(|error| array_failure(error, "AppParCurves_MultiCurve::SetValue"))
    }

    /// `NbCurves()`: the number of points of the first MultiPoint, 0 without poles;
    /// `curve.nb_curves()?`.
    pub fn nb_curves(&self) -> Result<i32, Failure> {
        let Some(poles) = self.tab_point.get() else {
            return Ok(0);
        };
        let first: MultiPoint = self.pole_point(poles, 1)?;
        Ok(first.nb_points() + first.nb_points2d())
    }

    /// `Dimension(CuIndex)`: 3 or 2, read on the first MultiPoint;
    /// Standard_OutOfRange outside `1..=NbCurves()`; `curve.dimension(1)?`.
    pub fn dimension(&self, cu_index: i32) -> Result<i32, Failure> {
        let poles: &HArray1<MultiPoint> = self.poles("Dimension")?;
        let first: MultiPoint = self.pole_point(poles, poles.array1().lower())?;
        let nb: i32 = first.nb_points() + first.nb_points2d();
        if cu_index <= 0 || cu_index > nb {
            return Err(Failure::out_of_range(&format!(
                "AppParCurves_MultiCurve::Dimension: curve {cu_index}; expected 1..={nb}"
            )));
        }
        first.dimension(cu_index)
    }

    // Pole `index` (a copy sharing its points).
    pub(super) fn pole_point(
        &self,
        poles: &HArray1<MultiPoint>,
        index: i32,
    ) -> Result<MultiPoint, Failure> {
        poles
            .array1()
            .value(index)
            .cloned()
            .map_err(|error| array_failure(error, "AppParCurves_MultiCurve"))
    }

    /// `Curve(CuIndex, TabPnt)`: the 3D poles of curve `cu_index` into `tab_pnt(1..)`
    /// (indexes from 1 whatever its lower bound); Standard_OutOfRange for
    /// `cu_index <= 0`; `curve.curve_3d(1, &mut poles)?`.
    pub fn curve_3d(&self, cu_index: i32, tab_pnt: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        self.fill_curve(cu_index, tab_pnt, |pole| pole.point(cu_index))
    }

    /// `Curve(CuIndex, TabPnt2d)`: the 2D poles of curve `cu_index`;
    /// `curve.curve_2d(2, &mut poles)?`.
    pub fn curve_2d(&self, cu_index: i32, tab_pnt: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        self.fill_curve(cu_index, tab_pnt, |pole| pole.point2d(cu_index))
    }

    fn fill_curve<T>(
        &self,
        cu_index: i32,
        tab: &mut Array1<'_, T>,
        read: impl Fn(&MultiPoint) -> Result<T, Failure>,
    ) -> Result<(), Failure> {
        if cu_index <= 0 {
            return Err(Failure::out_of_range(&format!(
                "AppParCurves_MultiCurve::Curve: curve {cu_index}"
            )));
        }
        let poles: &HArray1<MultiPoint> = self.poles("Curve")?;
        for i in 1..=poles.array1().length() {
            let value: T = read(&self.pole_point(poles, i)?)?;
            tab.set_value(i, value)
                .map_err(|error| array_failure(error, "AppParCurves_MultiCurve::Curve"))?;
        }
        Ok(())
    }

    /// `Value(Index)`: pole `index` (a copy sharing its points); Standard_OutOfRange
    /// outside `1..=NbPoles()`; `curve.value(1)?`.
    pub fn value(&self, index: i32) -> Result<MultiPoint, Failure> {
        let poles: &HArray1<MultiPoint> = self.poles("Value")?;
        let length: i32 = poles.array1().length();
        if index <= 0 || index > length {
            return Err(Failure::out_of_range(&format!(
                "AppParCurves_MultiCurve::Value: index {index}; expected 1..={length}"
            )));
        }
        self.pole_point(poles, index)
    }

    /// `Pole(CuIndex, Nieme)`: the native check raises only when both indexes are
    /// `<= 0` (`&&`); `curve.pole(1, 2)?`.
    pub fn pole(&self, cu_index: i32, nieme: i32) -> Result<Point3, Failure> {
        self.check_pole(cu_index, nieme)?;
        self.pole_point(self.poles("Pole")?, nieme)?.point(cu_index)
    }

    /// `Pole2d(CuIndex, Nieme)`; see [`MultiCurve::pole`]; `curve.pole2d(2, 1)?`.
    pub fn pole2d(&self, cu_index: i32, nieme: i32) -> Result<Point2, Failure> {
        self.check_pole(cu_index, nieme)?;
        self.pole_point(self.poles("Pole2d")?, nieme)?
            .point2d(cu_index)
    }

    fn check_pole(&self, cu_index: i32, nieme: i32) -> Result<(), Failure> {
        if cu_index <= 0 && nieme <= 0 {
            return Err(Failure::out_of_range(&format!(
                "AppParCurves_MultiCurve::Pole: ({cu_index}, {nieme})"
            )));
        }
        Ok(())
    }

    /// `Transform(CuIndex, x, dx, y, dy, z, dz)`: transforms curve `cu_index` in
    /// every pole (a pole shared twice is transformed twice); Standard_OutOfRange for
    /// a 2D curve; `curve.transform(1, [0.0, 1.0, 0.0, 1.0, 0.0, 1.0])?`.
    pub fn transform(&self, cu_index: i32, coefficients: [f64; 6]) -> Result<(), Failure> {
        if self.dimension(cu_index)? != 3 {
            return Err(failure(
                "Standard_OutOfRange",
                "AppParCurves_MultiCurve::Transform: a 2D curve",
            ));
        }
        let poles: &HArray1<MultiPoint> = self.poles("Transform")?;
        (1..=poles.array1().length())
            .try_for_each(|i| self.pole_point(poles, i)?.transform(cu_index, coefficients))
    }

    /// `Transform2d(CuIndex, x, dx, y, dy)`; `curve.transform2d(2, [0.0, 1.0, 0.0, 1.0])?`.
    pub fn transform2d(&self, cu_index: i32, coefficients: [f64; 4]) -> Result<(), Failure> {
        if self.dimension(cu_index)? != 2 {
            return Err(failure(
                "Standard_OutOfRange",
                "AppParCurves_MultiCurve::Transform2d: a 3D curve",
            ));
        }
        let poles: &HArray1<MultiPoint> = self.poles("Transform2d")?;
        (1..=poles.array1().length()).try_for_each(|i| {
            self.pole_point(poles, i)?
                .transform2d(cu_index, coefficients)
        })
    }

    // The poles of curve `cu_index` of dimension `expected` (Standard_OutOfRange otherwise).
    pub(super) fn poles_3d(&self, cu_index: i32) -> Result<Array1<'static, Point3>, Failure> {
        self.expect_dimension(cu_index, 3)?;
        let mut poles: Array1<'static, Point3> = self.pole_array()?;
        self.curve_3d(cu_index, &mut poles)?;
        Ok(poles)
    }

    pub(super) fn poles_2d(&self, cu_index: i32) -> Result<Array1<'static, Point2>, Failure> {
        self.expect_dimension(cu_index, 2)?;
        let mut poles: Array1<'static, Point2> = self.pole_array()?;
        self.curve_2d(cu_index, &mut poles)?;
        Ok(poles)
    }

    fn pole_array<T: Default>(&self) -> Result<Array1<'static, T>, Failure> {
        Array1::with_bounds(1, self.poles("Value")?.array1().length())
            .map_err(|error| array_failure(error, "AppParCurves_MultiCurve"))
    }

    fn expect_dimension(&self, cu_index: i32, expected: i32) -> Result<(), Failure> {
        if self.dimension(cu_index)? != expected {
            return Err(Failure::out_of_range(&format!(
                "AppParCurves_MultiCurve: curve {cu_index} is not of dimension {expected}"
            )));
        }
        Ok(())
    }
}

impl MultiCurveEval for MultiCurve {
    fn nb_poles(&self) -> i32 {
        self.tab_point
            .get()
            .map_or(0, |poles| poles.array1().length())
    }

    fn degree(&self) -> Result<i32, Failure> {
        Ok(self.poles("Degree")?.array1().length() - 1)
    }

    fn value_3d(&self, cu_index: i32, u: f64) -> Result<Point3, Failure> {
        bsplclib::bezier_d0(u, &self.poles_3d(cu_index)?, None)
    }

    fn value_2d(&self, cu_index: i32, u: f64) -> Result<Point2, Failure> {
        bsplclib::bezier_d0(u, &self.poles_2d(cu_index)?, None)
    }

    fn d1_3d(&self, cu_index: i32, u: f64) -> Result<(Point3, Vector3), Failure> {
        bsplclib::bezier_d1(u, &self.poles_3d(cu_index)?, None)
    }

    fn d1_2d(&self, cu_index: i32, u: f64) -> Result<(Point2, Vector2), Failure> {
        bsplclib::bezier_d1(u, &self.poles_2d(cu_index)?, None)
    }

    fn d2_3d(&self, cu_index: i32, u: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        bsplclib::bezier_d2(u, &self.poles_3d(cu_index)?, None)
    }

    fn d2_2d(&self, cu_index: i32, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        bsplclib::bezier_d2(u, &self.poles_2d(cu_index)?, None)
    }

    // The native Dump prints the counts only (the pole loop is commented out).
    fn dump(&self, stream: &mut dyn fmt::Write) -> fmt::Result {
        let nb_curves: i32 = self.nb_curves().map_err(|_| fmt::Error)?;
        let degree: i32 = MultiCurveEval::degree(self).map_err(|_| fmt::Error)?;
        writeln!(stream, "AppParCurves_MultiCurve dump:")?;
        writeln!(
            stream,
            " It contains {nb_curves} Bezier curves of degree {degree}"
        )?;
        writeln!(stream, " The poles are: ")
    }
}
