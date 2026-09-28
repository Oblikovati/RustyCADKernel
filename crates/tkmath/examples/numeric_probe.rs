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
mod bnd_probe;
mod bounds_probe;
mod bvh_probe;
mod gp_planar_probe;
mod gp_standard_probe;
mod gp_transform_probe;
mod mathpoly_utils_probe;
mod ncollection_probe;
mod quaternion_probe;
mod scale_probe;
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut words = line.split_whitespace();
        let op = words.next().unwrap();
        let values: Vec<f64> = words.map(|w| w.parse().unwrap()).collect();
        // One arm per operation keeps concurrent probe additions merge-friendly.
        match op {
            "cslib_normal" => cslib_probe::cslib_normal_case(&values),
            "cslib_singular" => cslib_probe::cslib_singular_case(&values),
            "cslib_dn" => cslib_probe::cslib_dn_case(&values),
            "cslib_poly" => cslib_probe::cslib_poly_case(&values),
            "cslib_class2d" => cslib_probe::cslib_class2d_case(&values),
            "plib_poly" => plib_probe::poly_case(&values),
            "plib_poly2" => plib_probe::poly2_case(&values),
            "plib_rational" => plib_probe::rational_case(&values),
            "plib_lagrange" => plib_probe::lagrange_case(&values),
            "plib_cubic" => plib_probe::cubic_case(&values),
            "plib_hermite_coeffs" => plib_probe::hermite_coeffs_case(&values),
            "plib_hermite_interp" => plib_probe::hermite_interp_case(&values),
            "plib_coef_poles" => plib_probe::curve_case(&values, false),
            "plib_trim" => plib_probe::curve_case(&values, true),
            "plib_grid" => plib_probe::grid_case(&values),
            "plib_params" => plib_probe::params_case(&values),
            "plib_length" => plib_probe::length_case(&values),
            "plib_jacobi" => plib_probe::jacobi_probe_case(&values, false),
            "plib_hermit" => plib_probe::jacobi_probe_case(&values, true),
            "bspl_knots" => bspl_probe::knots_case(&values),
            "bspl_locate" => bspl_probe::locate_case(&values),
            "bspl_scheme" => bspl_probe::scheme_case(&values),
            "bspl_build" => bspl_probe::build_case(&values),
            "bspl_insert" => bspl_probe::insert_case(&values),
            "bspl_remove" => bspl_probe::remove_case(&values),
            "bspl_degree" => bspl_probe::degree_case(&values),
            "bspl_unperiodize" => bspl_probe::unperiodize_case(&values),
            "bspl_trim" => bspl_probe::trim_case(&values),
            "bspl_eval" => bspl_probe::eval_case(&values),
            "bspl_flat_eval" => bspl_probe::flat_eval_case(&values),
            "bspl_basis" => bspl_probe::basis_case(&values),
            "bspl_interpolate" => bspl_probe::interpolate_case(&values),
            "bspl_cache" => bspl_probe::cache_case(&values),
            "bspl_bezier" => bspl_probe::bezier_case(&values),
            "bspl_move" => bspl_probe::move_case(&values),
            "bspl_resolution" => bspl_probe::resolution_case(&values),
            "bspl_function" => bspl_probe::function_case(&values),
            "bspl_tang" => bspl_probe::tang_case(&values),
            "bspl_merge" => bspl_probe::merge_case(&values),
            "bspl_intervals" => bspl_probe::intervals_case(&values),
            "bsps_eval" => bsps_probe::eval_case(&values),
            "bsps_rational" => bsps_probe::rational_case(&values),
            "bsps_iso" => bsps_probe::iso_case(&values),
            "bsps_poles" => bsps_probe::poles_case(&values),
            "bsps_ops" => bsps_probe::ops_case(&values),
            "bsps_cache" => bsps_probe::cache_case(&values),
            "bsps_bezier" => bsps_probe::bezier_case(&values),
            "bsps_move" => bsps_probe::move_case(&values),
            "bsps_resolution" => bsps_probe::resolution_case(&values),
            "bsps_interpolate" => bsps_probe::interpolate_case(&values),
            "bsps_function" => bsps_probe::function_case(&values),
            "cv_conic" => convert_probe::conic_case(&values),
            "cv_cossin" => convert_probe::cossin_case(&values),
            "cv_polycs" => convert_probe::polycs_case(&values),
            "cv_surface" => convert_probe::surface_case(&values),
            "cv_bezier" => convert_probe::bezier_case(&values),
            "cv_comppoly" => convert_probe::comppoly_case(&values),
            "cv_grid" => convert_probe::grid_case(&values),
            "obb_build" => bnd_probe::obb_build_case(&values),
            "obb_pair" => bnd_probe::obb_pair_case(&values),
            "sortbox" => bnd_probe::sortbox_case(&values),
            "box_extra" => bnd_probe::box_extra_case(&values),
            "box2_extra" => bnd_probe::box2_extra_case(&values),
            "compact_trsf" => bnd_probe::compact_trsf_case(&values),
            "modern_lbfgs" => modern_opt_probe::modern_lbfgs_case(&values),
            "modern_newton" => modern_opt_probe::modern_newton_case(&values),
            "modern_global" => global_modern_probe::modern_global_case(&values),
            "modern_swarm" => swarm_probe::modern_swarm_case(&values),
            "modern_small" => small_probe::modern_small_case(&values),
            "modern_lm" => modern_sys_probe::modern_lm_case(&values),
            "modern_sys" => modern_sys_probe::modern_sys_case(&values),
            "modern_tensor" => modern_tensor_probe::modern_tensor_case(&values),
            "modern_double_exp" => modern_integ_probe::modern_double_exp_case(&values),
            "modern_kronrod" => modern_integ_probe::modern_kronrod_case(&values),
            "modern_integ" => modern_integ_probe::modern_integ_case(&values),
            "modern_multiple" => modern_multiple_probe::modern_multiple_case(&values),
            "modern_trig" => modern_trig_probe::modern_trig_case(&values),
            "modern_poly" => modern_poly_probe::modern_poly_case(&values),
            "modern_laguerre" => modern_poly_probe::modern_laguerre_case(&values),
            "modern_complex" => modern_poly_probe::modern_complex_case(&values),
            "modern_factors" => modern_factors_probe::modern_factors_case(&values),
            "modern_lu" => modern_eigen_probe::modern_lu_case(&values),
            "modern_eigen" => modern_eigen_probe::modern_eigen_case(&values),
            "modern_frpr" => modern_opt_probe::frpr_case(&values),
            "scalar_opt" => modern_root_probe::scalar_opt_case(&values),
            "modern_opt" => modern_opt_probe::modern_opt_case(&values),
            "vector_functor" => vector_functor_probe::vector_functor_case(&values),
            "scalar_functor" => scalar_functor_probe::scalar_functor_case(&values),
            "modern_root" => modern_root_probe::modern_root_case(&values),
            "utils_line" => utils_probe::line_case(&values),
            "utils_core" => utils_probe::core_case(&values),
            "utils_bracket" => utils_probe::bracket_case(&values),
            "global" => global_probe::global_case(&values),
            "pso" => pso_probe::pso_case(&values),
            "bullard" => pso_probe::bullard_case(&values),
            "integration" => integration_probe::integration_case(&values),
            "quadrature" => integration_probe::quadrature_case(&values),
            "all_roots" => all_roots_probe::all_roots_case(&values),
            "sampled" => sampled_probe::sampled_case(&values),
            "trig" => trig_probe::trig_case(&values),
            "polynomial" => polynomial_probe::polynomial_case(&values),
            "uzawa" => uzawa_probe::uzawa_case(&values),
            "optimization" => optimization_probe::optimization_case(&values),
            "function_set" => newton_set_probe::function_set_case(&values),
            "newton_set" => newton_set_probe::newton_set_case(&values),
            "minimum" => minimum_probe::minimum_case(&values),
            "root" => root_probe::root_case(&values),
            "kronrod" => kronrod_probe::kronrod_case(&values),
            "eigen" => eigen_probe::eigen_case(&values),
            "svd" => svd_probe::svd_case(&values),
            "least_squares" => least_squares_probe::least_squares_case(&values),
            "numeric_matrix" => matrix_probe::numeric_matrix_case(&values),
            "numeric_vector" => vector_probe::numeric_vector_case(&values),
            "nodes" => nodes_probe::nodes_case(&values),
            "table" => table_probe::table_case(&values),
            "quaternion" => quaternion_probe::quaternion_case(&values),
            "scale" => scale_probe::scale_case(&values),
            "compact3" => bounds_probe::compact3_case(&values),
            "compact2" => bounds_probe::compact2_case(&values),
            "sphere" => bounds_probe::sphere_case(&values),
            "bounds2" => bounds_probe::bounds2_case(&values),
            "bounds" => bounds_probe::bounds_case(&values),
            "line_plane" => bounds_probe::line_plane_case(&values),
            "planar" => planar_case(&values),
            "frame" => frame_case(&values),
            "direction" => direction_case(&values),
            "point" => point_case(&values),
            "xyz" => vector_case(&values),
            "matrix" => matrix_case(&values),
            "range" => range_case(&values),
            "elclib3" => elementary_probe::elclib3_case(&values),
            "elclib2" => elementary_probe::elclib2_case(&values),
            "elperiod" => elementary_probe::elperiod_case(&values),
            "elto3d" => elementary_probe::elto3d_case(&values),
            "elslib" => elementary_probe::elslib_case(&values),
            "gpconic3" => elementary_probe::gpconic3_case(&values),
            "gpconic2" => elementary_probe::gpconic2_case(&values),
            "gpquadric" => elementary_probe::gpquadric_case(&values),
            "gpaxis" => elementary_probe::gpaxis_case(&values),
            "bvh_box" => bvh_probe::bvh_box_case(&values),
            "bvh_int_floor" => bvh_probe::bvh_int_floor_case(&values),
            "bvh_sort" => bvh_probe::bvh_sort_case(&values),
            "bvh_build" => bvh_probe::bvh_build_case(&values),
            "bvh_transform" => bvh_probe::bvh_transform_case(&values),
            "bvh_builders" => bvh_probe::bvh_builders_case(&values),
            "bvh_traverse" => bvh_probe::bvh_traverse_case(&values),
            "bvh_nearest" => bvh_probe::bvh_nearest_case(&values),
            "bvh_field" => bvh_probe::bvh_field_case(&values),
            "gp2d_xy" => gp_planar_probe::xy_case(&values),
            "gp2d_mat" => gp_planar_probe::mat_case(&values),
            "gp2d_trsf" => gp_planar_probe::trsf_case(&values),
            "gp2d_gtrsf" => gp_planar_probe::gtrsf_case(&values),
            "gp2d_geom" => gp_planar_probe::geom_case(&values),
            "gp_standard" => gp_standard_probe::gp_standard_case(&values),
            "mathpoly_utils_long" => mathpoly_utils_probe::mathpoly_utils_long_case(&values),
            "ncollection_vec" => ncollection_probe::ncollection_vec_case(&values),
            "ncollection_mat" => ncollection_probe::ncollection_mat_case(&values),
            "gp3d_trsf" => gp_transform_probe::gp3_trsf_case(&values),
            "gp3d_gtrsf" => gp_transform_probe::gp3_gtrsf_case(&values),
            "gp3d_quat" => gp_transform_probe::gp3_quat_case(&values),
            "mathpoly_utils_complex" => mathpoly_utils_probe::mathpoly_utils_complex_case(&values),
            _ => panic!("Unknown probe operation: {op}"),
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

mod bspl_probe;
mod bsps_probe;
mod convert_probe;
mod cslib_probe;
mod elementary_probe;
mod plib_probe;
