// Rust translation of the triangle integration of
// ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_MeshProps.cxx (CalculateProps,
// CalculateElSProps, CalculateElVProps) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The surface properties of a triangle, or the volume properties of the pyramid on a
//! triangle, by Gauss integration over its area.
use super::mesh_props::BRepGPropMeshProps;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{self, Axis3, Direction3, Point3, Vector3, Xyz};

/// The ten sums of `CalculateProps` (`GProps[10]`): the mass, the static moments
/// `Ix, Iy, Iz`, then `Ixx, Iyy, Izz, Ixy, Ixz, Iyz`.
pub type GPropsSums = [f64; 10];

impl BRepGPropMeshProps {
    /// `CalculateProps(p1, p2, p3, Apex, isVolume, GProps, NbGaussPoints, GaussPnts)`: adds
    /// to `sums` the surface properties of the triangle (or, with `is_volume`, the volume
    /// properties of the pyramid on it with apex `apex`), relative to `apex`, with
    /// `nb_gauss_points` barycentric points `(l1, l2, w)` of `gauss_points`; nothing for
    /// a triangle whose normal is below gp::Resolution(); Standard_OutOfRange when
    /// `gauss_points` holds fewer than `3 * nb_gauss_points` values (a native overrun);
    /// `BRepGPropMeshProps::calculate_props((p1, p2, p3), apex, false, &mut sums, 1, &[1. / 3., 1. / 3., 0.5])?`.
    pub fn calculate_props(
        triangle: (Point3, Point3, Point3),
        apex: Point3,
        is_volume: bool,
        sums: &mut GPropsSums,
        nb_gauss_points: i32,
        gauss_points: &[f64],
    ) -> Result<(), Failure> {
        let (p1, p2, p3) = triangle;
        let normal = Vector3::between(p2, p1).crossed(Vector3::between(p3, p2));
        let det = normal.magnitude();
        if det <= gp::resolution() {
            return Ok(());
        }
        let centre: Xyz = (p1.xyz() + p2.xyz() + p3.xyz()) / 3.;
        let direction = Direction3::from_vector(normal)?;
        let plane = Axis3::from_normal(Point3::from(centre), direction)?;
        let uv = [p1, p2, p3].map(|p| elslib::plane_parameters(plane, p));
        for i in 0..nb_gauss_points.max(0) as usize {
            let rule = gauss_points
                .get(3 * i..3 * i + 3)
                .ok_or_else(|| overrun(i))?;
            let (l1, l2, w) = (rule[0], rule[1], rule[2] * det);
            let x = l1 * (uv[0].0 - uv[2].0) + l2 * (uv[1].0 - uv[2].0) + uv[2].0;
            let y = l1 * (uv[0].1 - uv[2].1) + l2 * (uv[1].1 - uv[2].1) + uv[2].1;
            let relative: Xyz = elslib::plane_value(x, y, plane).xyz() - apex.xyz();
            if is_volume {
                let normal_w = direction.xyz() * w;
                let dv = relative.x() * normal_w.x()
                    + relative.y() * normal_w.y()
                    + relative.z() * normal_w.z();
                add_volume_point(relative, dv, sums);
            } else {
                add_surface_point(relative, w, sums);
            }
        }
        Ok(())
    }
}

/// `CalculateElSProps`: one Gauss point of the surface properties.
fn add_surface_point(p: Xyz, ds: f64, sums: &mut GPropsSums) {
    let (x, y, z) = (p.x(), p.y(), p.z());
    let (x2, y2, z2) = (x * x, y * y, z * z);
    sums[0] += ds;
    sums[1] += x * ds;
    sums[2] += y * ds;
    sums[3] += z * ds;
    sums[7] += x * y * ds;
    sums[8] += x * z * ds;
    sums[9] += y * z * ds;
    sums[4] += (y2 + z2) * ds;
    sums[5] += (x2 + z2) * ds;
    sums[6] += (x2 + y2) * ds;
}

/// `CalculateElVProps`: one Gauss point of the volume properties of the pyramid.
fn add_volume_point(p: Xyz, dv: f64, sums: &mut GPropsSums) {
    let (x, y, z) = (p.x(), p.y(), p.z());
    let (x2, y2, z2) = (x * x, y * y, z * z);
    sums[0] += dv / 3.0;
    sums[1] += 0.25 * x * dv;
    sums[2] += 0.25 * y * dv;
    sums[3] += 0.25 * z * dv;
    let dv1 = 0.2 * dv;
    sums[7] += x * y * dv1;
    sums[8] += x * z * dv1;
    sums[9] += y * z * dv1;
    sums[4] += (y2 + z2) * dv1;
    sums[5] += (x2 + z2) * dv1;
    sums[6] += (x2 + y2) * dv1;
}

fn overrun(point: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!(
            "BRepGProp_MeshProps::CalculateProps: Gauss point {point} is past the end of \
             GaussPnts; expected 3 values (l1, l2, w) per point"
        ),
        "",
    )
}
