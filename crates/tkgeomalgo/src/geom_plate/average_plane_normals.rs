// Rust translation of the constructor GeomPlate_BuildAveragePlane(Normals, Pts) of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildAveragePlane.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The average plane normal to the "best vector" of a set of normals.
use super::build_average_plane::{GeomPlateBuildAveragePlane, point_at};
use super::live;
use rustycad_tkernel::ncollection::{Array1, HArray1, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkgeombase::geom_lib::axe_of_inertia;
use rustycad_tkmath::elslib::plane_parameters;
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Plane3, Point3, Vector3};

/// The number of rotation steps on each side of a pair bisector.
const NINT: i32 = 50;

/// `Normals(i)`, Standard_OutOfRange outside the sequence.
fn normal_at(normals: &Sequence<Vector3>, index: i32) -> Result<Vector3, Failure> {
    normals.value(index.max(0) as usize).copied()
}

/// The best vector of two normals: their unit bisector, or the longer one normalized
/// when they cancel.
fn best_of_two(n1: Vector3, n2: Vector3) -> Result<Vector3, Failure> {
    let mut best: Vector3 = n1 + n2;
    let sq_magn: f64 = best.square_magnitude();
    if sq_magn < precision::square_confusion() {
        let (sq1, sq2) = (n1.square_magnitude(), n2.square_magnitude());
        return if sq1 > sq2 {
            n1.normalized()
        } else {
            n2.normalized()
        };
    }
    best /= sq_magn.sqrt();
    Ok(best)
}

/// Half of 1.2 times the greatest angle between two normals.
fn half_spread(normals: &Sequence<Vector3>) -> Result<f64, Failure> {
    let nn: i32 = normals.length();
    let mut max_angle: f64 = 0.0;
    for i in 1..nn {
        for j in i + 1..=nn {
            let angle: f64 = normal_at(normals, i)?.angle(normal_at(normals, j)?)?;
            if angle > max_angle {
                max_angle = angle;
            }
        }
    }
    max_angle *= 1.2;
    max_angle /= 2.0;
    Ok(max_angle)
}

/// The smallest scalar product of `vec` with the normals.
fn min_scal(vec: Vector3, normals: &Sequence<Vector3>) -> f64 {
    let mut min_scal: f64 = real_last();
    for normal in normals.iter() {
        let scal: f64 = vec * *normal;
        if scal < min_scal {
            min_scal = scal;
        }
    }
    min_scal
}

/// The best vector near the bisector of normals `i` and `j`: the rotation of the
/// bisector around the axis normal to it in their plane, by steps of `max_angle /
/// NINT` over `[-max_angle, max_angle]`, whose smallest scalar product with the
/// normals is the greatest. `None` when the two normals cancel.
fn best_near_pair(
    normals: &Sequence<Vector3>,
    (i, j): (i32, i32),
    max_angle: f64,
) -> Result<Option<(f64, Vector3)>, Failure> {
    let step: f64 = max_angle / f64::from(NINT);
    let (ni, nj) = (normal_at(normals, i)?, normal_at(normals, j)?);
    let mut vec: Vector3 = ni + nj;
    let sq_magn: f64 = vec.square_magnitude();
    if sq_magn < precision::square_confusion() {
        return Ok(None);
    }
    vec /= sq_magn.sqrt();
    let cross1 = Direction3::from_vector(ni ^ nj)?;
    let cross2 = Direction3::from_vector(vec ^ Vector3::from(cross1))?;
    let axe = Axis1::new(Point3::new(0.0, 0.0, 0.0), cross2);
    let mut vec1: Vector3 = vec.rotated(axe, -max_angle)?;
    let (mut opt_scal, mut opt_vec) = (real_first(), Vector3::new(0.0, 0.0, 0.0));
    for _ in 0..=2 * NINT {
        vec1.rotate(axe, step)?;
        let scal: f64 = min_scal(vec1, normals);
        if scal > opt_scal {
            (opt_scal, opt_vec) = (scal, vec1);
        }
    }
    Ok(Some((opt_scal, opt_vec)))
}

