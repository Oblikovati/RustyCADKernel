//! CSLib behavior beyond the legacy suite: singular normals, normal derivatives,
//! native quirks, failures and Class2d move semantics.
use rustycad_tkernel::ncollection::{Array1, Array2, DynamicArray};
use rustycad_tkmath::cslib::{
    self, Class2d, Class2dResult, NormalPolyDef, NormalStatus, ParametricPosition,
};
use rustycad_tkmath::gp::{Point2, Vector3};

const CENTER: ParametricPosition = ParametricPosition {
    u: 0.5,
    v: 0.5,
    u_min: 0.0,
    u_max: 1.0,
    v_min: 0.0,
    v_max: 1.0,
};

fn table(entries: &[((i32, i32), Vector3)], size: i32) -> Array2<'static, Vector3> {
    let mut table = Array2::with_bounds(0, size, 0, size).unwrap();
    for &(index, value) in entries {
        table[index] = value;
    }
    table
}

fn components(vector: Vector3) -> [f64; 3] {
    [vector.x(), vector.y(), vector.z()]
}

#[test]
fn second_derivative_normal_reports_each_null_and_parallel_case() {
    let x = Vector3::new(1.0, 0.0, 0.0);
    let y = Vector3::new(0.0, 1.0, 0.0);
    let zero = Vector3::new(0.0, 0.0, 0.0);
    let null = cslib::normal_from_second_derivatives(zero, zero, x, y, zero, 1e-6);
    assert_eq!(null.status, NormalStatus::D1NIsNull);
    assert!(!null.is_done());
    // D1U = 0: dN/du = D2U ^ D1V, dN/dv = D2UV ^ D1V.
    let parallel = cslib::normal_from_second_derivatives(zero, y, x, zero, x, 1e-6);
    assert_eq!(parallel.status, NormalStatus::D1NuIsParallelD1Nv);
    assert_eq!(components(parallel.normal.unwrap().into()), [0.0, 0.0, 1.0]);
    let z = Vector3::new(0.0, 0.0, 1.0);
    let spread = cslib::normal_from_second_derivatives(x, y, z, z, zero, 1e-6);
    assert_eq!(spread.status, NormalStatus::InfinityOfSolutions);
}

