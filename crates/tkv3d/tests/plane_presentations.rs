//! Plane builders exercise actual graphic callbacks, including native submission order.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{geom::GeomPlane, geom_adaptor::GeomAdaptorSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{dsg_prs::*, prs3d::*, std_prs::StdPrsPlane, v3d::V3dViewer};
fn plane_fixture() -> (
    Handle<Graphic3dStructure>,
    Handle<Prs3dDrawer>,
    display_driver::DisplayTrace,
) {
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.set_plane_aspect(Handle::new(Prs3dPlaneAspect::new().unwrap()));
    drawer.set_shading_aspect(Handle::new(Prs3dShadingAspect::new().unwrap()));
    trace.write().events.clear();
    (prs, drawer, trace)
}
fn surface() -> GeomAdaptorSurface {
    GeomAdaptorSurface::with_surface(&Handle::new(GeomPlane::new(Axis3::default())).cast()).unwrap()
}
fn primitive_packets(events: &[f64]) -> Vec<(i32, Vec<[f64; 3]>)> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < events.len() {
        match events[offset] as i32 {
            2 => offset += 1,
            3 => offset += 8,
            4 => {
                let count = events[offset + 2] as usize;
                let start = offset + 3;
                let vertices = events[start..start + count * 3].as_chunks::<3>().0.to_vec();
                result.push((events[offset + 1] as i32, vertices));
                offset = start + count * 3;
                offset += 1 + events[offset] as usize;
            }
            code => panic!("event {code}: expected group, aspect or primitive packet"),
        }
    }
    result
}
#[test]
fn datum_plane_builders_close_the_triangle_and_retain_native_topology() {
    for shaded in [false, true] {
        check_datum_plane(shaded);
    }
}
fn check_datum_plane(shaded: bool) {
    let (prs, drawer, trace) = plane_fixture();
    let points = [
        Point3::new(1., 2., 3.),
        Point3::new(4., 5., 6.),
        Point3::new(7., 8., 9.),
    ];
    if shaded {
        DsgPrsShadedPlanePresentation::add(
            &prs.cast(),
            &drawer,
            &points[0],
            &points[1],
            &points[2],
        )
        .unwrap();
    } else {
        DsgPrsXYZPlanePresentation::add(&prs.cast(), &drawer, &points[0], &points[1], &points[2])
            .unwrap();
    }
    let trace = trace.read();
    let packets = primitive_packets(&trace.events);
    let kind = if shaded {
        Graphic3dTypeOfPrimitiveArray::Polygons
    } else {
        Graphic3dTypeOfPrimitiveArray::Polylines
    };
    assert_eq!(
        packets,
        vec![(
            kind as i32,
            vec![[1., 2., 3.], [4., 5., 6.], [7., 8., 9.], [1., 2., 3.]]
        )]
    );
    assert_eq!(trace.events[1], 3.);
    assert_eq!(trace.events[9], if shaded { 3. } else { 4. });
}
#[test]
fn rectangular_plane_edges_and_isos_use_native_corner_and_increment_order() {
    let (prs, drawer, trace) = plane_fixture();
    let aspect = drawer.plane_aspect();
    aspect.set_plane_length(4., 6.);
    aspect.set_display_iso(true);
    aspect.set_iso_distance(2.);
    StdPrsPlane::add(&prs.cast(), &surface(), &drawer).unwrap();
    let trace = trace.read();
    let packets = primitive_packets(&trace.events);
    assert_eq!(packets.len(), 2);
    assert_eq!(
        packets[0].1,
        [
            [-2., 3., 0.],
            [2., 3., 0.],
            [2., -3., 0.],
            [-2., -3., 0.],
            [-2., 3., 0.]
        ]
    );
    assert_eq!(
        packets[1].1,
        [
            [0., 3., 0.],
            [0., -3., 0.],
            [2., -1., 0.],
            [-2., -1., 0.],
            [2., 1., 0.],
            [-2., 1., 0.]
        ]
    );
    assert_eq!(trace.events[trace.events.len() - 8], 3.);
}
#[test]
fn arrows_preserve_center_shaft_first_and_corner_shafts_last() {
    let (prs, drawer, trace) = plane_fixture();
    let aspect = drawer.plane_aspect();
    aspect.set_display_edges(false);
    aspect.set_display_center_arrow(true);
    aspect.set_display_edges_arrows(true);
    aspect.set_arrows_size(2.);
    StdPrsPlane::add(&prs.cast(), &surface(), &drawer).unwrap();
    let packets = primitive_packets(&trace.read().events);
    assert_eq!(packets.len(), 7);
    assert_eq!(packets[0].1, [[0., 0., 0.], [0., 0., 2.]]);
    assert_eq!(
        packets.last().unwrap().1,
        [
            [-0.5, -0.5, 0.],
            [-0.5, -0.5, 2.],
            [-0.5, 0.5, 0.],
            [-0.5, 0.5, 2.],
            [0.5, 0.5, 0.],
            [0.5, 0.5, 2.],
            [0.5, -0.5, 0.],
            [0.5, -0.5, 2.]
        ]
    );
}
#[test]
fn match_uses_infinite_plane_inclusive_distance_and_ignores_null_drawer() {
    for (x, z, distance, expected) in [
        (1e10, 1., 1., true),
        (0., -1., 1., true),
        (0., 1., 0.999, false),
        (0., 0., -1., false),
        (0., 0., f64::NAN, false),
    ] {
        assert_eq!(
            StdPrsPlane::matches(x, 0., z, distance, &surface(), &Handle::default()).unwrap(),
            expected
        );
    }
}
#[test]
fn iso_undefined_casts_report_values() {
    for spacing in [0., f64::NAN] {
        let (prs, drawer, _) = plane_fixture();
        drawer.plane_aspect().set_display_iso(true);
        drawer.plane_aspect().set_iso_distance(spacing);
        assert!(
            StdPrsPlane::add(&prs.cast(), &surface(), &drawer).is_err(),
            "spacing {spacing}"
        );
    }
}

#[test]
fn nonpositive_iso_capacity_keeps_native_empty_submission_behavior() {
    for spacing in [f64::INFINITY, 10., 1.] {
        let (prs, drawer, trace) = plane_fixture();
        let aspect = drawer.plane_aspect();
        aspect.set_display_edges(false);
        aspect.set_display_iso(true);
        aspect.set_iso_distance(spacing);
        StdPrsPlane::add(&prs.cast(), &surface(), &drawer).unwrap();
        assert!(primitive_packets(&trace.read().events).is_empty());
    }
}

#[test]
fn nonplane_creates_current_group_without_submitting_aspects_or_vertices() {
    let (prs, drawer, trace) = plane_fixture();
    let sphere =
        Handle::new(rustycad_tkg3d::geom::GeomSphericalSurface::new(Axis3::default(), 2.).unwrap());
    let surface = GeomAdaptorSurface::with_surface(&sphere.cast()).unwrap();
    StdPrsPlane::add(&prs.cast(), &surface, &drawer).unwrap();
    assert_eq!(trace.read().events, [2.]);
    assert!(!StdPrsPlane::matches(0., 0., 0., 100., &surface, &drawer).unwrap());
}
