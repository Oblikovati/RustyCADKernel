// Rust translation of the helpers of GeomBndLib_SurfaceOfExtrusion.cxx (buildExtrusionBox)
// and GeomBndLib_SurfaceOfRevolution.cxx (addRevolutionCircle, buildRevolutionBox) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Sweeping a basis curve box: along a direction (extrusion) or around an axis
//! (revolution).
use super::circle::Circle;
use super::curve::Curve;
use super::infinite::InfiniteDirection;
use super::other_surface::OtherSurface;
use super::spline_surface::surface_adaptor_of;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bnd::{BoundingBox3, BoxLimits3};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3, Xyz};

/// The basis parameter range of a patch: on a non-periodic curve each end is clamped to
/// `[First, Last]` and the pair ordered; `clamped_basis_range(curve, (0.0, 2.0))`.
pub fn clamped_basis_range(curve: &dyn GeomCurve, (mut a, mut b): (f64, f64)) -> (f64, f64) {
    if curve.is_periodic() {
        return (a, b);
    }
    let (first, last) = (curve.first_parameter(), curve.last_parameter());
    let clamp = |u: f64| {
        if u < first {
            first
        } else if u > last {
            last
        } else {
            u
        }
    };
    a = clamp(a);
    b = clamp(b);
    if a > b {
        std::mem::swap(&mut a, &mut b);
    }
    (a, b)
}

/// A `Bnd_Box::OpenXmin`-like member.
type BoxOpener = fn(&mut BoundingBox3);

fn copy_open_flags(from: &BoundingBox3, to: &mut BoundingBox3) {
    let flags: [(bool, BoxOpener); 6] = [
        (from.is_open_xmin(), BoundingBox3::open_xmin),
        (from.is_open_xmax(), BoundingBox3::open_xmax),
        (from.is_open_ymin(), BoundingBox3::open_ymin),
        (from.is_open_ymax(), BoundingBox3::open_ymax),
        (from.is_open_zmin(), BoundingBox3::open_zmin),
        (from.is_open_zmax(), BoundingBox3::open_zmax),
    ];
    for (open, opener) in flags {
        if open {
            opener(to);
        }
    }
}

// The curve box corners shifted by `v * D`.
fn add_shifted_corners(bounds: &mut BoundingBox3, l: &BoxLimits3, direction: Xyz, v: f64) {
    let shift: Xyz = direction * v;
    bounds.add_point(Point3::new(
        l.x_min + shift.x(),
        l.y_min + shift.y(),
        l.z_min + shift.z(),
    ));
    bounds.add_point(Point3::new(
        l.x_max + shift.x(),
        l.y_max + shift.y(),
        l.z_max + shift.z(),
    ));
}

// The infinite V ends open the direction: both, the min side or the max side.
fn open_infinite_v(
    bounds: &mut BoundingBox3,
    direction: Direction3,
    v_min_inf: bool,
    v_max_inf: bool,
) {
    if v_min_inf && v_max_inf {
        direction.open_min_max(bounds);
    } else if v_min_inf {
        direction.open_min(bounds);
    } else if v_max_inf {
        direction.open_max(bounds);
    }
}

/// `buildExtrusionBox`: the curve box corners at `v_min` and `v_max` along the
/// direction; an open curve box keeps its open sides and sweeps its finite part; an
/// infinite V opens the direction (`VMin` only negative-infinite, `VMax` only positive,
/// as natively); `extrusion_box(&curve_box, direction, (0.0, 1.0), 0.0)`.
pub fn extrusion_box(
    curve_box: &BoundingBox3,
    direction: Direction3,
    (v_min, v_max): (f64, f64),
    tolerance: f64,
) -> BoundingBox3 {
    let mut bounds: BoundingBox3 = BoundingBox3::default();
    if curve_box.is_void() {
        bounds.enlarge(tolerance);
        return bounds;
    }
    let v_min_inf: bool = precision::is_negative_infinite(v_min);
    let v_max_inf: bool = precision::is_positive_infinite(v_max);
    let d: Xyz = direction.xyz();
    if curve_box.is_open() {
        copy_open_flags(curve_box, &mut bounds);
        open_infinite_v(&mut bounds, direction, v_min_inf, v_max_inf);
        if curve_box.has_finite_part() {
            let limits: BoxLimits3 = curve_box.finite_part().limits();
            [(v_min_inf, v_min), (v_max_inf, v_max)]
                .into_iter()
                .filter(|(infinite, _)| !infinite)
                .for_each(|(_, v)| add_shifted_corners(&mut bounds, &limits, d, v));
        }
    } else if v_min_inf && v_max_inf {
        bounds.add_box(curve_box);
        direction.open_min_max(&mut bounds);
    } else {
        let limits: BoxLimits3 = curve_box.limits();
        open_infinite_v(&mut bounds, direction, v_min_inf, v_max_inf);
        [(v_min_inf, v_min), (v_max_inf, v_max)]
            .into_iter()
            .filter(|(infinite, _)| !infinite)
            .for_each(|(_, v)| add_shifted_corners(&mut bounds, &limits, d, v));
    }
    bounds.enlarge(tolerance);
    bounds
}

