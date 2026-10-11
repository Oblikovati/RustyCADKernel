//! Exact native trihedron appearance transitions and protected geometry snapshots.
use super::{
    array::array_state,
    common::{Reader, attempt, end, flag, int, pnt, real},
    sensitive_display::display_driver,
};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
    tcollection::ExtendedString,
};
use rustycad_tkg3d::geom::GeomAxis2Placement;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, select3d::*, v3d::V3dViewer};
fn frame(kind: i32) -> Handle<GeomAxis2Placement> {
    let axis = if kind == 0 {
        Axis2::with_x_direction(Point3::new(1., 2., 3.), Direction3::Z, Direction3::X).unwrap()
    } else {
        Axis2::with_x_direction(
            Point3::new(-2., 4., 6.),
            Direction3::new(1., 2., 3.).unwrap(),
            Direction3::new(2., -1., 0.).unwrap(),
        )
        .unwrap()
    };
    Handle::new(GeomAxis2Placement::new(axis))
}
fn color(value: Color) {
    for component in [value.red(), value.green(), value.blue()] {
        real(component);
    }
}
fn snapshot(object: &AisTrihedron) {
    int(object.signature());
    int(object.kind_of_interactive() as i32);
    flag(object.is_auto_hilight());
    int(object.datum_display_mode() as i32);
    flag(object.has_own_size());
    real(object.size());
    flag(object.has_color());
    flag(object.has_text_color());
    flag(object.has_arrow_color());
    flag(object.to_draw_arrows());
    color(object.text_color());
    color(object.arrow_color());
    flag(object.attributes().has_own_datum_aspect());
    for part in [
        Prs3dDatumParts::Origin,
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
        Prs3dDatumParts::XArrow,
        Prs3dDatumParts::YArrow,
        Prs3dDatumParts::ZArrow,
        Prs3dDatumParts::XOYAxis,
        Prs3dDatumParts::YOZAxis,
        Prs3dDatumParts::XOZAxis,
        Prs3dDatumParts::None,
    ] {
        let datum = object.attributes().datum_aspect();
        int(object.selection_priority(part));
        int(object.label(part).length());
        flag(datum.draw_datum_part(part));
        real(datum.axis_length(part));
        if part == Prs3dDatumParts::None {
            continue;
        }
        color(
            datum
                .shading_aspect(part)
                .color(AspectTypeOfFacingModel::BothSide),
        );
        if part != Prs3dDatumParts::Origin {
            color(datum.line_aspect(part).aspect().interior_color());
        }
    }
}
fn cached_arrays(object: &AisTrihedron) {
    let fields = object.trihedron_base().fields.read();
    for array in &fields.primitives {
        flag(array.is_null());
        if let Some(array) = array.get() {
            array_state(array);
        }
    }
}
fn selected(object: &AisTrihedron, mode: i32) -> Result<(), Failure> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    object.compute_selection(&selection, mode)?;
    int(selection.entities().len() as i32);
    let mut first: Handle<dyn SelectMgrEntityOwnerSource> = Handle::default();
    for item in selection.entities() {
        let sensitive = item.base_sensitive();
        let owner = sensitive.owner_id();
        if first.is_null() {
            first = owner.clone();
        }
        int(owner.priority());
        flag(owner == first);
        flag(
            owner.selectable().object_address()
                == object.presentable_base().this().object_address(),
        );
        let part_owner = owner.cast::<dyn AisTrihedronOwnerSource>();
        int(if part_owner.is_null() {
            -1
        } else {
            part_owner.datum_part() as i32
        });
        let segment = sensitive.cast::<Select3DSensitiveSegment>();
        let point = sensitive.cast::<Select3DSensitivePoint>();
        flag(!segment.is_null());
        flag(!point.is_null());
        if let Some(segment) = segment.get() {
            pnt(segment.start_point());
            pnt(segment.end_point());
        }
        if let Some(point) = point.get() {
            pnt(point.point());
        }
    }
    Ok(())
}
fn mutation(object: &AisTrihedron, op: i32, size: f64) -> Result<(), Failure> {
    let red = Color::named(NamedColor::RED);
    let blue = Color::named(NamedColor::BLUE);
    let green = Color::named(NamedColor::GREEN);
    match op {
        0 => object.set_size(size),
        1 => object.unset_size(),
        2 => object.set_color(red),
        3 => object.unset_color(),
        4 => object.set_text_color(blue),
        5 => object.set_text_color_part(Prs3dDatumParts::XAxis, green),
        6 => object.set_arrow_color(blue),
        7 => object.set_arrow_color_part(Prs3dDatumParts::XAxis, green),
        8 => object.set_origin_color(red),
        9 => {
            object.set_x_axis_color(red)?;
            object.set_y_axis_color(green)?;
            object.set_axis_color(blue)
        }
        10 => {
            object.set_datum_display_mode(Prs3dDatumMode::Shaded);
            Ok(())
        }
        11 => {
            object.set_datum_display_mode(Prs3dDatumMode::WireFrame);
            Ok(())
        }
        12 => object.set_draw_arrows(false),
        13 => object.set_draw_arrows(true),
        14 => {
            object.set_component(&frame(1));
            Ok(())
        }
        15 => {
            object.set_label(Prs3dDatumParts::XAxis, &ExtendedString::new("U"));
            object.set_selection_priority(Prs3dDatumParts::XAxis, 11);
            Ok(())
        }
        _ => Ok(()),
    }
}
pub fn trihedron_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (kind, count, size) = (reader.integer(), reader.integer(), reader.real());
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    attempt(|| {
        let object = AisTrihedron::new(&frame(kind))?;
        let linked = Handle::new(Prs3dDrawer::new());
        linked.setup_own_defaults()?;
        linked.datum_aspect().set_axis_length(2., 3., 4.);
        object.attributes().link_to(&linked);
        snapshot(&object);
        for _ in 0..count {
            let op = reader.integer();
            attempt(|| {
                if op == 16 {
                    prs.clear(true);
                    object.compute(&manager, &prs.cast(), 0)?;
                    cached_arrays(&object);
                    return Ok(());
                }
                if (20..=24).contains(&op) {
                    return selected(&object, op - 20);
                }
                mutation(&object, op, size)
            });
            snapshot(&object);
        }
        flag(prs.is_infinite());
        int(prs.groups().len() as i32);
        Ok(())
    });
    end();
}
