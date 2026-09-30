// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_MultiPoint.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_MultiPoint`: the 3D and 2D points of one parameter of a MultiLine
//! (or one pole of a MultiCurve).
use crate::native_failure::{array_failure, failure};
use crate::native_text::stream_real;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Point3};
use std::fmt;

/// `AppParCurves_MultiPoint`: `NbPoints()` 3D points (indexes `1..=nbP`) then
/// `NbPoints2d()` 2D points (indexes `nbP + 1..=nbP + nbP2d`).
///
/// The points live in shared arrays: a copy (`clone`) shares them, as the native
/// copy shares its handles, so setting a point of a copy changes the original.
///
/// ```
/// use rustycad_tkgeombase::app_par_curves::MultiPoint;
/// use rustycad_tkmath::gp::{Point2, Point3};
/// let point = MultiPoint::with_counts(1, 1)?;
/// point.set_point(1, Point3::new(1.0, 2.0, 3.0))?;
/// point.set_point2d(2, Point2::new(4.0, 5.0))?;
/// assert_eq!((point.dimension(1)?, point.dimension(2)?), (3, 2));
/// assert_eq!(point.clone().point(1)?, Point3::new(1.0, 2.0, 3.0));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct MultiPoint {
    points: Handle<HArray1<Point3>>,
    points2d: Handle<HArray1<Point2>>,
    nb_p: i32,
    nb_p2d: i32,
}

// A shared array of `count` default points, null for 0 as natively.
fn new_array<T: Default + Send + Sync + 'static>(
    count: i32,
) -> Result<Handle<HArray1<T>>, Failure> {
    if count == 0 {
        return Ok(Handle::default());
    }
    let array: HArray1<T> = HArray1::with_bounds(1, count)
        .map_err(|error| array_failure(error, "AppParCurves_MultiPoint"))?;
    Ok(Handle::new(array))
}

/// A shared copy of `values` reindexed from 1 (the native `SetValue(i, tab.Value(Lower + i - 1))`
/// copies of AppParCurves_MultiPoint and AppDef_MultiPointConstraint).
pub(crate) fn copy_array<T: Clone + Default + Send + Sync + 'static>(
    values: &Array1<'_, T>,
) -> Handle<HArray1<T>> {
    let mut copy: Array1<'static, T> = Array1::with_size(values.len());
    copy.update_lower_bound(1);
    copy.iter_mut()
        .zip(values.iter())
        .for_each(|(slot, value)| *slot = value.clone());
    Handle::new(HArray1::from_owned(copy))
}

fn out_of_range(member: &str, index: i32, range: String) -> Failure {
    Failure::out_of_range(&format!(
        "AppParCurves_MultiPoint::{member}() - wrong index {index}; expected {range}"
    ))
}

impl MultiPoint {
    /// `AppParCurves_MultiPoint()`: no points; `MultiPoint::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `AppParCurves_MultiPoint(NbPoints, NbPoints2d)`: origin points, to be set with
    /// SetPoint and SetPoint2d; `MultiPoint::with_counts(2, 1)?`.
    pub fn with_counts(nb_points: i32, nb_points2d: i32) -> Result<Self, Failure> {
        Ok(Self {
            points: new_array(nb_points)?,
            points2d: new_array(nb_points2d)?,
            nb_p: nb_points,
            nb_p2d: nb_points2d,
        })
    }

    /// `AppParCurves_MultiPoint(tabP)`: 3D points only; `MultiPoint::from_points(&points)`.
    pub fn from_points(points: &Array1<'_, Point3>) -> Self {
        Self::from_points_and_2d_arrays(copy_array(points), points.length(), Handle::default(), 0)
    }

    /// `AppParCurves_MultiPoint(tabP2d)`: 2D points only; `MultiPoint::from_points2d(&points)`.
    pub fn from_points2d(points2d: &Array1<'_, Point2>) -> Self {
        Self::from_points_and_2d_arrays(
            Handle::default(),
            0,
            copy_array(points2d),
            points2d.length(),
        )
    }

    /// `AppParCurves_MultiPoint(tabP, tabP2d)`: 3D then 2D points (empty arrays are
    /// allocated, not null); `MultiPoint::from_points_and_2d(&p3d, &p2d)`.
    pub fn from_points_and_2d(points: &Array1<'_, Point3>, points2d: &Array1<'_, Point2>) -> Self {
        Self::from_points_and_2d_arrays(
            copy_array(points),
            points.length(),
            copy_array(points2d),
            points2d.length(),
        )
    }

    fn from_points_and_2d_arrays(
        points: Handle<HArray1<Point3>>,
        nb_p: i32,
        points2d: Handle<HArray1<Point2>>,
        nb_p2d: i32,
    ) -> Self {
        Self {
            points,
            points2d,
            nb_p,
            nb_p2d,
        }
    }

    /// `SetPoint(Index, Point)`: Standard_OutOfRange outside `1..=NbPoints()`;
    /// `point.set_point(1, p)?`.
    pub fn set_point(&self, index: i32, point: Point3) -> Result<(), Failure> {
        if index <= 0 || index > self.nb_p {
            return Err(out_of_range(
                "SetPoint",
                index,
                format!("1..={}", self.nb_p),
            ));
        }
        self.points
            .change_array1()
            .set_value(index, point)
            .map_err(|error| array_failure(error, "AppParCurves_MultiPoint::SetPoint"))
    }

