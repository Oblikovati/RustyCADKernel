// Rust translation of ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_Geom.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeAnalysis_Geom`: the nearest plane of a point cloud and a transformation read from a
//! 3 x 4 matrix.
use rustycad_tkernel::ncollection::{Array1, HArray2};
use rustycad_tkernel::standard::real::real_first;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::gprop::{PEquation, PEquationType};
use rustycad_tkmath::gp::{
    Axis3, Direction3, GeneralTransform3, Plane3, Point3, Transform3, Vector3, Xyz,
};

/// `ShapeAnalysis_Geom`: analysis of primitive geometric objects (static members only).
///
/// ```
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tkshhealing::shape_analysis::ShapeAnalysisGeom;
/// let mut points = Array1::with_bounds(1, 4).unwrap();
/// points[1] = Point3::new(0.0, 0.0, 0.0);
/// points[2] = Point3::new(1.0, 0.0, 0.0);
/// points[3] = Point3::new(1.0, 1.0, 0.0);
/// points[4] = Point3::new(0.0, 1.0, 0.0);
/// let (plane, dmax) = ShapeAnalysisGeom::nearest_plane(&points).unwrap().unwrap();
/// assert!(dmax < 1e-12 && plane.distance(Point3::new(0.5, 0.5, 0.0)) < 1e-12);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeAnalysisGeom;

impl ShapeAnalysisGeom {
    /// `NearestPlane(Pnts, aPln, Dmax)`: the plane through the barycentre normal to the
    /// principal axis of smallest extent, when that extent is at most half of each other
    /// one, with the largest distance of the points to it; `None` (native false) for a
    /// cloud that fits no plane. The native `Dmax` is `RealFirst()` when no point is
    /// farther, which the loop always overrides for a non-empty cloud;
    /// `ShapeAnalysisGeom::nearest_plane(&points)?`.
    pub fn nearest_plane(points: &Array1<'_, Point3>) -> Result<Option<(Plane3, f64)>, Failure> {
        let equation = PEquation::new(points, 0.0);
        if equation.get_type() == PEquationType::None {
            return Ok(None);
        }
        let dev = [
            equation.extent(1)?,
            equation.extent(2)?,
            equation.extent(3)?,
        ];
        let axis = smallest_extent_axis(dev);
        if !is_flat(dev, axis) {
            return Ok(None);
        }
        let normal = Direction3::from_vector(equation.principal_axis(axis as i32 + 1)?)?;
        let plane = Plane3::new(equation.barycentre(), normal)?;
        let mut max_dist = real_first();
        for point in points.iter() {
            let d = plane.distance(*point);
            if max_dist < d {
                max_dist = d;
            }
        }
        Ok(Some((plane, max_dist)))
    }

    /// `PositionTrsf(coefs, trsf, unit, prec)`: the transformation of a 3 x 4 matrix
    /// (the rotation columns must have equal norms within `prec` times their mean and be
    /// orthogonal within `prec`); the translation is multiplied by `unit`. A null matrix
    /// gives the identity and true; a matrix that is not a similarity gives false with the
    /// identity. Natively `SetScale` resets the rotation that `SetTransformation` set, so a
    /// scaled matrix keeps only its scale and translation; a coefficient outside the
    /// matrix bounds raises Standard_OutOfRange;
    /// `ShapeAnalysisGeom::position_trsf(&coefs, 1.0, 1e-7)?`.
    pub fn position_trsf(
        coefs: &Handle<HArray2<f64>>,
        unit: f64,
        prec: f64,
    ) -> Result<(bool, Transform3), Failure> {
        let mut trsf = Transform3::default();
        let Some(matrix) = coefs.get() else {
            return Ok((true, trsf));
        };
        let gtrsf = general_transform(matrix)?;
        let (v1, v2, v3) = (column(&gtrsf, 1)?, column(&gtrsf, 2)?, column(&gtrsf, 3)?);
        let norms = [v1.modulus(), v2.modulus(), v3.modulus()];
        let Some(mm) = similarity_scale(norms, prec) else {
            return Ok((false, trsf));
        };
        let (v1, v2, v3) = (v1 / norms[0], v2 / norms[1], v3 / norms[2]);
        if v1.dot(v2).abs() > prec || v2.dot(v3).abs() > prec || v3.dot(v1).abs() > prec {
            return Ok((false, trsf));
        }
        set_rotation(&mut trsf, v1, v2, v3)?;
        if (mm - 1.0).abs() > prec {
            trsf.set_scale(Point3::new(0.0, 0.0, 0.0), mm)?;
        }
        let mut tp = Vector3::from(gtrsf.translation_part());
        if unit != 1.0 {
            tp *= unit;
        }
        if tp.x() != 0.0 || tp.y() != 0.0 || tp.z() != 0.0 {
            trsf.set_translation_part(tp);
        }
        Ok((true, trsf))
    }
}

