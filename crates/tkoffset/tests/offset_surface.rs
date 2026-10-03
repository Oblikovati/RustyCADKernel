//! BRepOffset::Surface branch regressions; native fixtures cover the sampled outputs.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkoffset::brep_offset::{BRepOffsetStatus, surface};

#[test]
fn plane_offset_respects_the_parametric_normal() {
    for indirect in [false, true] {
        let input: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame(indirect))).cast();
        let (offset, status) = surface(&input, 3.0, false).unwrap();
        assert_eq!(status, BRepOffsetStatus::Good);
        let expected = Point3::new(0.0, 0.0, if indirect { -3.0 } else { 3.0 });
        assert!(offset.value(0.0, 0.0).unwrap().distance(&expected) < 1e-14);
    }
}

#[test]
fn radial_surfaces_preserve_good_reversed_and_degenerated_status() {
    for input in radial_inputs() {
        for (amount, expected) in [
            (1.0, BRepOffsetStatus::Good),
            (-3.0, BRepOffsetStatus::Reversed),
            (-2.0, BRepOffsetStatus::Degenerated),
        ] {
            let (offset, status) = surface(&input, amount, true).unwrap();
            assert_eq!(status, expected);
            assert!(!offset.is_null());
            if expected == BRepOffsetStatus::Degenerated {
                assert_eq!(offset.dynamic_type().unwrap().name(), "Geom_OffsetSurface");
            }
        }
    }
}

#[test]
fn indirect_cylinder_subtracts_the_offset_from_radius() {
    let input: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(frame(true), 2.0).unwrap()).cast();
    let (offset, status) = surface(&input, 3.0, false).unwrap();
    assert_eq!(status, BRepOffsetStatus::Reversed);
    assert_eq!(offset.cast::<GeomCylindricalSurface>().radius(), 1.0);
}

#[test]
fn cone_radius_crossing_keeps_native_good_status() {
    let input: Handle<dyn GeomSurface> =
        Handle::new(GeomConicalSurface::new(frame(false), 0.4, 2.0).unwrap()).cast();
    let (offset, status) = surface(&input, -3.0, false).unwrap();
    assert_eq!(status, BRepOffsetStatus::Good);
    let cone = offset.cast::<GeomConicalSurface>();
    assert_eq!(cone.semi_angle(), -0.4);
    assert_eq!(cone.ref_radius(), -(2.0 - 3.0 * 0.4_f64.cos()));
}

#[test]
fn spindle_torus_and_negative_minor_radius_use_generic_fallback() {
    for (major, minor, amount, expected) in [
        (2.0, 3.0, 1.0, BRepOffsetStatus::Good),
        (5.0, 2.0, -3.0, BRepOffsetStatus::Reversed),
    ] {
        let input: Handle<dyn GeomSurface> =
            Handle::new(GeomToroidalSurface::new(frame(false), major, minor).unwrap()).cast();
        let (offset, status) = surface(&input, amount, false).unwrap();
        assert_eq!(status, expected);
        assert_eq!(offset.dynamic_type().unwrap().name(), "Geom_OffsetSurface");
    }
}

#[test]
fn trimmed_surface_retains_bounds_and_recursive_status() {
    let basis: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(frame(false), 2.0).unwrap()).cast();
    let trimmed =
        GeomRectangularTrimmedSurface::new(&basis, (0.2, 1.0), (-1.0, 2.0), (true, true)).unwrap();
    let (offset, status) = surface(&Handle::new(trimmed).cast(), -3.0, false).unwrap();
    assert_eq!(status, BRepOffsetStatus::Reversed);
    assert_eq!(offset.bounds(), (0.2, 1.0, -1.0, 2.0));
    assert_eq!(
        offset.dynamic_type().unwrap().name(),
        "Geom_RectangularTrimmedSurface"
    );
}

#[test]
fn status_discriminants_match_native_enum() {
    assert_eq!(BRepOffsetStatus::Good as i32, 0);
    assert_eq!(BRepOffsetStatus::Reversed as i32, 1);
    assert_eq!(BRepOffsetStatus::Degenerated as i32, 2);
    assert_eq!(BRepOffsetStatus::Unknown as i32, 3);
}

fn frame(indirect: bool) -> Axis3 {
    let mut frame = Axis3::default();
    if indirect {
        frame.y_reverse();
    }
    frame
}

fn radial_inputs() -> [Handle<dyn GeomSurface>; 3] {
    [
        Handle::new(GeomCylindricalSurface::new(frame(false), 2.0).unwrap()).cast(),
        Handle::new(GeomSphericalSurface::new(frame(false), 2.0).unwrap()).cast(),
        Handle::new(GeomToroidalSurface::new(frame(false), 5.0, 2.0).unwrap()).cast(),
    ]
}