    /// `Point(Index)`: Standard_OutOfRange outside `1..=NbPoints()`; `point.point(1)?`.
    pub fn point(&self, index: i32) -> Result<Point3, Failure> {
        if index <= 0 || index > self.nb_p {
            return Err(out_of_range("Point", index, format!("1..={}", self.nb_p)));
        }
        let points = self.points.array1();
        points
            .value(index)
            .copied()
            .map_err(|error| array_failure(error, "AppParCurves_MultiPoint::Point"))
    }

    /// `SetPoint2d(Index, Point)`: `index` counts the 3D points first;
    /// Standard_OutOfRange outside `NbPoints() + 1..=NbPoints() + NbPoints2d()`;
    /// `point.set_point2d(2, p)?`.
    pub fn set_point2d(&self, index: i32, point: Point2) -> Result<(), Failure> {
        if index <= self.nb_p || index > self.nb_p + self.nb_p2d {
            return Err(out_of_range("SetPoint2d", index, self.range2d()));
        }
        self.points2d
            .change_array1()
            .set_value(index - self.nb_p, point)
            .map_err(|error| array_failure(error, "AppParCurves_MultiPoint::SetPoint2d"))
    }

    /// `Point2d(Index)`; see [`MultiPoint::set_point2d`]; `point.point2d(2)?`.
    pub fn point2d(&self, index: i32) -> Result<Point2, Failure> {
        if index <= self.nb_p || index > self.nb_p + self.nb_p2d {
            return Err(out_of_range("Point2d", index, self.range2d()));
        }
        let points = self.points2d.array1();
        points
            .value(index - self.nb_p)
            .copied()
            .map_err(|error| array_failure(error, "AppParCurves_MultiPoint::Point2d"))
    }

    fn range2d(&self) -> String {
        format!("{}..={}", self.nb_p + 1, self.nb_p + self.nb_p2d)
    }

    /// `Dimension(Index)`: 3 up to NbPoints() (index 0 included), 2 above;
    /// Standard_OutOfRange below 0 or above the total; `point.dimension(1)?`.
    pub fn dimension(&self, index: i32) -> Result<i32, Failure> {
        if index < 0 || index > self.nb_p + self.nb_p2d {
            return Err(Failure::out_of_range(&format!(
                "AppParCurves_MultiPoint::Dimension: index {index}; expected 0..={}",
                self.nb_p + self.nb_p2d
            )));
        }
        Ok(if index <= self.nb_p { 3 } else { 2 })
    }

    /// `NbPoints()`; `point.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.nb_p
    }

    /// `NbPoints2d()`; `point.nb_points2d()`.
    pub fn nb_points2d(&self) -> i32 {
        self.nb_p2d
    }

    /// `Transform(CuIndex, x, dx, y, dy, z, dz)`: `(x + X dx, y + Y dy, z + Z dz)` for
    /// the 3D point `cu_index`; Standard_OutOfRange for a 2D index;
    /// `point.transform(1, [0.0, 1.0, 0.0, 1.0, 0.0, 2.0])?`.
    pub fn transform(&self, cu_index: i32, [x, dx, y, dy, z, dz]: [f64; 6]) -> Result<(), Failure> {
        if self.dimension(cu_index)? != 3 {
            return Err(failure(
                "Standard_OutOfRange",
                &format!("AppParCurves_MultiPoint::Transform: {cu_index} is 2D"),
            ));
        }
        let p: Point3 = self.point(cu_index)?;
        self.set_point(
            cu_index,
            Point3::new(x + p.x() * dx, y + p.y() * dy, z + p.z() * dz),
        )
    }

    /// `Transform2d(CuIndex, x, dx, y, dy)`: the 2D version of [`MultiPoint::transform`];
    /// `point.transform2d(2, [0.0, 1.0, 0.0, 1.0])?`.
    pub fn transform2d(&self, cu_index: i32, [x, dx, y, dy]: [f64; 4]) -> Result<(), Failure> {
        if self.dimension(cu_index)? != 2 {
            return Err(failure(
                "Standard_OutOfRange",
                &format!("AppParCurves_MultiPoint::Transform2d: {cu_index} is 3D"),
            ));
        }
        let p: Point2 = self.point2d(cu_index)?;
        self.set_point2d(cu_index, Point2::new(x + p.x() * dx, y + p.y() * dy))
    }

    /// `Dump(o)`: the native text, numbers as `%g`; `point.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        writeln!(stream, "AppParCurves_MultiPoint dump:")?;
        writeln!(
            stream,
            "It contains {} 3d points and {} 2d points.",
            self.nb_p, self.nb_p2d
        )?;
        if self.nb_p > 0 {
            for (point, i) in self.points.array1().iter().zip(1..) {
                writeln!(stream, "3D-Point #{i}")?;
                writeln!(stream, " Pole x = {}", stream_real(point.x()))?;
                writeln!(stream, " Pole y = {}", stream_real(point.y()))?;
                writeln!(stream, " Pole z = {}", stream_real(point.z()))?;
            }
        }
        if self.nb_p2d > 0 {
            for (point, i) in self.points2d.array1().iter().zip(1..) {
                writeln!(stream, "2D-Point #{i}")?;
                writeln!(stream, " Pole x = {}", stream_real(point.x()))?;
                writeln!(stream, " Pole y = {}", stream_real(point.y()))?;
            }
        }
        Ok(())
    }
}

/// `operator<<(Standard_OStream&, const AppParCurves_MultiPoint&)`: the Dump text.
impl fmt::Display for MultiPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dump(formatter)
    }
}