/// The axis (0-based) of the smallest extent, with the native tie rules.
fn smallest_extent_axis(dev: [f64; 3]) -> usize {
    if dev[0] < dev[1] {
        if dev[0] < dev[2] { 0 } else { 2 }
    } else if dev[1] < dev[2] {
        1
    } else {
        2
    }
}

/// The smallest extent is at most half of each other one (the order of the native
/// comparisons does not matter: they only read).
fn is_flat(dev: [f64; 3], axis: usize) -> bool {
    let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
    !(2.0 * dev[axis] > dev[a] || 2.0 * dev[axis] > dev[b])
}

/// The `gp_GTrsf` of the 3 x 4 coefficients, set cell by cell as natively.
fn general_transform(matrix: &HArray2<f64>) -> Result<GeneralTransform3, Failure> {
    let cells = matrix.array2();
    let mut gtrsf = GeneralTransform3::default();
    for i in 1..=3 {
        for j in 1..=4 {
            let value = *cells.value(i, j).map_err(|_| out_of_range(i, j))?;
            gtrsf.set_value(i, j, value)?;
        }
    }
    Ok(gtrsf)
}

/// Column `j` of the vectorial part.
fn column(gtrsf: &GeneralTransform3, j: i32) -> Result<Xyz, Failure> {
    let cell = |i: i32| gtrsf.value(i, j);
    Ok(Xyz::new(cell(1)?, cell(2)?, cell(3)?))
}

/// The mean norm of the three columns when each is at least `prec` and within `prec`
/// times the mean of it.
fn similarity_scale(norms: [f64; 3], prec: f64) -> Option<f64> {
    if norms.iter().any(|&m| m < prec) {
        return None;
    }
    let mm = (norms[0] + norms[1] + norms[2]) / 3.0;
    let pmm = prec * mm;
    if norms.iter().any(|&m| (m - mm).abs() > pmm) {
        return None;
    }
    Some(mm)
}

/// `SetTransformation(gp_Ax3(origin, d3, d1))`, the Y axis reversed for a left-handed
/// frame, unless the columns are exactly the unit axes.
fn set_rotation(trsf: &mut Transform3, v1: Xyz, v2: Xyz, v3: Xyz) -> Result<(), Failure> {
    let unit = |v: Xyz, x: f64, y: f64, z: f64| v.x() == x && v.y() == y && v.z() == z;
    if unit(v1, 1.0, 0.0, 0.0) && unit(v2, 0.0, 1.0, 0.0) && unit(v3, 0.0, 0.0, 1.0) {
        return Ok(());
    }
    let (d1, d2, mut d3) = (
        Direction3::from_xyz(v1)?,
        Direction3::from_xyz(v2)?,
        Direction3::from_xyz(v3)?,
    );
    let mut axes = Axis3::new(Point3::new(0.0, 0.0, 0.0), d3, d1)?;
    d3.cross(d1)?;
    if d3.dot(d2) < 0.0 {
        axes.y_reverse();
    }
    trsf.set_transformation_to(axes);
    Ok(())
}

fn out_of_range(i: i32, j: i32) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!(
            "ShapeAnalysis_Geom::PositionTrsf: coefficient ({i}, {j}); expected a matrix of rows 1..3 and columns 1..4"
        ),
    )
}