/// `addRevolutionCircle`: the arc `[u_min, u_max]` of the circle `point` describes
/// around the axis, or the point itself when it lies on the axis (radius below
/// Precision::Confusion()); `add_revolution_circle(&mut bounds, origin, axis, p, (0.0, 1.0))?`.
pub fn add_revolution_circle(
    bounds: &mut BoundingBox3,
    (origin, axis): (Point3, Direction3),
    point: Point3,
    (u_min, u_max): (f64, f64),
) -> Result<(), Failure> {
    let delta: Xyz = point.xyz() - origin.xyz();
    let height: f64 = delta.dot(axis.xyz());
    let centre: Xyz = origin.xyz() + axis.xyz() * height;
    let radial: Xyz = point.xyz() - centre;
    let radius: f64 = radial.modulus();
    if radius < precision::confusion() {
        bounds.add_point(point);
        return Ok(());
    }
    let frame: Axis2 =
        Axis2::with_x_direction(Point3::from(centre), axis, Direction3::from_xyz(radial)?)?;
    let circle: Circle3 = Circle3::new(frame, radius)?;
    bounds.add_box(&Circle::circ_arc_box(&circle, u_min, u_max, 0.0));
    Ok(())
}

// The eight corners of `l`, x outermost and z innermost as the native loops.
fn revolve_corners(
    bounds: &mut BoundingBox3,
    axis: (Point3, Direction3),
    l: &BoxLimits3,
    range: (f64, f64),
) -> Result<(), Failure> {
    for x in [l.x_min, l.x_max] {
        for y in [l.y_min, l.y_max] {
            for z in [l.z_min, l.z_max] {
                add_revolution_circle(bounds, axis, Point3::new(x, y, z), range)?;
            }
        }
    }
    Ok(())
}

/// Which boxes a swept surface uses: `Box` (the basis curve's `Box`, OtherSurface's
/// `Box`) or `BoxOptimal` (both optimal).
#[derive(Clone, Copy)]
pub enum BasisBox {
    /// The `Box` members.
    Box,
    /// The `BoxOptimal` members.
    Optimal,
}

/// The basis curve box over `[c1, c2]` with zero tolerance, through GeomBndLib_Curve;
/// `basis_curve_box(&basis, (0.0, 1.0), BasisBox::Box)?`.
pub fn basis_curve_box(
    basis: &Handle<dyn GeomCurve>,
    (c1, c2): (f64, f64),
    kind: BasisBox,
) -> Result<BoundingBox3, Failure> {
    let evaluator = Curve::from_curve(basis)?;
    match kind {
        BasisBox::Box => evaluator.bounding_box_range(c1, c2, 0.0),
        BasisBox::Optimal => evaluator.bounding_box_optimal_range(c1, c2, 0.0),
    }
}

/// The fallback of the swept surfaces (their `catch (Standard_Failure)` and void curve
/// box): GeomBndLib_OtherSurface on a GeomAdaptor_Surface of `surface`;
/// `other_surface_box(&handle, [0.0, 1.0, 0.0, 1.0], 0.0, BasisBox::Box)?`.
pub fn other_surface_box<T: GeomSurface + RuntimeTyped>(
    surface: &Handle<T>,
    [u_min, u_max, v_min, v_max]: [f64; 4],
    tolerance: f64,
    kind: BasisBox,
) -> Result<BoundingBox3, Failure> {
    let adaptor: GeomAdaptorSurface = surface_adaptor_of(surface)?;
    let other = OtherSurface::new(&adaptor as &dyn Adaptor3dSurface);
    match kind {
        BasisBox::Box => other.bounding_box_range(u_min, u_max, v_min, v_max, tolerance),
        BasisBox::Optimal => {
            other.bounding_box_optimal_range(u_min, u_max, v_min, v_max, tolerance)
        }
    }
}

/// `buildRevolutionBox`: the revolution circles of the eight curve box corners; an open
/// curve box revolves its finite part and keeps its open sides;
/// `revolution_box(&curve_box, (origin, axis), (0.0, 6.3), 0.0)?`.
pub fn revolution_box(
    curve_box: &BoundingBox3,
    axis: (Point3, Direction3),
    range: (f64, f64),
    tolerance: f64,
) -> Result<BoundingBox3, Failure> {
    let mut bounds: BoundingBox3 = BoundingBox3::default();
    if curve_box.is_void() {
        bounds.enlarge(tolerance);
        return Ok(bounds);
    }
    if curve_box.is_open() {
        if curve_box.has_finite_part() {
            revolve_corners(&mut bounds, axis, &curve_box.finite_part().limits(), range)?;
        }
        copy_open_flags(curve_box, &mut bounds);
    } else {
        revolve_corners(&mut bounds, axis, &curve_box.limits(), range)?;
    }
    bounds.enlarge(tolerance);
    Ok(bounds)
}
