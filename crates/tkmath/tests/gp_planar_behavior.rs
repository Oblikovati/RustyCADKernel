//! Supplemental gp_XY, gp_Mat2d and planar primitive behavior pinned against native OCCT.
use rustycad_tkmath::gp::{
    Axis2d, Direction2, Line2, Matrix2, Point2, StandardDirection2, Transform2, Xy,
};

#[test]
fn xy_members_follow_native_order_and_checks() {
    let mut xy = Xy::new(3.0, 4.0);
    assert_eq!(xy.modulus(), 5.0);
    assert!(!xy.is_equal(&xy, 0.0), "strict tolerance");
    assert_eq!(xy.coord(2).unwrap(), 4.0);
    assert!(xy.coord(3).is_err());
    assert_eq!(
        xy.set_coord(0, 1.0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    *xy.change_coord(1).unwrap() = 6.0;
    assert_eq!(xy, Xy::new(6.0, 4.0));
    assert_eq!(xy ^ Xy::new(1.0, 0.0), -4.0);
    assert_eq!(xy * Xy::new(1.0, 2.0), 14.0);
    assert_eq!(xy.cross_magnitude(Xy::new(1.0, 0.0)), 4.0);
    assert_eq!(xy.cross_square_magnitude(Xy::new(1.0, 0.0)), 16.0);
    assert_eq!(-xy, Xy::new(-6.0, -4.0));
    let mut zero = Xy::default();
    assert_eq!(
        zero.normalize().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert!(zero.normalized().is_err());
}

#[test]
fn xy_linear_forms_and_matrix_products() {
    let (a, b) = (Xy::new(1.0, 2.0), Xy::new(3.0, 5.0));
    let mut xy = Xy::default();
    xy.set_linear_form(2.0, a, 3.0, b);
    assert_eq!(xy, Xy::new(11.0, 19.0));
    xy.set_linear_form_offset(2.0, a, 3.0, b, a);
    assert_eq!(xy, Xy::new(12.0, 21.0));
    xy.set_scaled_sum(2.0, a, b);
    assert_eq!(xy, Xy::new(5.0, 9.0));
    xy.set_sum(a, b);
    assert_eq!(xy, Xy::new(4.0, 7.0));
    let m = Matrix2::new([[1.0, 2.0], [3.0, 4.0]]);
    assert_eq!(a * m, Xy::new(5.0, 11.0));
    assert_eq!(m * a, a.multiplied_matrix(m));
    let mut c = a;
    c *= m;
    assert_eq!(c, Xy::new(5.0, 11.0));
    c = a;
    c *= b;
    assert_eq!(c, Xy::new(3.0, 10.0));
    assert_eq!(2.0 * a, Xy::new(2.0, 4.0));
    c /= 2.0;
    assert_eq!(c, Xy::new(1.5, 5.0));
}

#[test]
fn matrix2_accessors_and_release_index_rules() {
    let mut m = Matrix2::from_columns(Xy::new(1.0, 3.0), Xy::new(2.0, 4.0));
    assert_eq!(m.coefficients(), &[[1.0, 2.0], [3.0, 4.0]]);
    assert_eq!(m.determinant(), -2.0);
    assert!(!m.is_singular());
    assert_eq!(m.diagonal(), Xy::new(1.0, 4.0));
    assert_eq!(m.value(2, 1).unwrap(), 3.0);
    assert!(m.value(3, 1).is_err());
    assert!(m.set_value(1, 0, 1.0).is_err());
    *m.change_value(1, 1).unwrap() = 5.0;
    // Release gp_Mat2d.cxx reads and writes column/row 2 for any index other than 1.
    if !cfg!(feature = "occt-debug-checks") {
        assert_eq!(m.column(7).unwrap(), Xy::new(2.0, 4.0));
        assert_eq!(m.row(0).unwrap(), Xy::new(3.0, 4.0));
        m.set_column(-3, Xy::new(8.0, 9.0)).unwrap();
        assert_eq!(m.coefficients(), &[[5.0, 8.0], [3.0, 9.0]]);
        m.set_row(5, Xy::new(1.0, 1.0)).unwrap();
        assert_eq!(m.coefficients(), &[[5.0, 8.0], [1.0, 1.0]]);
    }
    m.set_rows(Xy::new(1.0, 2.0), Xy::new(3.0, 4.0));
    m.set_diagonal(7.0, 8.0);
    assert_eq!(m.coefficients(), &[[7.0, 2.0], [3.0, 8.0]]);
    m.set_scale(2.0);
    assert_eq!(m.coefficients(), &[[2.0, 0.0], [0.0, 2.0]]);
    m.set_identity();
    assert_eq!(m, Matrix2::identity());
    assert_eq!(Matrix2::default().coefficients(), &[[0.0; 2]; 2]);
}

#[test]
fn matrix2_algebra_matches_native_operation_order() {
    let m = Matrix2::new([[1.0, 2.0], [3.0, 4.0]]);
    let o = Matrix2::new([[0.0, 1.0], [1.0, 0.0]]);
    assert_eq!(m * o, Matrix2::new([[2.0, 1.0], [4.0, 3.0]]));
    let mut pre = m;
    pre.pre_multiply(o);
    assert_eq!(pre, o * m);
    assert_eq!(m + o, Matrix2::new([[1.0, 3.0], [4.0, 4.0]]));
    assert_eq!(m - o, Matrix2::new([[1.0, 1.0], [2.0, 4.0]]));
    assert_eq!(2.0 * m, m * 2.0);
    assert_eq!(m / 2.0, Matrix2::new([[0.5, 1.0], [1.5, 2.0]]));
    assert_eq!(m.transposed(), Matrix2::new([[1.0, 3.0], [2.0, 4.0]]));
    assert_eq!(
        m.inverted().unwrap(),
        Matrix2::new([[-2.0, 1.0], [1.5, -0.5]])
    );
    assert_eq!(m.powered(0).unwrap(), Matrix2::identity());
    assert_eq!(m.powered(2).unwrap(), m * m);
    let singular = Matrix2::new([[1.0, 2.0], [2.0, 4.0]]);
    assert!(singular.is_singular());
    if !cfg!(feature = "occt-debug-checks") {
        // The zero-determinant raise is compiled only with OCCT_DEBUG.
        assert!(singular.inverted().unwrap().coefficients()[0][0].is_infinite());
    }
}

#[test]
fn matrix2_power_reproduces_the_aliased_native_square() {
    // gp_Mat2d::Power squares with aTemp.Multiply(aTemp), which reads the freshly stored [0][1]
    // while computing [1][1]; OCCT returns these bits for a 0.3 rad rotation cubed.
    let mut rotation = Matrix2::default();
    rotation.set_rotation(0.3);
    let cubed = rotation.powered(3).unwrap();
    assert_eq!(
        cubed.coefficients(),
        &[
            [0.6216099682706644, -0.759823872010906],
            [0.7833269096274832, 0.5456310371035684]
        ]
    );
    // One multiplication (power 2) has no squaring and stays exact.
    assert_eq!(rotation.powered(2).unwrap(), rotation * rotation);
}

#[test]
fn standard_directions_build_exact_axes() {
    let expected = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)];
    let names = [
        StandardDirection2::X,
        StandardDirection2::Y,
        StandardDirection2::NegX,
        StandardDirection2::NegY,
    ];
    for (name, (x, y)) in names.into_iter().zip(expected) {
        let direction = Direction2::from_standard(name);
        assert_eq!((direction.x(), direction.y()), (x, y));
        assert_eq!(Direction2::from(name), direction);
        assert_eq!(Axis2d::from_standard(name).direction(), direction);
        let axis = Axis2d::with_standard(Point2::new(1.0, 2.0), name);
        assert_eq!(axis.location(), Point2::new(1.0, 2.0));
    }
    assert_eq!(
        Direction2::NEG_X,
        Direction2::from_standard(StandardDirection2::NegX)
    );
}

#[test]
fn direction2_setters_renormalize_or_keep_the_old_value() {
    let mut direction = Direction2::X;
    direction.set_coordinates(3.0, 4.0).unwrap();
    assert_eq!(direction.xy(), Xy::new(0.6, 0.8));
    direction.set_y(0.0).unwrap();
    assert_eq!(direction, Direction2::X);
    assert_eq!(
        direction.set_coord(3, 1.0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        direction.set_x(0.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(direction, Direction2::X);
    direction.set_coord(2, 1.0).unwrap();
    assert!((direction.x() - direction.y()).abs() == 0.0);
    direction.set_xy(Xy::new(0.0, -2.0)).unwrap();
    assert_eq!(direction, Direction2::NEG_Y);
    assert_eq!(direction.coord(2).unwrap(), -1.0);
    assert_eq!(-direction, Direction2::Y);
    assert_eq!(Direction2::X ^ Direction2::Y, 1.0);
    assert_eq!(Direction2::X * Direction2::Y, 0.0);
}

#[test]
fn direction2_from_overflowing_components_is_zero() {
    // The squared norm overflows to infinity, so both components divide to zero.
    let direction = Direction2::new(-1e300, 1e-160).unwrap();
    assert_eq!(direction.x(), 0.0);
    assert!(direction.x().is_sign_negative());
    assert_eq!(direction.y(), 0.0);
    let line = Line2::new(Point2::default(), direction);
    // Native gp_Lin2d::Normal is noexcept and terminates here; Rust reports the raise.
    assert_eq!(
        line.normal(Point2::default()).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn direction2_transform_handles_every_form() {
    let direction = Direction2::new(0.6, 0.8).unwrap();
    let center = Point2::new(1.0, 1.0);
    assert_eq!(
        direction.transformed(Transform2::mirror_point(center)),
        direction.reversed()
    );
    assert_eq!(
        direction.transformed(Transform2::scaling(center, -2.0)),
        -direction
    );
    assert_eq!(
        direction.transformed(Transform2::scaling(center, 3.0)),
        direction
    );
    let mut moved = direction;
    moved.transform(Transform2::translation([1.0, 2.0]));
    assert_eq!(moved, direction);
    moved.rotate(std::f64::consts::FRAC_PI_2);
    assert!((moved.x() + 0.8).abs() < 1e-15 && (moved.y() - 0.6).abs() < 1e-15);
    moved = direction;
    moved.mirror_direction(Direction2::X);
    assert_eq!(moved, direction.mirrored_direction(Direction2::X));
    moved = direction;
    moved.mirror_axis(Axis2d::default());
    assert_eq!(moved, direction.mirrored_axis(Axis2d::default()));
}
