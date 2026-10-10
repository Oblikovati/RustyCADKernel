//! Real AIS_Line mutations, presentations, sensitive owners and native exception state.
use super::{
    common::{Reader, attempt, end, flag, int, pnt, real},
    sensitive_display::display_driver,
};
#[path = "../../tests/support/ais_line_fixture.rs"]
mod fixture;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
    units_api::SystemUnits,
};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::*, v3d::V3dViewer,
};
fn color(value: Color) {
    real(value.red());
    real(value.green());
    real(value.blue());
}
fn state(line: &Handle<AisLine>) {
    int(line.signature());
    int(line.kind_of_interactive() as i32);
    flag(line.is_infinite());
    flag(line.has_color());
    flag(line.has_width());
    real(line.width());
    color(line.color());
    let drawer = line.attributes();
    flag(drawer.has_own_line_aspect());
    let aspect = drawer.line_aspect();
    flag(!aspect.is_null());
    if !aspect.is_null() {
        real(f64::from(aspect.aspect().width()));
        color(aspect.aspect().interior_color());
    }
    let component = line.line();
    flag(!component.is_null());
    if !component.is_null() {
        pnt(component.position().location());
        let d = component.position().direction();
        real(d.x());
        real(d.y());
        real(d.z());
    }
    let (a, b) = line.points();
    flag(!a.is_null());
    if !a.is_null() {
        pnt(a.pnt());
    }
    flag(!b.is_null());
    if !b.is_null() {
        pnt(b.pnt());
    }
}
fn select(line: &Handle<AisLine>, mode: i32) -> Result<(), Failure> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    line.compute_selection(&selection, mode)?;
    int(selection.entities().len() as i32);
    for sensitive in selection.entities() {
        let segment = sensitive
            .base_sensitive()
            .cast::<Select3DSensitiveSegment>();
        pnt(segment.start_point());
        pnt(segment.end_point());
        int(segment.owner_id().priority());
        flag(segment.owner_id().selectable().object_address() == line.object_address());
    }
    Ok(())
}
fn appearance(line: &Handle<AisLine>, op: i32, width: f64) -> Result<(), Failure> {
    let base = line.cast::<dyn PrsMgrPresentableObject>();
    match op {
        0 => base.set_color(Color::named(NamedColor::RED)),
        1 => base.unset_color(),
        2 => base.set_width(width),
        _ => base.unset_width(),
    }
}
pub fn ais_line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let segment = r.flag();
    let system = r.integer();
    let link = r.integer();
    let width = r.real();
    let units = fixture::units();
    units
        .write()
        .set_local_system(if system == 0 {
            SystemUnits::Si
        } else {
            SystemUnits::Mdtv
        })
        .unwrap();
    let line = fixture::object(segment, units.clone());
    let (a, b) = fixture::endpoints();
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    setup_drawer(&line, link);
    state(&line);
    let count = r.integer();
    for _ in 0..count {
        let op = r.integer();
        attempt(|| match op {
            0..=3 => appearance(&line, op, width),
            4 => {
                line.set_line(&fixture::component());
                Ok(())
            }
            5 => {
                line.set_points(&a, &b);
                Ok(())
            }
            6 => {
                b.cast::<GeomCartesianPoint>().set_coord(5., 1., 2.);
                Ok(())
            }
            7 => {
                let system = units.read().local_system();
                units
                    .write()
                    .set_local_system(if system == SystemUnits::Si {
                        SystemUnits::Mdtv
                    } else {
                        SystemUnits::Si
                    })
            }
            8 => {
                if !line.attributes().line_aspect().is_null() {
                    if manager.has_presentation(&line.cast(), 0) {
                        manager.update(&line.cast(), 0)?;
                    } else {
                        manager.presentation(&line.cast(), 0, true, &Handle::default())?;
                    }
                }
                Ok(())
            }
            9 => select(&line, 0),
            10 => select(&line, 1),
            _ => {
                line.set_points(&a, &a);
                Ok(())
            }
        });
        state(&line);
    }
    let events = trace.read();
    int(events.events.len() as i32);
    for value in &events.events {
        real(*value);
    }
    end();
}
fn setup_drawer(line: &AisLine, link: i32) {
    line.attributes().set_maximal_parameter_value(12.);
    if link == 0 {
        line.attributes().setup_own_defaults().unwrap();
        return;
    }
    if link == 2 {
        return;
    }
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.line_aspect().set_width(2.5).unwrap();
    drawer
        .line_aspect()
        .set_color(Color::named(NamedColor::GREEN));
    line.attributes().link_to(&drawer);
}
