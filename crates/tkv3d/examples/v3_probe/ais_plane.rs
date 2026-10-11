//! Exact AIS_Plane state, topology and callback comparisons against pinned native OCCT.
use super::{
    common::{Reader, attempt, end, flag, int, pnt, real},
    sensitive_display::display_driver,
};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomAxis2Placement, GeomPlane};
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point3};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, select3d::*, v3d::V3dViewer};
fn geometry(kind: i32) -> Handle<GeomPlane> {
    let mut frame = Axis3::new(Point3::new(1., 2., 3.), Direction3::Z, Direction3::X).unwrap();
    if kind == 1 {
        frame.y_reverse();
    }
    if kind == 2 {
        frame = Axis3::new(
            Point3::new(-2., 4., 6.),
            Direction3::new(0., 0.6, 0.8).unwrap(),
            Direction3::X,
        )
        .unwrap();
    }
    Handle::new(GeomPlane::new(frame))
}
fn axis() -> Handle<GeomAxis2Placement> {
    Handle::new(GeomAxis2Placement::new(
        Axis2::with_x_direction(
            Point3::new(2., 3., 4.),
            Direction3::new(0., 0.6, 0.8).unwrap(),
            Direction3::X,
        )
        .unwrap(),
    ))
}
fn construct(ctor: i32, kind: i32) -> Result<Handle<AisPlane>, Failure> {
    let plane = geometry(kind);
    let c = Point3::new(2., 3., 4.);
    match ctor {
        0 => AisPlane::new(&plane, false),
        1 => AisPlane::with_center(&plane, c, true),
        2 => AisPlane::with_bounds(
            &plane,
            c,
            Point3::new(-1., 0., 2.),
            Point3::new(99., 99., 99.),
            true,
        ),
        _ => AisPlane::new(&plane, false),
    }
}
fn color(value: Color) {
    real(value.red());
    real(value.green());
    real(value.blue());
}
fn snapshot(obj: &AisPlane) -> Result<(), Failure> {
    int(obj.signature());
    int(obj.kind_of_interactive() as i32);
    int(obj.type_of_plane() as i32);
    flag(obj.is_xyz_plane());
    flag(obj.current_mode());
    pnt(obj.center());
    flag(obj.axis2_placement().is_null());
    int(obj.type_of_sensitivity() as i32);
    snapshot_component(obj)?;
    snapshot_explicit_bounds(obj);
    for mode in [-1, 0, 1, 2, 99] {
        flag(obj.accept_display_mode(mode));
    }
    Ok(())
}
fn snapshot_component(obj: &AisPlane) -> Result<(), Failure> {
    let plane = obj.component();
    flag(!plane.is_null());
    if plane.is_null() {
        return Ok(());
    }
    pnt(plane.pln().location());
    for v in plane.coefficients()? {
        real(v);
    }
    Ok(())
}
fn snapshot_explicit_bounds(obj: &AisPlane) {
    let mut component = Handle::default();
    let mut c = Point3::new(9., 8., 7.);
    let mut a = c;
    let mut b = c;
    flag(obj.plane_attributes(&mut component, &mut c, &mut a, &mut b));
    flag(component.is_null());
    pnt(c);
    pnt(a);
    pnt(b);
}
fn appearance(obj: &AisPlane) {
    flag(obj.has_own_size());
    flag(obj.has_color());
    flag(obj.has_minimum_size());
    flag(obj.is_infinite());
    let (mut x, mut y) = (0., 0.);
    flag(obj.size(&mut x, &mut y));
    real(x);
    real(y);
    let d = obj.attributes();
    flag(d.has_own_plane_aspect());
    flag(d.has_own_datum_aspect());
    snapshot_plane_aspects(&d);
    color(d.color());
    color(d.shading_aspect().color(AspectTypeOfFacingModel::BothSide));
    real(
        d.shading_aspect()
            .transparency(AspectTypeOfFacingModel::BothSide),
    );
}
fn snapshot_plane_aspects(d: &Handle<Prs3dDrawer>) {
    let pa = d.plane_aspect();
    let da = d.datum_aspect();
    real(pa.iso_distance());
    real(f64::from(pa.edges_aspect().aspect().width()));
    color(pa.edges_aspect().aspect().interior_color());
    for part in [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ] {
        real(da.axis_length(part));
        color(da.line_aspect(part).aspect().interior_color());
    }
}
fn selection(obj: &AisPlane, mode: i32) -> Result<(), Failure> {
    let sel = Handle::new(SelectMgrSelection::new(mode));
    obj.compute_selection(&sel, mode)?;
    obj.compute_selection(&sel, mode)?;
    int(sel.entities().len() as i32);
    let s = sel.entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitiveTriangulation>();
    int(s.owner_id().priority());
    flag(
        s.owner_id().selectable().object_address()
            == obj.presentable_base().this().object_address(),
    );
    int(s.nb_sub_elements());
    snapshot_mesh(&s.triangulation())
}
fn snapshot_mesh(poly: &Handle<rustycad_tkmath::poly::Triangulation>) -> Result<(), Failure> {
    int(poly.nb_nodes());
    int(poly.nb_triangles());
    for i in 1..=poly.nb_nodes() {
        pnt(poly.node(i)?);
    }
    for i in 1..=poly.nb_triangles() {
        let (a, b, c) = poly.triangle(i)?.get();
        int(a);
        int(b);
        int(c);
    }
    Ok(())
}
fn operation(
    obj: &AisPlane,
    prs: &Handle<Graphic3dStructure>,
    trace: &display_driver::DisplayTrace,
    op: i32,
) -> Result<(), Failure> {
    if op <= 3 {
        return capture_compute(obj, prs, trace, op);
    }
    if op <= 11 {
        return appearance_operation(obj, op);
    }
    if op <= 18 {
        return geometry_operation(obj, op);
    }
    match op {
        19 => selection(obj, -1)?,
        20 => selection(obj, 0)?,
        21 => selection(obj, 99)?,
        22 => obj.set_infinite_state(true),
        23 => obj.set_context(&Handle::default()),
        _ => {}
    }
    Ok(())
}
fn capture_compute(
    obj: &AisPlane,
    prs: &Handle<Graphic3dStructure>,
    trace: &display_driver::DisplayTrace,
    op: i32,
) -> Result<(), Failure> {
    trace.write().events.clear();
    obj.compute(
        &Handle::default(),
        &prs.cast(),
        if op == 3 { 99 } else { op },
    )?;
    let events = trace.read().events.clone();
    int(events.len() as i32);
    for v in events {
        real(v);
    }
    flag(prs.is_infinite());
    Ok(())
}
fn appearance_operation(obj: &AisPlane, op: i32) -> Result<(), Failure> {
    match op {
        4 => obj.set_size(8.)?,
        5 => obj.set_size_xy(4., 6.)?,
        6 => obj.unset_size()?,
        7 => obj.set_color(Color::named(NamedColor::RED))?,
        8 => obj.unset_color()?,
        9 => obj.set_minimum_size(2.)?,
        10 => obj.set_minimum_size(0.)?,
        11 => obj.unset_minimum_size(),
        _ => {}
    }
    Ok(())
}
fn geometry_operation(obj: &AisPlane, op: i32) -> Result<(), Failure> {
    match op {
        12 => obj.set_center(Point3::new(7., 8., 9.)),
        13 => obj.set_component(&geometry(2)),
        14 => obj.set_axis2_placement(&axis(), AisTypeOfPlane::YZPlane)?,
        15 => obj.set_plane_attributes(
            &geometry(1),
            Point3::new(2., 3., 4.),
            Point3::new(2., 1., 0.),
            Point3::new(100., 100., 100.),
        ),
        16 => obj.set_current_mode(!obj.current_mode()),
        17 => obj.set_type_of_sensitivity(Select3DTypeOfSensitivity::Interior),
        18 => obj.set_type_of_sensitivity(Select3DTypeOfSensitivity::Boundary),
        _ => {}
    }
    Ok(())
}
fn setup_probe_object(ctor: i32, kind: i32, linked: i32) -> Result<Handle<AisPlane>, Failure> {
    let obj = construct(ctor, kind)?;
    setup_probe_drawer(&obj, linked)?;
    if ctor >= 3 {
        obj.set_axis2_placement(&axis(), coordinate_kind(ctor - 3))?;
    }
    snapshot(&obj)?;
    obj.set_size_xy(4., 6.)?;
    appearance(&obj);
    Ok(obj)
}
fn coordinate_kind(kind: i32) -> AisTypeOfPlane {
    [
        AisTypeOfPlane::Unknown,
        AisTypeOfPlane::XYPlane,
        AisTypeOfPlane::XZPlane,
        AisTypeOfPlane::YZPlane,
    ][kind as usize]
}
fn setup_probe_drawer(obj: &AisPlane, linked: i32) -> Result<(), Failure> {
    if linked == 0 {
        obj.attributes()
            .set_plane_aspect(Handle::new(Prs3dPlaneAspect::new()?));
        obj.attributes()
            .set_datum_aspect(Handle::new(Prs3dDatumAspect::new()?));
        return Ok(());
    }
    let d = Handle::new(Prs3dDrawer::new());
    d.setup_own_defaults()?;
    d.plane_aspect().set_plane_length(11., 13.);
    d.plane_aspect().set_iso_distance(7.);
    d.datum_aspect().set_axis_length(2., 3., 5.);
    obj.attributes().link_to(&d);
    Ok(())
}
/// Compare native plane operations; `plane_case(&[0.,0.,1.,0.])`.
pub fn plane_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (ctor, kind, linked, count) = (r.integer(), r.integer(), r.integer(), r.integer());
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    attempt(|| {
        let obj = setup_probe_object(ctor, kind, linked)?;
        for _ in 0..count {
            let op = r.integer();
            attempt(|| operation(&obj, &prs, &trace, op));
            snapshot(&obj)?;
            appearance(&obj);
        }
        Ok(())
    });
    end();
}
