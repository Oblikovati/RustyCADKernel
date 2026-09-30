// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_SurfToAnaSurf.cxx
// (TryCylinderByGaussField, GetLSGap, GetCylByLS) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cylinder of `GeomConvert_SurfToAnaSurf` from the Gauss field of a surface
//! (constant curvatures, one of them zero), refined by least squares.
use super::FuncCylinderLSDist;
use crate::array_access::matrix;
use crate::geom_lib::live;
use crate::geom_lprop::SLProps;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface, SurfaceBounds};
use rustycad_tkmath::gp::{Axis1, Axis3, Cylinder3, Direction3, Plane3, Point3, Vector3, Xyz};
use rustycad_tkmath::math::{Matrix, Powell, Pso, Vector};

/// The running sums of the Gauss field samples.
struct GaussSums {
    n: i32,
    av_max_curv: f64,
    av_min_curv: f64,
    av_r: f64,
    av_dir: Xyz,
    sign: f64,
    radii: Vec<f64>,
}

// One sample of the Gauss field: the absolute min and max curvatures (swapped
// when |min| > |max|, the sign then set to -1 for good) and the min direction.
struct GaussSample {
    min_curv: f64,
    max_curv: f64,
    min_dir: Direction3,
    swapped: bool,
}

/// `TryCylinderByGaussField(theSurf, theU1, theU2, theV1, theV2, theToler, theNbU,
/// theNbV, theLeastSquare)`: the cylinder of the average radius and minimum
/// curvature direction of NbU x NbV cell centers where the Gaussian curvature
/// vanishes, the max curvature is constant and the min one zero; refined by least
/// squares on request; null otherwise.
pub(super) fn try_cylinder_by_gauss_field(
    surf: &Handle<dyn GeomSurface>,
    (u1, u2, v1, v2): SurfaceBounds,
    toler: f64,
    (nb_u, nb_v): (i32, i32),
    least_square: bool,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let du: f64 = (u2 - u1) / f64::from(nb_u);
    let dv: f64 = (v2 - v1) / f64::from(nb_v);
    let tol: f64 = 1.0e3 * toler;
    // Quirk: the native point array has NbU * NbU entries.
    let mut points: Vec<Xyz> = Vec::with_capacity((nb_u * nb_u).max(0) as usize);
    let mut props = SLProps::new(surf.clone(), 2, precision::confusion())?;
    let mut sums = GaussSums {
        n: 0,
        av_max_curv: 0.0,
        av_min_curv: 0.0,
        av_r: 0.0,
        av_dir: Xyz::default(),
        sign: 1.0,
        radii: Vec::new(),
    };
    let mut u: f64 = u1 + du / 2.0;
    for _ in 1..=nb_u {
        let mut v: f64 = v1 + dv / 2.0;
        for _ in 1..=nb_v {
            props.set_parameters(u, v)?;
            let Some(sample) = gauss_sample(&mut props, toler)? else {
                return Ok(Handle::default());
            };
            if !sums.add(&sample, tol) {
                return Ok(Handle::default());
            }
            if least_square {
                points.push(props.value().xyz());
            }
            v += dv;
        }
        u += du;
    }
    let Some(cylinder) = averaged_cylinder(&mut sums, &mut props, (u1, v1), (toler, tol))? else {
        return Ok(Handle::default());
    };
    let cylinder: Cylinder3 = if least_square {
        refined(cylinder, points, nb_u, toler)?
    } else {
        cylinder
    };
    Ok(Handle::new(GeomCylindricalSurface::from_cylinder(cylinder)).cast())
}

// The curvatures of the current point, None when the curvature is undefined, the
// point is umbilic, or the Gaussian curvature is beyond the tolerance.
fn gauss_sample(props: &mut SLProps, toler: f64) -> Result<Option<GaussSample>, Failure> {
    if !props.is_curvature_defined()? || props.is_umbilic()? {
        return Ok(None);
    }
    let min_curv: f64 = props.min_curvature()?;
    let max_curv: f64 = props.max_curvature()?;
    let k1: f64 = props.gaussian_curvature()?.abs().sqrt();
    if k1 > toler {
        return Ok(None);
    }
    let (max_d, min_d): (Direction3, Direction3) = props.curvature_directions()?;
    let (min_abs, max_abs): (f64, f64) = (min_curv.abs(), max_curv.abs());
    if min_abs > max_abs {
        return Ok(Some(GaussSample {
            min_curv: max_abs,
            max_curv: min_abs,
            min_dir: max_d,
            swapped: true,
        }));
    }
    Ok(Some(GaussSample {
        min_curv: min_abs,
        max_curv: max_abs,
        min_dir: min_d,
        swapped: false,
    }))
}

