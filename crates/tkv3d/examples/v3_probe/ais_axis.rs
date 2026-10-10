//! Axis constructors, cached selection and appearance compared with the pinned native class.
use super::{
    ais_line::fixture,
    common::{Reader, attempt, end, flag, int, pnt, real},
    sensitive_display::display_driver,
};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, SharedState},
    units_api::{SystemUnits, UnitsApi},
};
use rustycad_tkg3d::geom::{GeomAxis1Placement, GeomAxis2Placement};
use rustycad_tkmath::gp::{Axis1, Axis2, Point3};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, select3d::*, v3d::V3dViewer};
use std::sync::Arc;
fn color(value: Color) {
    real(value.red());
    real(value.green());
    real(value.blue());
}
fn state(axis: &AisAxis) {
    int(axis.signature());
    int(axis.kind_of_interactive() as i32);
    int(axis.type_of_axis() as i32);
    flag(axis.is_xyz_axis());
    flag(axis.is_infinite());
    flag(axis.has_color());
    flag(axis.has_width());
    real(axis.width());
    color(axis.color());
    flag(!axis.axis2_placement().is_null());
    let component = axis.component().position();
    pnt(component.location());
    let d = component.direction();
    real(d.x());
    real(d.y());
    real(d.z());
    let drawer = axis.attributes();
    flag(drawer.has_own_line_aspect());
    let line = drawer.line_aspect().aspect();
    color(line.interior_color());
    real(f64::from(line.width()));
    int(line.line_type() as i32);
    for part in [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ] {
        let datum = drawer.datum_aspect();
        real(datum.axis_length(part));
        let style = datum.line_aspect(part).aspect();
        color(style.interior_color());
        real(f64::from(style.width()));
    }
    flag(drawer.datum_aspect().to_draw_labels());
    for mode in [-1, 0, 1, 3] {
        flag(axis.accept_display_mode(mode));
    }
}
fn select(axis: &AisAxis, mode: i32) -> Result<(), Failure> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    axis.compute_selection(&selection, mode)?;
    int(selection.entities().len() as i32);
    for item in selection.entities() {
        let segment = item.base_sensitive().cast::<Select3DSensitiveSegment>();
        pnt(segment.start_point());
        pnt(segment.end_point());
        int(segment.owner_id().priority());
        flag(
            segment.owner_id().selectable().object_address()
                == axis.presentable_base().this().object_address(),
        );
    }
    Ok(())
}
fn create(
    kind: i32,
    length: f64,
    units: Arc<SharedState<UnitsApi>>,
) -> Result<Handle<AisAxis>, Failure> {
    match kind {
        0 => AisAxis::new(&fixture::component(), units),
        4 => AisAxis::from_axis1(
            &Handle::new(GeomAxis1Placement::new(Axis1::default())),
            units,
        ),
        5 => AisAxis::from_ray(&Axis1::default(), length, units),
        _ => AisAxis::from_axis2(&frame(), axis_kind(kind), units),
    }
}
fn frame() -> Handle<GeomAxis2Placement> {
    Handle::new(GeomAxis2Placement::new(Axis2::default()))
}
fn axis_kind(kind: i32) -> AisTypeOfAxis {
    [
        AisTypeOfAxis::Unknown,
        AisTypeOfAxis::XAxis,
        AisTypeOfAxis::YAxis,
        AisTypeOfAxis::ZAxis,
    ][kind as usize]
}
fn operation(
    axis: &Handle<AisAxis>,
    units: &Arc<SharedState<UnitsApi>>,
    manager: &Handle<PrsMgrPresentationManager>,
    op: i32,
    width: f64,
) -> Result<(), Failure> {
    match op {
        0 => axis.set_color(Color::named(NamedColor::BLUE)),
        1 => axis.unset_color(),
        2 => axis.set_width(width),
        3 => axis.unset_width(),
        4 => axis.set_component(&fixture::component()),
        5 => axis.set_axis1_placement(&Handle::new(GeomAxis1Placement::new(Axis1::default()))),
        6..=8 => axis.set_axis2_placement(&frame(), axis_kind(op - 5)),
        9 => {
            axis.set_type_of_axis(AisTypeOfAxis::Unknown);
            Ok(())
        }
        10 => select(axis, 0),
        11 => select(axis, 3),
        12 => {
            let old = units.read().local_system();
            units.write().set_local_system(if old == SystemUnits::Si {
                SystemUnits::Mdtv
            } else {
                SystemUnits::Si
            })
        }
        13 => {
            if manager.has_presentation(&axis.cast(), 0) {
                manager.update(&axis.cast(), 0)?;
            } else {
                manager.presentation(&axis.cast(), 0, true, &Handle::default())?;
            }
            Ok(())
        }
        14 => {
            axis.attributes().datum_aspect().set_draw_labels(false);
            Ok(())
        }
        15 => axis.set_display_aspect(&Handle::new(Prs3dLineAspect::new(
            Color::named(NamedColor::GREEN),
            AspectTypeOfLine::Dash,
            2.,
        )?)),
        17 => axis.set_axis2_placement(&frame(), AisTypeOfAxis::Unknown),
        _ => {
            axis.component().set_location(Point3::new(6., 7., 8.));
            Ok(())
        }
    }
}
pub fn ais_axis_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, system, length, width, count) =
        (r.integer(), r.integer(), r.real(), r.real(), r.integer());
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
        let axis = create(kind, length, units.clone())?;
        let linked = Handle::new(Prs3dDrawer::new());
        linked.setup_own_defaults()?;
        axis.attributes().link_to(&linked);
        axis.attributes().set_maximal_parameter_value(12.);
        state(&axis);
        for _ in 0..count {
            let op = r.integer();
            attempt(|| operation(&axis, &units, &manager, op, width));
            state(&axis);
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

pub fn xyz_axis_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (styled, label, length, angle) = (r.flag(), r.flag(), r.real(), r.real());
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    attempt(|| {
        let line = Handle::new(Prs3dLineAspect::new(
            Color::named(NamedColor::RED),
            AspectTypeOfLine::Solid,
            2.,
        )?);
        let arrow = Handle::new(Prs3dArrowAspect::new()?);
        arrow.set_angle(angle)?;
        arrow.set_color(Color::named(NamedColor::BLUE));
        let text = Handle::new(Prs3dTextAspect::new());
        text.set_color(Color::named(NamedColor::GREEN));
        let direction = rustycad_tkmath::gp::Direction3::new(1., 2., 3.)?;
        let label = if label { "X" } else { "" };
        let last = Point3::from(direction.xyz() * length);
        if styled {
            rustycad_tkv3d::dsg_prs::DsgPrsXYZAxisPresentation::add_with_aspects(
                &prs.cast(),
                &line,
                &arrow,
                &text,
                &direction,
                length,
                label,
                &Point3::default(),
                &last,
            )?;
        } else {
            rustycad_tkv3d::dsg_prs::DsgPrsXYZAxisPresentation::add(
                &prs.cast(),
                &line,
                &direction,
                length,
                label,
                &Point3::default(),
                &last,
            )?;
        }
        Ok(())
    });
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
    end();
}
