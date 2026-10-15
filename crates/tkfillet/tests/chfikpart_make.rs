//! The closed-form fillets, chamfers and corners of ChFiKPart written into the data
//! structure: the kind of the blend surface, and the edges and pcurves of the two
//! contacts agree with the surfaces they lie on.
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::chfi_ds::{ChamfMode, SurfData, SurfDataHandle, lock, shared};
use rustycad_tkfillet::chfikpart::{self, ChFiKPartComputeData};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomCylindricalSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation::{Forward, Reversed};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Direction3, Line3, Point2, Point3};
use std::f64::consts::PI;

fn frame(p: (f64, f64, f64), n: (f64, f64, f64), x: (f64, f64, f64)) -> Axis3 {
    Axis3::new(
        Point3::new(p.0, p.1, p.2),
        Direction3::new(n.0, n.1, n.2).unwrap(),
        Direction3::new(x.0, x.1, x.2).unwrap(),
    )
    .unwrap()
}

fn plane(n: (f64, f64, f64), x: (f64, f64, f64)) -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(frame((0., 0., 0.), n, x))).cast()
}

fn cylinder(p: (f64, f64, f64), n: (f64, f64, f64), x: (f64, f64, f64)) -> Handle<dyn GeomSurface> {
    Handle::new(GeomCylindricalSurface::new(frame(p, n, x), 3.0).unwrap()).cast()
}

fn cone() -> Handle<dyn GeomSurface> {
    let f = frame((0., 0., 0.), (0., 0., 1.), (1., 0., 0.));
    Handle::new(GeomConicalSurface::new(f, 0.5, 3.0).unwrap()).cast()
}

fn adaptor(s: &Handle<dyn GeomSurface>) -> GeomAdaptorSurface {
    GeomAdaptorSurface::with_surface(s).unwrap()
}

fn plane_z() -> Handle<dyn GeomSurface> {
    plane((0., 0., 1.), (1., 0., 0.))
}