impl GaussSums {
    // Adds a sample; false when its curvatures differ from the running averages.
    fn add(&mut self, sample: &GaussSample, tol: f64) -> bool {
        if sample.swapped {
            self.sign = -1.0;
        }
        self.n += 1;
        let r: f64 = 1.0 / sample.max_curv;
        let r2: f64 = r * r;
        self.radii.push(r);
        if self.n > 1 {
            let previous: f64 = f64::from(self.n - 1);
            if (sample.max_curv - self.av_max_curv / previous).abs() > tol / r2 {
                return false;
            }
            if (sample.min_curv - self.av_min_curv / previous).abs() > tol {
                return false;
            }
        }
        self.av_r += r;
        self.av_dir += sample.min_dir.xyz();
        self.av_max_curv += sample.max_curv;
        self.av_min_curv += sample.min_curv;
        true
    }
}

// The cylinder of the averages, placed from the normal at (U1, V1); None when the
// average min curvature or the radius spread (3 sigma / sqrt(n)) is too large, or
// the curvature at (U1, V1) is undefined.
fn averaged_cylinder(
    sums: &mut GaussSums,
    props: &mut SLProps,
    (u1, v1): (f64, f64),
    (toler, tol): (f64, f64),
) -> Result<Option<Cylinder3>, Failure> {
    let n: f64 = f64::from(sums.n);
    sums.av_max_curv /= n;
    sums.av_min_curv /= n;
    sums.av_r /= n;
    sums.av_dir /= n;
    if sums.av_min_curv.abs() > toler {
        return Ok(None);
    }
    let mut sigma_r: f64 = 0.0;
    for r in &sums.radii {
        let d: f64 = r - sums.av_r;
        sigma_r += d * d;
    }
    sigma_r = (sigma_r / n).sqrt();
    sigma_r = 3.0 * sigma_r / n.sqrt();
    if sigma_r > tol {
        return Ok(None);
    }
    props.set_parameters(u1, v1)?;
    if !props.is_curvature_defined()? {
        return Ok(None);
    }
    let norm: Direction3 = props.normal()?;
    let mut loc: Point3 = props.value();
    let ax_d = Direction3::from_xyz(sums.av_dir)?;
    loc.translate(Vector3::from(sums.sign * sums.av_r * norm.xyz()));
    let mut cylinder = Cylinder3::default();
    cylinder.set_axis(Axis1::new(loc, ax_d))?;
    cylinder.set_radius(sums.av_r)?;
    Ok(Some(cylinder))
}

// The cylinder refined by GetCylByLS on the sampled points when it succeeds.
fn refined(
    mut cylinder: Cylinder3,
    points: Vec<Xyz>,
    nb_u: i32,
    toler: f64,
) -> Result<Cylinder3, Failure> {
    let mut cloud: Array1<'static, Xyz> =
        Array1::with_bounds(1, nb_u * nb_u).map_err(|error| array_failure(error, "aPoints"))?;
    for (slot, point) in cloud.iter_mut().zip(points) {
        *slot = point;
    }
    let cloud: Handle<HArray1<Xyz>> = Handle::new(HArray1::from_owned(cloud));
    let fit = get_cyl_by_ls(&cloud, toler, (cylinder.position(), cylinder.radius()))?;
    if fit.done {
        cylinder.set_position(fit.pos);
        cylinder.set_radius(fit.radius)?;
    }
    Ok(cylinder)
}

// `GetLSGap(thePoints, thePos, theR, theGap)`: the greatest | distance to the axis
// - R | of the points.
fn get_ls_gap(points: &[Xyz], pos: Axis3, radius: f64) -> f64 {
    let loc: Xyz = pos.location().xyz();
    let dir: Vector3 = Vector3::from(pos.direction());
    let mut gap: f64 = 0.0;
    for point in points {
        let mut d = Vector3::from(*point - loc);
        d.cross(dir);
        gap = std_max(gap, (d.magnitude() - radius).abs());
    }
    gap
}

/// The in-out results of GetCylByLS: success, placement, radius and gap.
struct CylFit {
    done: bool,
    pos: Axis3,
    radius: f64,
}

