//! Plane frame mutation, child selection owners and labeled presentation native oracle.
use super::{
    ais_line::fixture,
    common::{Reader, attempt, end, flag, int, pnt, real},
    sensitive_display::display_driver,
};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
    tcollection::AsciiString,
    units_api::SystemUnits,
};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, select3d::*, v3d::V3dViewer};
fn plane(kind: i32) -> Handle<GeomPlane> {
    let mut frame = Axis3::new(
        Point3::new(1., 2., 3.),
        Direction3::new(0., 0., 1.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    if kind == 1 {
        frame.y_reverse();
    }
    if kind == 2 {
        frame = Axis3::new(
            Point3::new(-2., 4., 6.),
            Direction3::new(1., 2., 3.).unwrap(),
            Direction3::new(2., -1., 0.).unwrap(),
        )
        .unwrap();
    }
    Handle::new(GeomPlane::new(frame))
}
fn color(value: Color) {
    real(value.red());
    real(value.green());
    real(value.blue());
}
fn child_state(child: &Handle<dyn SelectMgrSelectableObject>) {
    let ais = child.cast::<dyn AisInteractiveObject>();
    int(ais.signature());
    if ais.signature() == 1 {
        pnt(ais.cast::<AisPoint>().component().pnt());
    } else {
        let axis = ais.cast::<AisLine>().line().position();
        pnt(axis.location());
        let d = axis.direction();
        real(d.x());
        real(d.y());
        real(d.z());
    }
}
fn state(object: &AisPlaneTrihedron) -> Result<(), Failure> {
    int(object.signature());
    int(object.kind_of_interactive() as i32);
    flag(object.is_infinite());
    flag(object.has_color());
    color(object.color());
    real(object.get_length());
    let datum = object.attributes().datum_aspect();
    int(datum.datum_axes() as i32);
    for part in [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ] {
        real(datum.axis_length(part));
        color(datum.line_aspect(part).aspect().interior_color());
    }
    for mode in [-1, 0, 1, 2, 3] {
        flag(object.accept_display_mode(mode));
    }
    child_state(&object.position()?.cast());
    child_state(&object.x_axis()?.cast());
    child_state(&object.y_axis()?.cast());
    flag(object.position()? == object.position()?);
    flag(object.x_axis()? == object.x_axis()?);
    Ok(())
}
fn select(object: &AisPlaneTrihedron, mode: i32) -> Result<(), Failure> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    if !(0..=2).contains(&mode) {
        object.compute_selection(&selection, 0)?;
    }
    object.compute_selection(&selection, mode)?;
    int(selection.entities().len() as i32);
    for item in selection.entities() {
        let sensitive = item.base_sensitive();
        let owner = sensitive.owner_id();
        int(owner.priority());
        flag(
            owner.selectable().object_address()
                == object.presentable_base().this().object_address(),
        );
        if mode == 1 {
            pnt(sensitive.cast::<Select3DSensitivePoint>().point());
        } else {
            let segment = sensitive.cast::<Select3DSensitiveSegment>();
            pnt(segment.start_point());
            pnt(segment.end_point());
        }
        if mode == 1 || mode == 2 {
            child_state(&owner.selectable());
        }
    }
    Ok(())
}
fn operation(
    object: &Handle<AisPlaneTrihedron>,
    manager: &Handle<PrsMgrPresentationManager>,
    op: i32,
    length: f64,
) -> Result<(), Failure> {
    match op {
        0..=2 => select(object, op),
        3 => select(object, -1),
        4 => select(object, 99),
        5 => {
            object.set_length(length);
            Ok(())
        }
        6 => {
            object.set_component(&plane(2));
            Ok(())
        }
        7 => {
            let mut p = object.component().pln();
            p.set_location(Point3::new(7., 8., 9.));
            object.component().set_pln(p);
            Ok(())
        }
        8 => object.set_color(Color::named(NamedColor::RED)),
        9 => {
            object.set_x_label(&AsciiString::from_raw(b"U\0tail"));
            object.set_y_label(&AsciiString::from("V"));
            Ok(())
        }
        10 => {
            object.set_x_label(&AsciiString::default());
            object.set_y_label(&AsciiString::default());
            Ok(())
        }
        11 => {
            object
                .attributes()
                .datum_aspect()
                .set_axis_length(2., 3., 4.);
            object
                .attributes()
                .datum_aspect()
                .line_aspect(Prs3dDatumParts::YAxis)
                .set_color(Color::named(NamedColor::GREEN));
            Ok(())
        }
        _ => {
            if manager.has_presentation(&object.cast(), 0) {
                manager.update(&object.cast(), 0)?;
            } else {
                manager.presentation(&object.cast(), 0, true, &Handle::default())?;
            }
            Ok(())
        }
    }
}
pub fn plane_trihedron_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, system, length, count) = (r.integer(), r.integer(), r.real(), r.integer());
    let units = fixture::units();
    units
        .write()
        .set_local_system(if system == 0 {
            SystemUnits::Si
        } else {
            SystemUnits::Mdtv
        })
        .unwrap();
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    attempt(|| {
        let object = AisPlaneTrihedron::new(&plane(kind), units)?;
        let linked = Handle::new(Prs3dDrawer::new());
        linked.setup_own_defaults()?;
        object.attributes().link_to(&linked);
        state(&object)?;
        for _ in 0..count {
            let op = r.integer();
            attempt(|| operation(&object, &manager, op, length));
            state(&object)?;
        }
        let trace = trace.read();
        int(trace.events.len() as i32);
        for v in &trace.events {
            real(*v);
        }
        int(trace.texts.len() as i32);
        for text in &trace.texts {
            real(f64::from(text.height()));
            pnt(text.position());
            int(text.text().to_c_string()[0] as i32);
        }
        Ok(())
    });
    end();
}
