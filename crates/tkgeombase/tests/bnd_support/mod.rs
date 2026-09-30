//! Construction shorthands shared by the translated BndLib and GeomBndLib suites: the
//! gp/Geom values the C++ tests build inline, and the structured `Bnd_Box::Get()`.
#![allow(non_snake_case, dead_code)]
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3};

/// `gp_Pnt(x, y, z)`.
pub fn pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// `gp_Dir(x, y, z)`.
pub fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

/// `gp_Pnt2d(x, y)`.
pub fn pnt2d(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

/// `gp_Dir2d(x, y)`.
pub fn dir2d(x: f64, y: f64) -> Direction2 {
    Direction2::new(x, y).unwrap()
}

/// The structured `Get()`: `[Xmin, Xmax, Ymin, Ymax, Zmin, Zmax]`.
pub fn get3(bounds: &BoundingBox3) -> [f64; 6] {
    let l = bounds.limits();
    [l.x_min, l.x_max, l.y_min, l.y_max, l.z_min, l.z_max]
}

/// The structured `Get()` of a 2D box: `[Xmin, Xmax, Ymin, Ymax]`.
pub fn get2(bounds: &BoundingBox2) -> [f64; 4] {
    let l = bounds.limits();
    [l.x_min, l.x_max, l.y_min, l.y_max]
}

/// A `Handle<dyn Geom_Curve>` of a concrete curve.
pub fn curve<T: GeomCurve + RuntimeTyped + rustycad_tkernel::standard::Transient>(
    value: T,
) -> Handle<dyn GeomCurve> {
    Handle::new(value).cast()
}

/// A `Handle<dyn Geom2d_Curve>` of a concrete curve.
pub fn curve2d<T: Geom2dCurve + RuntimeTyped + rustycad_tkernel::standard::Transient>(
    value: T,
) -> Handle<dyn Geom2dCurve> {
    Handle::new(value).cast()
}

/// A `Handle<dyn Geom_Surface>` of a concrete surface.
pub fn surface<T: GeomSurface + RuntimeTyped + rustycad_tkernel::standard::Transient>(
    value: T,
) -> Handle<dyn GeomSurface> {
    Handle::new(value).cast()
}

/// `NCollection_Array1<T>(1, n)` filled from `values`.
pub fn array1<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, value) in (1..).zip(values) {
        array[i] = *value;
    }
    array
}

/// `NCollection_Array2<gp_Pnt>(1, rows, 1, cols)` filled row by row from `values`.
pub fn array2(rows: i32, cols: i32, values: &[Point3]) -> Array2<'static, Point3> {
    let mut array = Array2::with_bounds(1, rows, 1, cols).unwrap();
    for i in 1..=rows {
        for j in 1..=cols {
            array[(i, j)] = values[((i - 1) * cols + (j - 1)) as usize];
        }
    }
    array
}