/// The best vector of three normals or more (or none): the best of the pair
/// optima. Standard_OutOfRange when no pair gives one (`OptVec(0)` natively).
fn best_of_many(normals: &Sequence<Vector3>) -> Result<Vector3, Failure> {
    let nn: i32 = normals.length();
    let max_angle: f64 = half_spread(normals)?;
    let (mut best_scal, mut best_vec): (f64, Option<Vector3>) = (real_first(), None);
    for i in 1..nn {
        for j in i + 1..=nn {
            // A skipped pair keeps OptScal(k) = RealFirst(), which never wins.
            let Some((scal, vec)) = best_near_pair(normals, (i, j), max_angle)? else {
                continue;
            };
            if scal > best_scal {
                (best_scal, best_vec) = (scal, Some(vec));
            }
        }
    }
    best_vec.ok_or_else(|| {
        errors::out_of_range(&format!(
            "GeomPlate_BuildAveragePlane: no best vector among {nn} normals (OptVec(0)); expected two non-opposite normals"
        ))
    })
}

impl GeomPlateBuildAveragePlane {
    /// The plane through the inertia centre of `pts` normal to the "best vector" of
    /// `normals`, the vector of greatest smallest scalar product with them
    /// (`GeomPlate_BuildAveragePlane(Normals, Pts)`); its X axis is the inertia X axis
    /// projected on it. Standard_ConstructionError for a null best vector or one
    /// parallel to the inertia X axis, Standard_OutOfRange without normals.
    /// `GeomPlateBuildAveragePlane::with_normals(&normals, &points)?`.
    pub fn with_normals(
        normals: &Sequence<Vector3>,
        pts: &Handle<HArray1<Point3>>,
    ) -> Result<Self, Failure> {
        let best_vec: Vector3 = match normals.length() {
            1 => normal_at(normals, 1)?,
            2 => best_of_two(normal_at(normals, 1)?, normal_at(normals, 2)?)?,
            _ => best_of_many(normals)?,
        };
        let mut bap = Self::empty(pts);
        bap.plane_from_best(best_vec)?;
        Ok(bap)
    }

    /// The plane of the best vector and the box of the points projected on it.
    fn plane_from_best(&mut self, best_vec: Vector3) -> Result<(), Failure> {
        let pts = self.pts.clone();
        let pts: &HArray1<Point3> = live(&pts, "the points")?;
        let pts_array: Array1<'static, Point3> = pts.array1().clone();
        let axe = axe_of_inertia(&pts_array, 1.0e-7)?.axe;
        let best_dir = Direction3::from_vector(best_vec)?;
        let mut x_dir: Direction3 = best_dir.crossed(axe.x_direction())?;
        x_dir.cross(best_dir)?;
        let plane = GeomPlane::new(Axis3::new(axe.location(), best_dir, x_dir)?);
        let pln: Plane3 = plane.pln();
        (self.umax, self.vmax) = plane_parameters(pln.position(), axe.location());
        (self.umin, self.vmin) = (self.umax, self.vmax);
        for i in 1..=pts.array1().length() {
            let p: Point3 = point_at(pts, i)?;
            let a_vec = Vector3::between(pln.location(), p);
            let mut norm_vec = Vector3::from(pln.axis().direction());
            norm_vec = (a_vec * norm_vec) * norm_vec;
            let (u, v) = plane_parameters(pln.position(), p.translated(-norm_vec));
            self.widen_normals_box(u, v);
        }
        self.ox = Vector3::from(pln.x_axis().direction());
        self.oy = Vector3::from(pln.y_axis().direction());
        self.plane = Handle::new(plane);
        Ok(())
    }

    /// Widens the box to `(u, v)` in the order of this constructor's tests.
    fn widen_normals_box(&mut self, u: f64, v: f64) {
        if u > self.umax {
            self.umax = u;
        }
        if u < self.umin {
            self.umin = u;
        }
        if v > self.vmax {
            self.vmax = v;
        }
        if v < self.vmin {
            self.vmin = v;
        }
    }
}
