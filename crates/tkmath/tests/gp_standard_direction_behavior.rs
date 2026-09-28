//! Supplemental gp_Dir::D constructors and gp statics behavior pinned against native OCCT.
use rustycad_tkmath::gp::{self, Axis1, Axis2, Axis3, Direction3, Point3, StandardDirection3, Xyz};

const NAMES: [StandardDirection3; 6] = [
    StandardDirection3::X,
    StandardDirection3::Y,
    StandardDirection3::Z,
    StandardDirection3::NegX,
    StandardDirection3::NegY,
    StandardDirection3::NegZ,
];

fn xyz(direction: Direction3) -> Xyz {
    Xyz::from(direction)
}

#[test]
fn standard_directions_are_exact_unit_axes() {
    let expected = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [-1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, -1.0],
    ];
    for (name, coordinates) in NAMES.into_iter().zip(expected) {
        assert_eq!(Direction3::from_standard(name).coordinates(), &coordinates);
        assert_eq!(Direction3::from(name), Direction3::from_standard(name));
    }
}

#[test]
fn standard_frames_use_the_cross_table_for_orthogonal_pairs() {
    let origin = Point3::new(1.0, 2.0, 3.0);
    for normal in NAMES {
        for x in NAMES {
            let frame = Axis3::with_standard_frame(origin, normal, x);
            let (n, v) = (xyz(frame.direction()), xyz(frame.x_direction()));
            if n.dot(v) != 0.0 {
                continue;
            }
            assert_eq!(xyz(frame.y_direction()), n.crossed(v), "{normal:?} x {x:?}");
            assert!(frame.direct().unwrap());
            let ax2 = Axis2::with_standard_frame(origin, normal, x);
            assert_eq!(xyz(ax2.y_direction()), n.crossed(v));
        }
    }
}

#[test]
fn parallel_standard_pairs_keep_the_native_fallback() {
    use StandardDirection3::*;
    let fallback =
        |normal, x| xyz(Axis3::with_standard_frame(Point3::default(), normal, x).y_direction());
    assert_eq!(fallback(X, X), Xyz::new(0.0, 0.0, 1.0));
    assert_eq!(fallback(Y, NegY), Xyz::new(1.0, 0.0, 0.0));
    assert_eq!(fallback(Z, Z), Xyz::new(0.0, 1.0, 0.0));
    assert_eq!(fallback(NegX, X), Xyz::new(0.0, 0.0, -1.0));
    assert_eq!(fallback(NegY, NegY), Xyz::new(-1.0, 0.0, 0.0));
    assert_eq!(fallback(NegZ, Z), Xyz::new(0.0, -1.0, 0.0));
}

#[test]
fn single_standard_direction_frames_are_right_handed() {
    for normal in NAMES {
        let frame = Axis3::from_standard(normal);
        let (n, x, y) = (
            xyz(frame.direction()),
            xyz(frame.x_direction()),
            xyz(frame.y_direction()),
        );
        assert_eq!(n.crossed(x), y, "{normal:?}");
        assert_eq!(frame.location(), Point3::default());
        assert_eq!(
            Axis2::from_standard(normal).axis3().x_direction(),
            frame.x_direction()
        );
        let moved = Axis3::with_standard(Point3::new(1.0, 0.0, 0.0), normal);
        assert_eq!(moved.location(), Point3::new(1.0, 0.0, 0.0));
        assert_eq!(Axis1::from_standard(normal).direction(), frame.direction());
    }
}

#[test]
fn gp_statics_match_the_native_frames() {
    assert_eq!(gp::resolution(), f64::MIN_POSITIVE);
    assert_eq!(gp::origin(), Point3::default());
    assert_eq!(gp::dz(), Direction3::Z);
    assert_eq!(gp::oy().direction(), Direction3::Y);
    // gp::ZOX is gp_Ax2(origin, D::Y): its X direction is Z and its Y direction is X.
    let zox = gp::zox();
    assert_eq!(zox.x_direction(), Direction3::Z);
    assert_eq!(zox.y_direction(), Direction3::X);
    let yoz = gp::yoz();
    assert_eq!(
        (yoz.x_direction(), yoz.y_direction()),
        (Direction3::Y, Direction3::Z)
    );
    assert_eq!(gp::xoy().x_direction(), Direction3::X);
    assert_eq!(gp::ox2d().direction(), gp::dx2d());
    assert_eq!(gp::oy2d().location(), gp::origin2d());
    assert_eq!(gp::dy2d().y(), 1.0);
    assert_eq!(gp::dx().x(), 1.0);
    assert_eq!(gp::ox().location(), gp::oz().location());
}