fn circle_z() -> Circle3 {
    let ax = Axis2::with_x_direction(
        Point3::default(),
        Direction3::new(0., 0., 1.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    Circle3::new(ax, 3.0).unwrap()
}

fn line(p: (f64, f64, f64), d: (f64, f64, f64)) -> Line3 {
    Line3::new(
        Point3::new(p.0, p.1, p.2),
        Direction3::new(d.0, d.1, d.2).unwrap(),
    )
}

/// The kind of the blend surface; on each contact the 3d edge, its pcurve on the face and
/// its pcurve on the blend give the same points.
fn check(
    ds: &TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    faces: [&Handle<dyn GeomSurface>; 2],
) -> SurfaceType {
    let d = lock(data);
    let blend = ds.surface(d.surf()).surface().clone();
    for (i, face) in (1..=2).zip(faces) {
        let fi = d.interference(i);
        if fi.line_index() == 0 {
            continue;
        }
        let c = ds.curve(fi.line_index()).curve().clone();
        let (f, l) = (fi.first_parameter(), fi.last_parameter());
        for t in [f, 0.5 * (f + l), l] {
            let p = c.get().unwrap().value(t).unwrap();
            let uv = fi.pcurve_on_face().get().unwrap().value(t).unwrap();
            let on_face = face.get().unwrap().value(uv.x(), uv.y()).unwrap();
            let st = fi.pcurve_on_surf().get().unwrap().value(t).unwrap();
            let on_blend = blend.get().unwrap().value(st.x(), st.y()).unwrap();
            assert!(
                p.distance(&on_face) < 1e-9,
                "contact {i} at {t} off its face"
            );
            assert!(
                p.distance(&on_blend) < 1e-9,
                "contact {i} at {t} off the blend"
            );
        }
    }
    adaptor(&blend).get_type().unwrap()
}

/// The kind of the blend surface alone (the corners of arbitrary points are not checked
/// for contact: the numeric oracle compares them).
fn blend_type(ds: &TopOpeBRepDSDataStructure, data: &SurfDataHandle) -> SurfaceType {
    adaptor(ds.surface(lock(data).surf()).surface())
        .get_type()
        .unwrap()
}

fn data() -> (TopOpeBRepDSDataStructure, SurfDataHandle) {
    (TopOpeBRepDSDataStructure::new(), shared(SurfData::new()))
}

#[test]
fn the_fillet_of_two_planes_is_a_cylinder() {
    let (s1, s2) = (plane_z(), plane((1., 0., 0.), (0., 1., 0.)));
    let (p1, p2) = (adaptor(&s1).plane().unwrap(), adaptor(&s2).plane().unwrap());
    let (mut ds, d) = data();
    let spine = line((0., 0., 0.), (0., 1., 0.));
    assert!(
        chfikpart::make_fillet_pln_pln(
            &mut ds,
            &d,
            (&p1, &p2),
            (Forward, Forward),
            0.5,
            &spine,
            0.,
            Forward
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Cylinder);
}

#[test]
fn the_chamfers_of_two_planes_are_planes() {
    let (s1, s2) = (plane_z(), plane((1., 0., 0.), (0., 1., 0.)));
    let (p1, p2) = (adaptor(&s1).plane().unwrap(), adaptor(&s2).plane().unwrap());
    let spine = line((0., 0., 0.), (0., 1., 0.));
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_chamfer_pln_pln(
            &mut ds,
            &d,
            ChamfMode::ClassicChamfer,
            (&p1, &p2),
            (Reversed, Forward),
            (0.5, 0.8),
            &spine,
            0.,
            Forward
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Plane);
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_ch_asym_pln_pln(
            &mut ds,
            &d,
            (&p1, &p2),
            (Reversed, Forward),
            (0.5, 0.8),
            &spine,
            0.,
            Forward,
            true
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Plane);
}

#[test]
fn a_plane_and_a_cylinder_along_a_line() {
    let (s1, s2) = (
        plane_z(),
        cylinder((0., 0., 2.), (1., 0., 0.), (0., 1., 0.)),
    );
    let (p, c) = (
        adaptor(&s1).plane().unwrap(),
        adaptor(&s2).cylinder().unwrap(),
    );
    let spine = line((0., 5f64.sqrt(), 0.), (1., 0., 0.));
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_fillet_pln_cyl_line(
            &mut ds,
            &d,
            (&p, &c),
            (0., 2. * PI),
            (Forward, Forward),
            0.5,
            &spine,
            0.,
            Forward,
            true
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Cylinder);
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_chamfer_pln_cyl_line(
            &mut ds,
            &d,
            ChamfMode::ClassicChamfer,
            (&p, &c),
            (Forward, Forward),
            (0.5, 0.7),
            &spine,
            0.,
            Forward,
            true
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Plane);
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_ch_asym_pln_cyl_line(
            &mut ds,
            &d,
            (&p, &c),
            (Forward, Forward),
            (0.5, 0.7),
            &spine,
            0.,
            Forward,
            true,
            true
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Plane);
}

#[test]
fn a_plane_and_a_cylinder_along_a_circle() {
    let (s1, s2) = (
        plane_z(),
        cylinder((0., 0., 0.), (0., 0., 1.), (1., 0., 0.)),
    );
    let (p, c) = (
        adaptor(&s1).plane().unwrap(),
        adaptor(&s2).cylinder().unwrap(),
    );
    let spine = circle_z();
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_fillet_pln_cyl_circle(
            &mut ds,
            &d,
            (&p, &c),
            (0., 2. * PI),
            (Forward, Forward),
            0.5,
            &spine,
            0.,
            Forward,
            true
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Torus);
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_chamfer_pln_cyl_circle(
            &mut ds,
            &d,
            ChamfMode::ClassicChamfer,
            (&p, &c),
            (0., 2. * PI),
            (Forward, Forward),
            (0.5, 0.8),
            &spine,
            0.,
            Forward,
            true
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Cone);
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_ch_asym_pln_cyl_circle(
            &mut ds,
            &d,
            (&p, &c),
            (0., 2. * PI),
            (Forward, Forward),
            (0.5, 0.8),
            &spine,
            0.,
            Forward,
            true,
            false
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Cone);
}

#[test]
fn a_plane_and_a_cone_along_a_circle() {
    let (s1, s2) = (plane_z(), cone());
    let (p, c) = (adaptor(&s1).plane().unwrap(), adaptor(&s2).cone().unwrap());
    let spine = circle_z();
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_fillet_pln_con(
            &mut ds,
            &d,
            (&p, &c),
            (0., 2. * PI),
            (Forward, Forward),
            0.5,
            &spine,
            0.,
            Forward,
            true
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Torus);
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_chamfer_pln_con(
            &mut ds,
            &d,
            ChamfMode::ClassicChamfer,
            (&p, &c),
            (0., 2. * PI),
            (Forward, Forward),
            (0.5, 0.6),
            &spine,
            0.,
            Forward,
            true
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Cone);
    let (mut ds, d) = data();
    assert!(
        chfikpart::make_ch_asym_pln_con(
            &mut ds,
            &d,
            (&p, &c),
            (0., 2. * PI),
            (Forward, Forward),
            (0.5, 0.6),
            &spine,
            0.,
            Forward,
            true,
            true
        )
        .unwrap()
    );
    assert_eq!(check(&ds, &d, [&s1, &s2]), SurfaceType::Cone);
}

#[test]
fn the_corners_are_a_torus_a_sphere_and_a_joint() {
    let (g1, g2) = (
        plane_z(),
        cylinder((0., 0., 0.), (0., 0., 1.), (1., 0., 0.)),
    );
    let s1: Handle<dyn Adaptor3dSurface> = Handle::new(adaptor(&g1)).cast();
    let s2: Handle<dyn Adaptor3dSurface> = Handle::new(adaptor(&g2)).cast();
    let pts = [
        Point2::new(2.5, 0.5),
        Point2::new(0.5, 2.5),
        Point2::new(0.2, 0.1),
        Point2::new(1.2, 0.1),
    ];
    let (mut ds, d) = data();
    assert!(
        ChFiKPartComputeData::compute_corner_torus(
            &mut ds,
            &d,
            (&s1, &s2),
            (Forward, Forward, Forward),
            (0.5, 1.0),
            pts
        )
        .unwrap()
    );
    assert_eq!(blend_type(&ds, &d), SurfaceType::Torus);
    let (mut ds, d) = data();
    assert!(
        ChFiKPartComputeData::compute_corner_sphere(
            &mut ds,
            &d,
            (&s1, &s2),
            (Forward, Forward),
            1.0,
            (pts[0], pts[1], pts[2])
        )
        .unwrap()
    );
    assert_eq!(blend_type(&ds, &d), SurfaceType::Sphere);
    let gy = plane((0., 1., 0.), (0., 0., 1.));
    let gx = plane((1., 0., 0.), (0., 1., 0.));
    let sy: Handle<dyn Adaptor3dSurface> = Handle::new(adaptor(&gy)).cast();
    let sx: Handle<dyn Adaptor3dSurface> = Handle::new(adaptor(&gx)).cast();
    let (mut ds, d) = data();
    assert!(
        ChFiKPartComputeData::compute_corner_rotule(
            &mut ds,
            &d,
            (&s1, &sx, &sy),
            (Forward, Forward, Forward, Forward),
            0.5
        )
        .unwrap()
    );
    assert_eq!(blend_type(&ds, &d), SurfaceType::Torus);
}

#[test]
fn the_indices_of_the_geometry_grow() {
    let mut ds = TopOpeBRepDSDataStructure::new();
    assert_eq!(chfikpart::index_surface_in_ds(&plane_z(), &mut ds), 1);
    assert_eq!(chfikpart::index_surface_in_ds(&cone(), &mut ds), 2);
    let c: Handle<dyn rustycad_tkg3d::geom::GeomCurve> = Handle::default();
    assert_eq!(chfikpart::index_curve_in_ds(&c, &mut ds), 1);
}