#[test]
fn null_cross_products_fail_like_gp_dir() {
    let x = Vector3::new(1.0, 0.0, 0.0);
    let failure = cslib::normal(x, x * 2.0, 0.0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    let zero = Vector3::new(0.0, 0.0, 0.0);
    let failure = cslib::normal_with_magnitude(zero, x, -1.0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}

#[test]
fn regular_singular_normal_uses_order_zero() {
    let der = table(&[((0, 0), Vector3::new(0.0, 0.0, 5.0))], 1);
    let result = cslib::normal_at_singularity(1, &der, 1e-7, CENTER).unwrap();
    assert_eq!(result.result.status, NormalStatus::Defined);
    assert_eq!(
        components(result.result.normal.unwrap().into()),
        [0.0, 0.0, 1.0]
    );
    assert_eq!((result.order_u, result.order_v), (0, 0));
}

#[test]
fn all_null_derivatives_are_singular_and_still_report_orders() {
    let der = table(&[], 2);
    let result = cslib::normal_at_singularity(2, &der, 1e-7, CENTER).unwrap();
    assert_eq!(result.result.status, NormalStatus::Singular);
    assert!(result.result.normal.is_none());
    assert_eq!((result.order_u, result.order_v), (0, 2));
}

#[test]
fn non_parallel_first_order_derivatives_define_the_normal() {
    let der = table(
        &[
            ((1, 0), Vector3::new(0.0, 0.0, 2.0)),
            ((0, 1), Vector3::new(1.0, 0.0, 0.0)),
        ],
        1,
    );
    let result = cslib::normal_at_singularity(1, &der, 1e-7, CENTER).unwrap();
    assert_eq!(result.result.status, NormalStatus::Defined);
    assert_eq!(
        components(result.result.normal.unwrap().into()),
        [0.0, 0.0, 1.0]
    );
    assert_eq!((result.order_u, result.order_v), (1, 0));
}

// Regression for the preserved native sign analysis: F = cos X changes sign at
// +-pi/2, yet the midpoint of [-pi/2, pi/2] is 0 where NormalPolyDef returns 0,
// and the index-0 interval restarts the comparison, so OCCT reports a unique
// normal (with the sign of the last interval) instead of InfinityOfSolutions.
#[test]
fn sign_analysis_keeps_native_zero_midpoint_and_first_interval_quirks() {
    let n = Vector3::new(0.0, 0.0, 1.0);
    let der = table(&[((1, 0), n), ((1, 1), Vector3::new(1.0, 2.0, 3.0))], 1);
    let result = cslib::normal_at_singularity(1, &der, 1e-7, CENTER).unwrap();
    assert_eq!(result.result.status, NormalStatus::Defined);
    assert_eq!(
        components(result.result.normal.unwrap().into()),
        [-0.0, -0.0, -1.0]
    );
    assert_eq!((result.order_u, result.order_v), (1, 0));
    // F = sin X: the change across pi/2..pi is detected.
    let der = table(&[((0, 1), n), ((1, 1), Vector3::new(1.0, 2.0, 3.0))], 1);
    let result = cslib::normal_at_singularity(1, &der, 1e-7, CENTER).unwrap();
    assert_eq!(result.result.status, NormalStatus::InfinityOfSolutions);
    assert_eq!((result.order_u, result.order_v), (0, 1));
}

#[test]
fn edge_domains_change_the_sign_analysis() {
    // On U = Umin the domain is [-pi/2, pi/2]. The native result (from the
    // oracle probe) is Defined with the sign of the last sampled interval.
    let n = Vector3::new(0.0, 0.0, 1.0);
    let der = table(&[((1, 0), n)], 1);
    let edge = ParametricPosition { u: 0.0, ..CENTER };
    let result = cslib::normal_at_singularity(1, &der, 1e-7, edge).unwrap();
    assert_eq!(result.result.status, NormalStatus::Defined);
    assert_eq!(
        components(result.result.normal.unwrap().into()),
        [-0.0, -0.0, -1.0]
    );
}

#[test]
fn degree_above_plib_range_fails_with_out_of_range() {
    let der = table(&[((26, 0), Vector3::new(0.0, 0.0, 1.0))], 26);
    let failure = cslib::normal_at_singularity(26, &der, 1e-7, CENTER).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}

#[test]
fn raw_normal_derivatives_follow_leibniz() {
    let x = Vector3::new(1.0, 0.0, 0.0);
    let y = Vector3::new(0.0, 1.0, 0.0);
    let surface = table(&[((1, 0), x), ((0, 1), y), ((2, 0), y)], 2);
    // d/du (Su ^ Sv) = Suu ^ Sv + Su ^ Suv = y ^ y + x ^ 0 = 0.
    assert_eq!(
        components(cslib::dnnuv(1, 0, &surface).unwrap()),
        [0.0, 0.0, 0.0]
    );
    let other = table(&[((0, 1), Vector3::new(0.0, 0.0, 1.0))], 2);
    let osculating = cslib::dnnuv_osculating(0, 0, &surface, &other).unwrap();
    assert_eq!(components(osculating), [0.0, -1.0, 0.0]);
    let failure = cslib::dnnuv(2, 0, &surface).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}

#[test]
fn unit_normal_derivative_of_a_turning_normal() {
    // N(u) = (1, u, 0): n'(0) = (0, 1, 0).
    let der = table(
        &[
            ((0, 0), Vector3::new(1.0, 0.0, 0.0)),
            ((1, 0), Vector3::new(0.0, 1.0, 0.0)),
        ],
        1,
    );
    assert_eq!(
        components(cslib::dn_normal(0, 0, &der, 0, 0).unwrap()),
        [1.0, 0.0, 0.0]
    );
    assert_eq!(
        components(cslib::dn_normal(1, 0, &der, 0, 0).unwrap()),
        [0.0, 1.0, 0.0]
    );
    let null = table(&[], 1);
    let failure = cslib::dn_normal(1, 0, &null, 0, 0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    let failure = cslib::dn_normal(-1, 1, &der, 0, 0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}

#[test]
fn normal_poly_def_rejects_invalid_degrees_and_coefficients() {
    let mut li = Array1::with_bounds(0, 26).unwrap();
    li[0] = 1.0;
    let failure = NormalPolyDef::new(26, &li).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    let failure = NormalPolyDef::new(-2, &li).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_RangeError");
    let short = Array1::with_bounds(0, 1).unwrap();
    let failure = NormalPolyDef::new(3, &short).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(NormalPolyDef::new(-1, &short).is_ok());
}

#[test]
fn moved_classifiers_answer_uncertain_and_transfer_state() {
    let mut points = DynamicArray::default();
    for (x, y) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
        points.append(Point2::new(x, y));
    }
    let mut source = Class2d::from_dynamic_array(&points, 1e-10, 1e-10, 0.0, 0.0, 1.0, 1.0);
    let inside = Point2::new(0.5, 0.5);
    let mut moved = Class2d::moved_from(&mut source);
    assert_eq!(source.si_dans(inside), Class2dResult::Uncertain);
    assert_eq!(moved.si_dans(inside), Class2dResult::Inside);
    let mut target = Class2d::default();
    target.move_from(&mut moved);
    assert_eq!(moved.si_dans(inside), Class2dResult::Uncertain);
    assert_eq!(target.si_dans_on_mode(inside, 0.01), Class2dResult::Inside);
    assert_eq!(Class2d::default().si_dans(inside), Class2dResult::Uncertain);
}
