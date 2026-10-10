//! Live circle state, appearance, sensitivity snapshots and actual presentation callbacks.
use super::{
    common::{Reader, attempt, end, flag, int, pnt, real},
    sensitive_display::display_driver,
};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomCircle, GeomConic};
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::*, v3d::V3dViewer,
};
fn component(radius: f64) -> Handle<GeomCircle> {
    Handle::new(GeomCircle::new(Axis2::default(), radius).unwrap())
}
fn color(value: Color) {
    real(value.red());
    real(value.green());
    real(value.blue());
}
fn state(object: &AisCircle) {
    int(object.signature());
    int(object.kind_of_interactive() as i32);
    flag(object.is_infinite());
    flag(object.has_color());
    flag(object.has_width());
    real(object.width());
    color(object.color());
    let drawer = object.attributes();
    flag(drawer.has_own_line_aspect());
    let aspect = drawer.line_aspect();
    flag(!aspect.is_null());
    if !aspect.is_null() {
        real(f64::from(aspect.aspect().width()));
        color(aspect.aspect().interior_color());
    }
    let (first, last) = object.parameters();
    real(first);
    real(last);
    flag(object.is_filled_circle_sens());
    real(object.circle().radius());
    pnt(object.circle().position().location());
    real(drawer.deviation_coefficient());
    flag(drawer.has_own_deviation_coefficient());
    for mode in [-1, 0, 1, 3] {
        flag(object.accept_display_mode(mode));
    }
}
fn select(object: &AisCircle, mode: i32) -> Result<(), Failure> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    object.compute_selection(&selection, mode)?;
    int(selection.entities().len() as i32);
    for item in selection.entities() {
        let entity = item.base_sensitive();
        int(entity.owner_id().priority());
        int(entity.sensitivity_factor());
        flag(
            entity.owner_id().selectable().object_address()
                == object.presentable_base().this().object_address(),
        );
        let circle = entity.cast::<Select3DSensitiveCircle>();
        flag(!circle.is_null());
        if !circle.is_null() {
            real(circle.radius());
            pnt(circle.circle()?.location());
        } else {
            let poly = entity.cast::<Select3DSensitivePoly>();
            let points = poly.points_3d();
            int(points.len() as i32);
            for point in points {
                pnt(point);
            }
        }
    }
    Ok(())
}
fn operation(
    object: &Handle<AisCircle>,
    manager: &Handle<PrsMgrPresentationManager>,
    op: i32,
    width: f64,
) -> Result<(), Failure> {
    match op {
        0 => object.set_color(Color::named(NamedColor::RED)),
        1 => object.unset_color(),
        2 => object.set_width(width),
        3 => object.unset_width(),
        4 => {
            object.set_first_param(0.5);
            Ok(())
        }
        5 => {
            object.set_last_param(2.);
            Ok(())
        }
        6 => {
            object.set_circle(&component(2.));
            Ok(())
        }
        7 => object.circle().set_radius(4.),
        9 => select(object, 0),
        10 => select(object, 99),
        11 => {
            object.set_filled_circle_sens(!object.is_filled_circle_sens());
            Ok(())
        }
        12 => {
            object.circle().set_location(Point3::new(1., 2., 3.));
            Ok(())
        }
        _ => {
            if !object.attributes().line_aspect().is_null() {
                if manager.has_presentation(&object.cast(), 0) {
                    manager.update(&object.cast(), 0)?;
                } else {
                    manager.presentation(&object.cast(), 0, true, &Handle::default())?;
                }
            }
            Ok(())
        }
    }
}
fn setup(object: &AisCircle, link: i32) {
    if link == 0 {
        object.attributes().setup_own_defaults().unwrap();
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
    object.attributes().link_to(&drawer);
}
pub fn circle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (variant, filled, link) = (r.integer(), r.flag(), r.integer());
    let (first, last, width) = (r.real(), r.real(), r.real());
    let count = r.integer();
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let object = if variant == 0 {
        AisCircle::new(&component(3.))
    } else {
        AisCircle::with_parameters(&component(3.), first, last, filled)
    };
    setup(&object, link);
    state(&object);
    for _ in 0..count {
        let op = r.integer();
        attempt(|| operation(&object, &manager, op, width));
        state(&object);
    }
    let events = trace.read();
    int(events.events.len() as i32);
    for value in &events.events {
        real(*value);
    }
    end();
}
