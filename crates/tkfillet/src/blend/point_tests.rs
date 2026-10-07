use super::*;

fn tangents() -> SectionTangents {
    SectionTangents {
        tg1: Vector3::new(1., 0., 0.),
        tg2: Vector3::new(0., 1., 0.),
        tg12d: Vector2::new(2., 3.),
        tg22d: Vector2::new(4., 5.),
    }
}

#[test]
fn default_point_is_a_tangency_point_without_supports() {
    let point = BlendPoint::new();
    assert!(point.is_tangency_point());
    let failure = point.parameters_on_s1().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_DomainError");
    assert!(point.parameter_on_c1().is_err() && point.tangent_on_c2().is_err());
}

#[test]
fn two_surface_point_carries_both_parameters_and_tangents() {
    let p1 = Point3::new(1., 2., 3.);
    let point = BlendPoint::on_surfaces_with_tangents(
        p1,
        Point3::default(),
        0.5,
        [1., 2., 3., 4.],
        tangents(),
    );
    assert_eq!(point.point_on_s1(), p1);
    assert_eq!(point.parameters_on_s1().unwrap(), (1., 2.));
    assert_eq!(point.tangent2d_on_s2().unwrap(), Vector2::new(4., 5.));
    assert_eq!(point.tangent_on_s2().unwrap(), Vector3::new(0., 1., 0.));
    assert!(point.parameter_on_c().is_err());
}

#[test]
fn surface_curve_point_has_no_second_surface() {
    let tg = CurveSectionTangents {
        tgs: Vector3::new(0., 0., 1.),
        tgc: Vector3::new(1., 0., 0.),
        tg2d: Vector2::new(1., 1.),
    };
    let point = BlendPoint::on_surface_curve_with_tangents(
        Point3::default(),
        Point3::default(),
        1.0,
        [0.1, 0.2, 0.3],
        tg,
    );
    assert_eq!(point.parameters_on_s().unwrap(), (0.1, 0.2));
    assert_eq!(point.parameter_on_c().unwrap(), 0.3);
    assert!(point.parameters_on_s2().is_err());
    assert!(point.tangent2d_on_s2().is_err());
    assert_eq!(point.tangent2d().unwrap(), Vector2::new(1., 1.));
    assert_eq!(point.tangent_on_c().unwrap(), Vector3::new(1., 0., 0.));
}

#[test]
fn restriction_points_carry_their_curve_parameters() {
    let point = BlendPoint::on_rsts(Point3::default(), Point3::default(), 0., [0.; 4], [7., 8.]);
    assert_eq!(
        (
            point.parameter_on_c1().unwrap(),
            point.parameter_on_c2().unwrap()
        ),
        (7., 8.)
    );
    let mut on_rst = BlendPoint::on_surface_rst_with_tangents(
        Point3::default(),
        Point3::default(),
        0.,
        [0.; 4],
        9.,
        tangents(),
    );
    assert_eq!(on_rst.parameter_on_c2().unwrap(), 9.);
    assert!(on_rst.parameter_on_c1().is_err());
    assert_eq!(on_rst.tangent_on_c1().unwrap(), Vector3::new(1., 0., 0.));
    on_rst.set_on_curves(Point3::default(), Point3::default(), 2., 1., 2.);
    assert!(on_rst.is_tangency_point() && on_rst.parameters_on_s1().is_err());
    assert_eq!(on_rst.parameter_on_c1().unwrap(), 1.);
    on_rst.set_parameter(5.);
    assert_eq!(on_rst.parameter(), 5.);
}

#[test]
fn set_value_without_tangents_resets_the_tangency_flag() {
    let mut point = BlendPoint::on_surfaces_with_tangents(
        Point3::default(),
        Point3::default(),
        0.,
        [0.; 4],
        tangents(),
    );
    assert!(!point.is_tangency_point());
    point.set_on_surfaces(Point3::default(), Point3::default(), 1., [1.; 4]);
    assert!(point.is_tangency_point() && point.tangent_on_s1().is_err());
    point.set_on_surface_rst(Point3::default(), Point3::default(), 1., [1.; 4], 3.);
    assert_eq!(point.parameter_on_c().unwrap(), 3.);
    let curve =
        BlendPoint::on_surface_curve(Point3::default(), Point3::new(1., 0., 0.), 2., [1., 2., 3.]);
    assert_eq!(curve.point_on_c(), Point3::new(1., 0., 0.));
    let rsts = BlendPoint::on_rsts_with_tangents(
        Point3::default(),
        Point3::default(),
        0.,
        [0.; 4],
        [1., 2.],
        tangents(),
    );
    assert_eq!(rsts.point_on_c1(), rsts.point_on_c2());
    assert_eq!(rsts.tangent_on_s().unwrap(), Vector3::new(1., 0., 0.));
    assert_eq!(
        BlendPoint::on_surface_rst(Point3::default(), Point3::default(), 0., [0.; 4], 1.)
            .point_on_s(),
        Point3::default()
    );
}
