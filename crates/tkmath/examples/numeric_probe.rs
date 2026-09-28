//! Exact gp_XYZ and Bnd_Range numeric oracle counterpart.
use rustycad_tkernel::{ncollection::List, standard::dump::DumpStream};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2, Vector2, Xy};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{Axis1, Axis2, Axis3, Direction3, GeneralTransform3, Matrix3, Point3, Transform3, Xyz},
};
use std::io::{self, BufRead};
fn xyz(value: Xyz) {
    print!(
        "{} {} {} ",
        value.x().to_bits(),
        value.y().to_bits(),
        value.z().to_bits()
    );
}
fn range(value: Range1) {
    let mut first: f64 = 91.0;
    let mut last: f64 = 92.0;
    let ok = value.get_bounds(&mut first, &mut last);
    print!(
        "{} {} {} {} ",
        i32::from(ok),
        first.to_bits(),
        last.to_bits(),
        value.delta().to_bits()
    );
}
fn vector_case(values: &[f64]) {
    let left = Xyz::new(values[0], values[1], values[2]);
    let right = Xyz::new(values[3], values[4], values[5]);
    let scalar = values[6];
    xyz(left + right);
    xyz(left - right);
    xyz(left * scalar);
    xyz(left / scalar);
    xyz(left.crossed(right));
    xyz(left.cross_crossed(right, left));
    print!(
        "{} {} {} {} ",
        left.dot(right).to_bits(),
        left.cross_magnitude(right).to_bits(),
        left.dot_cross(right, left).to_bits(),
        i32::from(left.is_equal(&right, scalar))
    );
    match left.normalized() {
        Ok(normalized) => xyz(normalized),
        Err(failure) => print!("{} ", failure.exception_type()),
    }
    println!();
}
fn interval(first: f64, last: f64) -> Range1 {
    if first > last {
        Range1::default()
    } else {
        Range1::new(first, last).unwrap()
    }
}
fn range_case(values: &[f64]) {
    let original = interval(values[0], values[1]);
    let other = interval(values[2], values[3]);
    let value = values[4];
    let period = values[5];
    range(original);
    print!(
        "{} {} {} ",
        original.is_intersected(value, period) as i32,
        i32::from(original.is_out(value)),
        i32::from(original.is_out_range(other))
    );
    let mut copy = original;
    copy.common(other);
    range(copy);
    copy = original;
    print!("{} ", i32::from(copy.union(other)));
    range(copy);
    copy = original;
    copy.add(value);
    range(copy);
    copy = original;
    copy.add_range(other);
    range(copy);
    copy = original;
    copy.enlarge(value);
    range(copy);
    copy = original;
    copy.trim_from(value);
    range(copy);
    copy = original;
    copy.trim_to(value);
    range(copy);
    range(original.shifted(value).unwrap());
    let mut middle: f64 = 91.0;
    print!(
        "{} {} ",
        i32::from(original.get_intermediate_point(value, &mut middle)),
        middle.to_bits()
    );
    let center = original.center();
    print!(
        "{} {} ",
        i32::from(center.is_some()),
        center.unwrap_or(91.0).to_bits()
    );
    let mut parts = List::default();
    original.split(value, &mut parts, period).unwrap();
    print!("{} ", parts.len());
    for part in parts.iter() {
        range(*part);
    }
    let mut dump = DumpStream::default();
    original.dump_json(&mut dump).unwrap();
    println!("{}", dump.text());
}
fn matrix(value: Matrix3) {
    for row in value.coefficients() {
        for coefficient in row {
            print!("{} ", coefficient.to_bits());
        }
    }
}
fn matrix_result(value: Result<Matrix3, rustycad_tkernel::standard::Failure>) {
    match value {
        Ok(value) => matrix(value),
        Err(failure) => print!("{} ", failure.exception_type()),
    }
}
fn matrix_case(values: &[f64]) {
    let mut m = Matrix3::new(std::array::from_fn(|row| {
        std::array::from_fn(|col| values[row * 3 + col])
    }));
    print!(
        "{} {} ",
        m.determinant().to_bits(),
        i32::from(m.is_singular())
    );
    matrix(m.transposed());
    xyz(m * Xyz::new(1.25, -2.5, 3.75));
    matrix_result(m.inverted());
    matrix_result(m.powered(values[10] as i32));
    matrix_result(m.divided(values[9]));
    let rotation = m
        .set_rotation(Xyz::new(values[0], values[1], values[2]), values[9])
        .map(|()| m);
    matrix_result(rotation);
    println!();
}
fn point_case(v: &[f64]) {
    let point = Point3::new(v[0], v[1], v[2]);
    let origin = Point3::new(v[3], v[4], v[5]);
    let normal = Direction3::new(v[6], v[7], v[8]).unwrap();
    let axis = Axis1::new(origin, normal);
    let plane = Axis2::new(origin, normal).unwrap();
    xyz(point.rotated(axis, v[9]).unwrap().xyz());
    xyz(point.mirrored_point(origin).xyz());
    xyz(point.mirrored_axis(axis).xyz());
    xyz(point.mirrored_plane(plane).xyz());
    xyz(point.scaled(origin, v[10]).xyz());
    let first = Transform3::rotation(axis, v[9]).unwrap();
    let second = Transform3::rotation(Axis1::new(origin, Direction3::Z), v[11]).unwrap();
    xyz(point.transformed(second.multiplied(first)).xyz());
    xyz(point.transformed(first.inverted().unwrap()).xyz());
    xyz(point
        .transformed(first.powered(v[12] as i32).unwrap())
        .xyz());
    println!();
}
fn direction_case(v: &[f64]) {
    let mut direction = Direction3::new(v[0], v[1], v[2]).unwrap();
    let normal = Direction3::new(v[3], v[4], v[5]).unwrap();
    let origin = Point3::new(v[6], v[7], v[8]);
    let axis = Axis1::new(origin, normal);
    let plane = Axis2::new(origin, normal).unwrap();
    let transform = Transform3::rotation(axis, v[9]).unwrap();
    xyz(Xyz::from(direction));
    print!(
        "{} {} ",
        direction.angle(&normal).to_bits(),
        direction.angle_with_ref(normal, Direction3::Z).to_bits()
    );
    xyz(Xyz::from(direction.mirrored_axis(axis)));
    xyz(Xyz::from(direction.mirrored_plane(plane)));
    xyz(Xyz::from(direction.mirrored_direction(normal)));
    xyz(Xyz::from(direction.rotated(axis, v[9]).unwrap()));
    xyz(Xyz::from(direction.transformed(transform)));
    print!(
        "{} {} {} {} ",
        direction.dot(normal).to_bits(),
        i32::from(direction.is_parallel(normal, 1e-12)),
        i32::from(direction.is_normal(normal, 1e-12)),
        i32::from(direction.is_opposite(normal, 1e-12))
    );
    match direction.crossed(normal) {
        Ok(value) => xyz(Xyz::from(value)),
        Err(error) => print!("{} ", error.exception_type()),
    }
    if let Err(error) = direction.cross(direction) {
        print!("{} ", error.exception_type());
    }
    xyz(Xyz::from(direction));
    println!();
}
fn frame_values(frame: Axis3) {
    xyz(Xyz::from(frame.direction()));
    xyz(Xyz::from(frame.x_direction()));
    xyz(Xyz::from(frame.y_direction()));
    print!("{} ", i32::from(frame.direct().unwrap()));
}
fn frame_case(v: &[f64]) {
    let normal = Direction3::new(v[0], v[1], v[2]).unwrap();
    let first = Direction3::new(v[3], v[4], v[5]).unwrap();
    let second = Direction3::new(v[6], v[7], v[8]).unwrap();
    let mut frame = Axis3::default();
    if v[9] < 0.0 {
        frame.z_reverse();
    }
    frame.set_direction(normal).unwrap();
    frame_values(frame);
    frame.set_x_direction(first).unwrap();
    frame_values(frame);
    frame.set_y_direction(second).unwrap();
    frame_values(frame);
    let mut affine = GeneralTransform3::default();
    let origin = Point3::new(v[3], v[4], v[5]);
    let point = Xyz::new(v[6], v[7], v[8]);
    affine.set_affinity_axis(Axis1::new(origin, normal), v[9]);
    xyz(affine.apply(point));
    affine.set_affinity_plane(Axis2::new(origin, normal).unwrap(), v[9]);
    xyz(affine.apply(point));
    println!();
}
fn xy(value: Xy) {
    print!("{} {} ", value.x().to_bits(), value.y().to_bits());
}
fn planar_case(v: &[f64]) {
    let point = Point2::new(v[0], v[1]);
    let origin = Point2::new(v[2], v[3]);
    let direction = Direction2::new(v[0], v[1]).unwrap();
    let normal = Direction2::new(v[4], v[5]).unwrap();
    let axis = Axis2d::new(origin, normal);
    let rotation = Transform2::rotation(origin, v[6]);
    let mirror = Transform2::mirror_axis(axis);
    xy(point.rotated(origin, v[6]).xy());
    xy(point.mirrored_axis(axis).xy());
    xy(point.scaled(origin, v[7]).xy());
    xy(point.transformed(mirror.multiplied(mirror)).xy());
    xy(point.transformed(rotation.multiplied(mirror)).xy());
    xy(direction.rotated(v[6]).xy());
    xy(direction.mirrored_axis(axis).xy());
    xy(direction.transformed(mirror).xy());
    print!("{} ", direction.angle(normal).to_bits());
    let left = Vector2::new(v[0], v[1]);
    let right = Vector2::new(v[4], v[5]);
    print!(
        "{} {} ",
        left.angle(right).unwrap().to_bits(),
        i32::from(left.is_normal(right, v[8]).unwrap())
    );
    let second = Transform2::rotation(Point2::default(), v[7]);
    xy(point.transformed(second.multiplied(rotation)).xy());
    println!();
}
mod bounds_probe;
mod quaternion_probe;
mod scale_probe;
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut words = line.split_whitespace();
        let op = words.next().unwrap();
        let values: Vec<f64> = words.map(|w| w.parse().unwrap()).collect();
        if op == "cslib_normal" {
            cslib_probe::cslib_normal_case(&values);
        } else if op == "cslib_singular" {
            cslib_probe::cslib_singular_case(&values);
        } else if op == "cslib_dn" {
            cslib_probe::cslib_dn_case(&values);
        } else if op == "cslib_poly" {
            cslib_probe::cslib_poly_case(&values);
        } else if op == "cslib_class2d" {
            cslib_probe::cslib_class2d_case(&values);
        } else if op == "modern_lbfgs" {
            modern_opt_probe::modern_lbfgs_case(&values);
        } else if op == "modern_newton" {
            modern_opt_probe::modern_newton_case(&values);
        } else if op == "modern_global" {
            global_modern_probe::modern_global_case(&values);
        } else if op == "modern_swarm" {
            swarm_probe::modern_swarm_case(&values);
        } else if op == "modern_small" {
            small_probe::modern_small_case(&values);
        } else if op == "modern_lm" {
            modern_sys_probe::modern_lm_case(&values);
        } else if op == "modern_sys" {
            modern_sys_probe::modern_sys_case(&values);
        } else if op == "modern_tensor" {
            modern_tensor_probe::modern_tensor_case(&values);
        } else if op == "modern_double_exp" {
            modern_integ_probe::modern_double_exp_case(&values);
        } else if op == "modern_kronrod" {
            modern_integ_probe::modern_kronrod_case(&values);
        } else if op == "modern_integ" {
            modern_integ_probe::modern_integ_case(&values);
        } else if op == "modern_multiple" {
            modern_multiple_probe::modern_multiple_case(&values);
        } else if op == "modern_trig" {
            modern_trig_probe::modern_trig_case(&values);
        } else if op == "modern_poly" {
            modern_poly_probe::modern_poly_case(&values);
        } else if op == "modern_laguerre" {
            modern_poly_probe::modern_laguerre_case(&values);
        } else if op == "modern_complex" {
            modern_poly_probe::modern_complex_case(&values);
        } else if op == "modern_factors" {
            modern_factors_probe::modern_factors_case(&values);
        } else if op == "modern_lu" {
            modern_eigen_probe::modern_lu_case(&values);
        } else if op == "modern_eigen" {
            modern_eigen_probe::modern_eigen_case(&values);
        } else if op == "modern_frpr" {
            modern_opt_probe::frpr_case(&values);
        } else if op == "scalar_opt" {
            modern_root_probe::scalar_opt_case(&values);
        } else if op == "modern_opt" {
            modern_opt_probe::modern_opt_case(&values);
        } else if op == "vector_functor" {
            vector_functor_probe::vector_functor_case(&values);
        } else if op == "scalar_functor" {
            scalar_functor_probe::scalar_functor_case(&values);
        } else if op == "modern_root" {
            modern_root_probe::modern_root_case(&values);
        } else if op == "utils_line" {
            utils_probe::line_case(&values);
        } else if op == "utils_core" {
            utils_probe::core_case(&values);
        } else if op == "utils_bracket" {
            utils_probe::bracket_case(&values);
        } else if op == "global" {
            global_probe::global_case(&values);
        } else if op == "pso" {
            pso_probe::pso_case(&values);
        } else if op == "bullard" {
            pso_probe::bullard_case(&values);
        } else if op == "integration" {
            integration_probe::integration_case(&values);
        } else if op == "quadrature" {
            integration_probe::quadrature_case(&values);
        } else if op == "all_roots" {
            all_roots_probe::all_roots_case(&values);
        } else if op == "sampled" {
            sampled_probe::sampled_case(&values);
        } else if op == "trig" {
            trig_probe::trig_case(&values);
        } else if op == "polynomial" {
            polynomial_probe::polynomial_case(&values);
        } else if op == "uzawa" {
            uzawa_probe::uzawa_case(&values);
        } else if op == "optimization" {
            optimization_probe::optimization_case(&values);
        } else if op == "function_set" {
            newton_set_probe::function_set_case(&values);
        } else if op == "newton_set" {
            newton_set_probe::newton_set_case(&values);
        } else if op == "minimum" {
            minimum_probe::minimum_case(&values);
        } else if op == "root" {
            root_probe::root_case(&values);
        } else if op == "kronrod" {
            kronrod_probe::kronrod_case(&values);
        } else if op == "eigen" {
            eigen_probe::eigen_case(&values);
        } else if op == "svd" {
            svd_probe::svd_case(&values);
        } else if op == "least_squares" {
            least_squares_probe::least_squares_case(&values);
        } else if op == "numeric_matrix" {
            matrix_probe::numeric_matrix_case(&values);
        } else if op == "numeric_vector" {
            vector_probe::numeric_vector_case(&values);
        } else if op == "nodes" {
            nodes_probe::nodes_case(&values);
        } else if op == "table" {
            table_probe::table_case(&values);
        } else if op == "quaternion" {
            quaternion_probe::quaternion_case(&values);
        } else if op == "scale" {
            scale_probe::scale_case(&values);
        } else if op == "compact3" {
            bounds_probe::compact3_case(&values);
        } else if op == "compact2" {
            bounds_probe::compact2_case(&values);
        } else if op == "sphere" {
            bounds_probe::sphere_case(&values);
        } else if op == "bounds2" {
            bounds_probe::bounds2_case(&values);
        } else if op == "bounds" {
            bounds_probe::bounds_case(&values);
        } else if op == "line_plane" {
            bounds_probe::line_plane_case(&values);
        } else if op == "planar" {
            planar_case(&values);
        } else if op == "frame" {
            frame_case(&values);
        } else if op == "direction" {
            direction_case(&values);
        } else if op == "point" {
            point_case(&values);
        } else if op == "xyz" {
            vector_case(&values);
        } else if op == "matrix" {
            matrix_case(&values);
        } else if op == "range" {
            range_case(&values);
        } else {
            panic!("Unknown probe operation: {op}");
        }
    }
}

mod table_probe;

mod nodes_probe;

mod vector_probe;

mod matrix_probe;

mod least_squares_probe;

mod svd_probe;

mod eigen_probe;

mod kronrod_probe;

mod root_probe;

mod minimum_probe;

mod newton_set_probe;

mod optimization_probe;

mod uzawa_probe;

mod polynomial_probe;

mod trig_probe;

mod sampled_probe;

mod all_roots_probe;

mod integration_probe;

mod pso_probe;

mod global_probe;

mod utils_probe;

mod modern_root_probe;

mod scalar_functor_probe;

mod vector_functor_probe;

mod modern_opt_probe;

mod modern_eigen_probe;
mod modern_factors_probe;
mod modern_integ_probe;
mod modern_multiple_probe;
mod modern_poly_probe;
mod modern_sys_probe;
mod modern_tensor_probe;
mod modern_trig_probe;

mod small_probe;

mod swarm_probe;

mod global_modern_probe;

mod cslib_probe;
