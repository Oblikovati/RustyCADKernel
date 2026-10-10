//! Ray intersection kernels of SelectMgr_BaseIntersector at OCCT 3d097a0.
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{std_max, std_min};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, StandardDirection3, Transform3};
/// The entry and exit parameters of a ray crossing a sphere.
pub type SelectMgrRayTimes = (f64, f64);

/// `RaySphereIntersection`: the entry and exit times of a ray, the one of smaller magnitude
/// first. Example: `select_mgr_ray_sphere_intersection(&c, 1.0, &loc, &dir)`.
pub fn select_mgr_ray_sphere_intersection(
    center: &Point3,
    radius: f64,
    loc: &Point3,
    ray_dir: &Direction3,
) -> Option<SelectMgrRayTimes> {
    let a = ray_dir.dot(*ray_dir);
    let k = ray_dir.x() * (loc.x() - center.x())
        + ray_dir.y() * (loc.y() - center.y())
        + ray_dir.z() * (loc.z() - center.z());
    let c = loc.distance(center) * loc.distance(center) - radius * radius;
    let discr = k * k - a * c;
    if discr < 0.0 {
        return None;
    }
    let (t1, t2) = ((-k - discr.sqrt()) / a, (-k + discr.sqrt()) / a);
    Some(if t1.abs() < t2.abs() {
        (t1, t2)
    } else {
        (t2, t1)
    })
}

/// The ray times through the end disks of a solid cylinder (bottom at z = 0, top at the height).
fn end_disk_times(
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    loc: &Point3,
    dir: &Direction3,
    times: &mut Vec<f64>,
) {
    let at = |z: f64| {
        let t = (z - loc.z()) / dir.z();
        (t, loc.x() + dir.x() * t, loc.y() + dir.y() * t)
    };
    let (t1, x1, y1) = at(0.0);
    if x1 * x1 + y1 * y1 <= bottom_radius * bottom_radius {
        times.push(t1);
    }
    let (t2, x2, y2) = at(height);
    if x2 * x2 + y2 * y2 <= top_radius * top_radius {
        times.push(t2);
    }
}

fn cone_frame(
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    triangle_height: f64,
) -> Result<Transform3, Failure> {
    let mut system = Axis3::default();
    if bottom_radius > top_radius {
        system.set_location(Point3::new(0.0, 0.0, height + triangle_height));
        system.set_direction(-Direction3::from_standard(StandardDirection3::Z))?;
    } else {
        system.set_location(Point3::new(0.0, 0.0, -triangle_height));
        system.set_direction(Direction3::from_standard(StandardDirection3::Z))?;
    }
    let mut trsf_cone = Transform3::default();
    trsf_cone.set_transformation_between(Axis3::default(), system);
    Ok(trsf_cone)
}

fn cone_crossings(
    p: Point3,
    d: Direction3,
    max_rad: f64,
    cone_height: f64,
    triangle_height: f64,
    times: &mut Vec<f64>,
) {
    let (r2, h2) = (max_rad * max_rad, cone_height * cone_height);
    let a = d.x() * d.x() / r2 + d.y() * d.y() / r2 - d.z() * d.z() / h2;
    let k = d.x() * p.x() / r2 + d.y() * p.y() / r2 - d.z() * p.z() / h2;
    let c = p.x() * p.x() / r2 + p.y() * p.y() / r2 - p.z() * p.z() / h2;
    let discr = k * k - a * c;
    if discr.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return;
    }
    let (enter, leave) = ((-k - discr.sqrt()) / a, (-k + discr.sqrt()) / a);
    for t in [enter, leave] {
        let z = p.z() + t * d.z();
        if z > triangle_height && z < cone_height {
            times.push(t);
        }
    }
}

/// The ray times through the side of a cone frustum (the radii differ).
fn cone_times(
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    loc: &Point3,
    ray_dir: &Direction3,
    times: &mut Vec<f64>,
) -> Result<(), Failure> {
    let triangle_height =
        std_min(bottom_radius, top_radius) * height / (bottom_radius - top_radius).abs();
    let trsf_cone = cone_frame(bottom_radius, top_radius, height, triangle_height)?;
    let (p, d) = (loc.transformed(trsf_cone), ray_dir.transformed(trsf_cone));
    let max_rad = std_max(bottom_radius, top_radius);
    let cone_height = height + triangle_height;
    cone_crossings(p, d, max_rad, cone_height, triangle_height, times);
    Ok(())
}

/// The ray times through the side of a cylinder (equal radii).
fn cylinder_side_times(
    radius: f64,
    height: f64,
    loc: &Point3,
    ray_dir: &Direction3,
    times: &mut Vec<f64>,
) {
    let (lx, ly, dx, dy) = (loc.x(), loc.y(), ray_dir.x(), ray_dir.y());
    let a = dx * dx + dy * dy;
    let k = lx * dx + ly * dy;
    let c = lx * lx + ly * ly - radius * radius;
    let discr = k * k - a * c;
    if discr.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return;
    }
    let (root1, root2) = ((-k + discr.sqrt()) / a, (-k - discr.sqrt()) / a);
    for root in [root1, root2] {
        let z = loc.z() + root * ray_dir.z();
        if z > 0.0 && z < height {
            times.push(root);
        }
    }
}

/// `RayCylinderIntersection`: the two nearest ray times through a cylinder or cone frustum
/// (the end disks of a solid one included), the leave time None for a single crossing (the
/// native output argument is left untouched then).
/// Example: `select_mgr_ray_cylinder_intersection(1.0, 1.0, 2.0, &loc, &dir, false)?`.
pub fn select_mgr_ray_cylinder_intersection(
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    loc: &Point3,
    ray_dir: &Direction3,
    is_hollow: bool,
) -> Result<Option<(f64, Option<f64>)>, Failure> {
    let mut times = Vec::with_capacity(4);
    if !is_hollow && ray_dir.z() != 0.0 {
        end_disk_times(bottom_radius, top_radius, height, loc, ray_dir, &mut times);
    }
    if top_radius != bottom_radius {
        cone_times(bottom_radius, top_radius, height, loc, ray_dir, &mut times)?;
    } else {
        cylinder_side_times(top_radius, height, loc, ray_dir, &mut times);
    }
    if times.is_empty() {
        return Ok(None);
    }
    // std::sort with operator< on at most four values: a stable comparison sort.
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Ok(Some((times[0], times.get(1).copied())))
}

/// `RayCircleIntersection`: the ray time on the circle plane, inside a filled circle or on
/// the circle within `Precision::Confusion`. Example: `select_mgr_ray_circle_intersection(1.0, &loc, &dir, true)`.
pub fn select_mgr_ray_circle_intersection(
    radius: f64,
    loc: &Point3,
    ray_dir: &Direction3,
    is_filled: bool,
) -> Option<f64> {
    if ray_dir.z() == 0.0 {
        return None;
    }
    let time = (0.0 - loc.z()) / ray_dir.z();
    let (x, y) = (loc.x() + ray_dir.x() * time, loc.y() + ray_dir.y() * time);
    let k = x * x + y * y;
    let on = if is_filled {
        k <= radius * radius
    } else {
        (k.sqrt() - radius).abs() <= confusion()
    };
    on.then_some(time)
}
