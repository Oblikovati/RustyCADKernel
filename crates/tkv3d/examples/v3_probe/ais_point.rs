//! Native point lifecycles, coordinate adapter, sensitive owners and presentation buffers.
use super::{
    common::{Reader, attempt, end, flag, int, pnt, real},
    sensitive_display::display_driver,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::*, std_prs::*, v3d::V3dViewer,
};
fn color(value: Color) {
    real(value.red());
    real(value.green());
    real(value.blue());
}
fn marker(value: i32) -> AspectTypeOfMarker {
    use AspectTypeOfMarker::*;
    [
        Empty,
        Point,
        Plus,
        Star,
        X,
        O,
        OPoint,
        OPlus,
        OStar,
        OX,
        Ring1,
        Ring2,
        Ring3,
        Ball,
        Userdefined,
    ][(value + 1) as usize]
}
fn point_aspect(drawer: &Prs3dDrawer) {
    flag(drawer.has_own_point_aspect());
    let aspect = drawer.point_aspect();
    flag(!aspect.is_null());
    if aspect.is_null() {
        return;
    }
    let style = aspect.aspect();
    color(style.interior_color());
    int(style.marker_kind() as i32);
    real(f64::from(style.scale()));
}
fn state(point: &Handle<AisPoint>) {
    int(point.signature());
    int(point.kind_of_interactive() as i32);
    flag(point.has_color());
    flag(point.has_width());
    flag(point.has_marker());
    real(point.width());
    color(point.color());
    pnt(point.component().pnt());
    point_aspect(&point.attributes());
    for drawer in [
        point.hilight_attributes(),
        point.dynamic_hilight_attributes(),
    ] {
        int(drawer.display_mode());
        int(drawer.z_layer());
        color(drawer.color());
        point_aspect(&drawer);
    }
    for mode in [-99, -1, 0, 1, 3] {
        flag(point.accept_display_mode(mode));
    }
}
fn select(point: &Handle<AisPoint>, mode: i32) -> Result<(), Failure> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    point.compute_selection(&selection, mode)?;
    int(selection.entities().len() as i32);
    for entity in selection.entities() {
        let sensitive = entity.base_sensitive().cast::<Select3DSensitivePoint>();
        pnt(sensitive.point());
        int(sensitive.owner_id().priority());
        flag(sensitive.owner_id().selectable().object_address() == point.object_address());
    }
    Ok(())
}
fn setup_drawer(point: &AisPoint, link: i32) {
    if link == 0 {
        point.attributes().setup_own_defaults().unwrap();
        return;
    }
    if link == 2 {
        return;
    }
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.point_aspect().set_scale(2.5).unwrap();
    drawer
        .point_aspect()
        .set_type_of_marker(AspectTypeOfMarker::O);
    drawer
        .point_aspect()
        .set_color(Color::named(NamedColor::GREEN));
    point.attributes().link_to(&drawer);
}
fn present(
    point: &Handle<AisPoint>,
    manager: &Handle<PrsMgrPresentationManager>,
    mode: i32,
) -> Result<(), Failure> {
    if mode == 0 && point.attributes().point_aspect().is_null() {
        return Ok(());
    }
    if manager.has_presentation(&point.cast(), mode) {
        manager.update(&point.cast(), mode)?;
    } else {
        manager.presentation(&point.cast(), mode, true, &Handle::default())?;
    }
    Ok(())
}
fn mutate(
    point: &Handle<AisPoint>,
    op: i32,
    width: f64,
    kind: AspectTypeOfMarker,
) -> Result<(), Failure> {
    match op {
        0 => point.set_color(Color::named(NamedColor::RED)),
        1 => point.unset_color(),
        2 => point.set_width(width),
        3 => point.unset_width(),
        4 => point.set_marker(kind),
        5 => point.unset_marker(),
        6 => {
            point.set_component(&Handle::new(GeomCartesianPoint::new(4., 5., 6.)).cast());
            Ok(())
        }
        7 => {
            point
                .component()
                .cast::<GeomCartesianPoint>()
                .set_coord(-2., 3., 4.);
            Ok(())
        }
        _ => {
            pnt(BRepTool::pnt(&point.vertex()?)?);
            Ok(())
        }
    }
}
pub fn ais_point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let link = r.integer();
    let width = r.real();
    let kind = marker(r.integer());
    let point = AisPoint::new(&Handle::new(GeomCartesianPoint::new(1., 2., 3.)).cast()).unwrap();
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    setup_drawer(&point, link);
    state(&point);
    let count = r.integer();
    for _ in 0..count {
        let op = r.integer();
        attempt(|| match op {
            8 => present(&point, &manager, 0),
            9 => present(&point, &manager, -99),
            10 => present(&point, &manager, 1),
            11 => select(&point, 0),
            12 => select(&point, -99),
            13 => select(&point, 7),
            _ => mutate(&point, op, width, kind),
        });
        state(&point);
    }
    let events = trace.read();
    int(events.events.len() as i32);
    for value in &events.events {
        real(*value);
    }
    end();
}
pub fn standard_point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let point = Handle::new(GeomCartesianPoint::new(r.real(), r.real(), r.real())).cast();
    let (x, y, z) = StdPrsToolPoint::coord(&point);
    real(x);
    real(y);
    real(z);
    flag(StdPrsPoint::matches(
        &point,
        r.real(),
        r.real(),
        r.real(),
        r.real(),
    ));
    end();
}