// `GetCylByLS(thePoints, theTol, thePos, theR, theGap)`: the cylinder minimizing
// FuncCylinderLSDist by math_PSO within 20% of the radius, then math_Powell
// across the axis; done when the gap is within Confusion after PSO or `tol` after
// Powell.
fn get_cyl_by_ls(
    points: &Handle<HArray1<Xyz>>,
    tol: f64,
    (mut pos, mut radius): (Axis3, f64),
) -> Result<CylFit, Failure> {
    let cloud: Vec<Xyz> = live(points, "GeomConvert_SurfToAnaSurf::GetCylByLS")?
        .array1()
        .as_slice()
        .to_vec();
    let mut gap: f64 = get_ls_gap(&cloud, pos, radius);
    if gap <= precision::confusion() {
        return Ok(CylFit {
            done: true,
            pos,
            radius,
        });
    }
    let mut func = FuncCylinderLSDist::new(points, pos.direction());
    let mut start: Vector<'static> = pso_start(&mut func, pos, radius)?;
    pos.set_location(Point3::new(start[1], start[2], start[3]));
    radius = start[4];
    gap = get_ls_gap(&cloud, pos, radius);
    if gap <= precision::confusion() {
        return Ok(CylFit {
            done: true,
            pos,
            radius,
        });
    }
    let directions: Matrix<'static> = powell_directions(pos)?;
    let mut solver = Powell::new(&func, precision::confusion())?;
    solver.perform(&mut func, &start, &directions)?;
    func.take_failure()?;
    if solver.is_done() {
        let mut pos2: Axis3 = pos;
        solver.copy_location(&mut start)?;
        pos2.set_location(Point3::new(start[1], start[2], start[3]));
        let (r2, gap2): (f64, f64) = (start[4], get_ls_gap(&cloud, pos2, start[4]));
        if gap2 < gap {
            (gap, pos, radius) = (gap2, pos2, r2);
        }
    }
    Ok(CylFit {
        done: gap <= tol,
        pos,
        radius,
    })
}

// The math_PSO minimum of the location and radius, within 20% of the radius of
// the start (10 steps per variable).
fn pso_start(
    func: &mut FuncCylinderLSDist,
    pos: Axis3,
    radius: f64,
) -> Result<Vector<'static>, Failure> {
    let rel_dev: f64 = 0.2; // relative precision of the sample surface parameters
    let loc: Point3 = pos.location();
    let mut start: Vector<'static> = Vector::with_bounds(1, 4)?;
    (start[1], start[2], start[3], start[4]) = (loc.x(), loc.y(), loc.z(), radius);
    let dr: f64 = rel_dev * radius;
    let mut lower: Vector<'static> = Vector::with_bounds(1, 4)?;
    let mut upper: Vector<'static> = Vector::with_bounds(1, 4)?;
    let mut steps: Vector<'static> = Vector::with_bounds(1, 4)?;
    for i in 1..=4 {
        lower[i] = start[i] - dr;
        upper[i] = start[i] + dr;
        steps[i] = (upper[i] - lower[i]) / 10.0;
    }
    let mut ls_dist: f64 = 0.0;
    {
        let mut global = Pso::new(func, &lower, &upper, &steps)?;
        global.perform(&steps, &mut ls_dist, &mut start)?;
    }
    func.take_failure()?;
    Ok(start)
}

// The Powell directions: the X, Y and X+Y directions of the plane normal to the
// axis for the location, and the radius direction.
fn powell_directions(pos: Axis3) -> Result<Matrix<'static>, Failure> {
    let mut directions: Matrix<'static> = matrix(1, 4, 1, 4, 0.0)?;
    for i in 1..=4 {
        directions[(i, i)] = 1.0;
    }
    // Search the location normal to the axis, to avoid searching along it.
    let plane = Plane3::new(pos.location(), pos.direction())?;
    let u_dir: Direction3 = plane.position().x_direction();
    let v_dir: Direction3 = plane.position().y_direction();
    for i in 1..=3 {
        directions[(i, 1)] = u_dir.coord(i)?;
        directions[(i, 2)] = v_dir.coord(i)?;
        let uv_dir = Direction3::from_xyz(u_dir.xyz() + v_dir.xyz())?;
        directions[(i, 3)] = uv_dir.coord(i)?;
    }
    Ok(directions)
}
