//! Martensite trihedron ownership, geometry, selection cache and highlight contracts.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
    tcollection::ExtendedString,
};
use rustycad_tkg3d::geom::GeomAxis2Placement;
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, select3d::*, v3d::V3dViewer};
fn object() -> Handle<AisTrihedron> {
    let object =
        AisTrihedron::new(&Handle::new(GeomAxis2Placement::new(Axis2::default()))).unwrap();
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    object.attributes().link_to(&linked);
    object
}
fn compute(
    object: &AisTrihedron,
) -> (
    Handle<PrsMgrPresentationManager>,
    Handle<dyn Graphic3dStructureSource>,
) {
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    object.compute(&manager, &prs.cast(), 0).unwrap();
    (manager, prs.cast())
}
fn selection(object: &AisTrihedron, mode: i32) -> Handle<SelectMgrSelection> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    object.compute_selection(&selection, mode).unwrap();
    selection
}
#[test]
fn constructor_preserves_native_identity_labels_priorities_and_alias() {
    let object = object();
    assert_eq!(object.signature(), 3);
    assert_eq!(object.kind_of_interactive(), AisKindOfInteractive::Datum);
    assert!(!object.is_auto_hilight());
    assert_eq!(object.datum_display_mode(), Prs3dDatumMode::WireFrame);
    for mode in [-1, 0, 1, 2] {
        assert_eq!(object.accept_display_mode(mode), mode == 0);
    }
    for (part, priority) in [
        (Prs3dDatumParts::None, 5),
        (Prs3dDatumParts::Origin, 8),
        (Prs3dDatumParts::XAxis, 7),
        (Prs3dDatumParts::XOYAxis, 5),
        (Prs3dDatumParts::XArrow, 0),
    ] {
        assert_eq!(object.selection_priority(part), priority);
    }
    assert_eq!(
        object.label(Prs3dDatumParts::XAxis),
        ExtendedString::new("X")
    );
    object.set_label(Prs3dDatumParts::XAxis, &ExtendedString::new("U"));
    object.set_selection_priority(Prs3dDatumParts::XAxis, 11);
    assert_eq!(
        object.label(Prs3dDatumParts::XAxis),
        ExtendedString::new("U")
    );
    assert_eq!(object.selection_priority(Prs3dDatumParts::XAxis), 11);
    let placement = Handle::new(GeomAxis2Placement::new(Axis2::default()));
    object.set_component(&placement);
    assert_eq!(object.component(), placement);
}
#[test]
fn size_ownership_retains_native_uncolored_lengths_and_restores_linked_colored_lengths() {
    let object = object();
    object.set_size(17.).unwrap();
    object.unset_size().unwrap();
    assert!(!object.has_own_size());
    assert_eq!(object.size(), 17.);
    assert!(object.attributes().has_own_datum_aspect());
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    linked.datum_aspect().set_axis_length(2., 3., 4.);
    object.attributes().link_to(&linked);
    object.set_color(Color::named(NamedColor::RED)).unwrap();
    object.set_size(9.).unwrap();
    object.unset_size().unwrap();
    assert_eq!(object.size(), 2.);
    assert_eq!(
        object
            .attributes()
            .datum_aspect()
            .axis_length(Prs3dDatumParts::ZAxis),
        4.
    );
    object.unset_size().unwrap();
    assert_eq!(object.size(), 2.);
}
#[test]
fn text_and_arrow_flags_follow_native_overload_and_unset_quirks() {
    let object = object();
    let red = Color::named(NamedColor::RED);
    let blue = Color::named(NamedColor::BLUE);
    let default_arrow = object.arrow_color();
    object.set_text_color(red).unwrap();
    assert_eq!(object.text_color(), red);
    assert!(!object.has_text_color());
    object
        .set_text_color_part(Prs3dDatumParts::XAxis, blue)
        .unwrap();
    assert_eq!(object.text_color(), blue);
    object
        .set_arrow_color_part(Prs3dDatumParts::XAxis, red)
        .unwrap();
    assert!(object.has_arrow_color());
    assert_eq!(object.arrow_color(), default_arrow);
    assert_eq!(
        object.datum_part_color(Prs3dDatumParts::XArrow).unwrap(),
        red
    );
    object.set_arrow_color(blue).unwrap();
    assert_eq!(object.arrow_color(), blue);
    object.unset_color().unwrap();
    assert!(object.has_color());
    assert!(!object.has_arrow_color());
    assert_eq!(
        object.arrow_color(),
        Color::named(NamedColor::LIGHTSTEELBLUE4)
    );
    assert_eq!(object.text_color(), blue);
}
#[test]
fn color_aliases_origin_policy_and_invalid_parts_are_explicit() {
    let object = object();
    let red = Color::named(NamedColor::RED);
    let before = object
        .attributes()
        .datum_aspect()
        .shading_aspect(Prs3dDatumParts::Origin)
        .aspect()
        .interior_color();
    object.set_origin_color(red).unwrap();
    assert_eq!(
        object
            .attributes()
            .datum_aspect()
            .shading_aspect(Prs3dDatumParts::Origin)
            .aspect()
            .interior_color(),
        before
    );
    assert!(object.datum_part_color(Prs3dDatumParts::Origin).is_err());
    object.set_datum_display_mode(Prs3dDatumMode::Shaded);
    object.set_origin_color(red).unwrap();
    assert_eq!(
        object.datum_part_color(Prs3dDatumParts::Origin).unwrap(),
        red
    );
    object.set_x_axis_color(red).unwrap();
    object.set_y_axis_color(red).unwrap();
    object.set_axis_color(red).unwrap();
    for part in [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ] {
        assert_eq!(object.datum_part_color(part).unwrap(), red);
    }
    assert!(
        object
            .set_text_color_part(Prs3dDatumParts::Origin, red)
            .is_err()
    );
    assert!(
        object
            .set_datum_part_color(Prs3dDatumParts::None, red)
            .is_err()
    );
}
#[test]
fn wire_arrays_and_selection_share_whole_owner_but_cache_geometry() {
    let object = object();
    let uncomputed = Handle::new(SelectMgrSelection::new(2));
    assert!(object.compute_selection(&uncomputed, 2).is_err());
    object.set_size(4.).unwrap();
    let (_, prs) = compute(&object);
    assert!(prs.is_infinite());
    assert!(!object.is_infinite());
    let whole = selection(&object, 0);
    assert_eq!(whole.entities().len(), 3);
    let entities = whole.entities();
    assert_eq!(
        entities[0].base_sensitive().owner_id(),
        entities[2].base_sensitive().owner_id()
    );
    assert_eq!(entities[0].base_sensitive().owner_id().priority(), 5);
    let axes = selection(&object, 2);
    let first = axes.entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitiveSegment>();
    assert_eq!(first.end_point(), Point3::new(4., 0., 0.));
    assert_eq!(
        first
            .owner_id()
            .cast::<dyn AisTrihedronOwnerSource>()
            .datum_part(),
        Prs3dDatumParts::XAxis
    );
    object.set_size(8.).unwrap();
    let stale = selection(&object, 2).entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitiveSegment>();
    assert_eq!(stale.end_point(), Point3::new(4., 0., 0.));
    compute(&object);
    let fresh = selection(&object, 2).entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitiveSegment>();
    assert_eq!(fresh.end_point(), Point3::new(8., 0., 0.));
    object.compute_selection(&axes, -1).unwrap();
    assert_eq!(axes.entities().len(), 3);
}
#[test]
fn shaded_arrays_planes_and_arrows_use_native_builders() {
    let object = object();
    object.set_size(10.).unwrap();
    object.set_datum_display_mode(Prs3dDatumMode::Shaded);
    compute(&object);
    assert_eq!(selection(&object, 0).entities().len(), 4);
    let origin = selection(&object, 1);
    assert!(
        !origin.entities()[0]
            .base_sensitive()
            .cast::<Select3DSensitivePrimitiveArray>()
            .is_null()
    );
    assert_eq!(
        origin.entities()[0].base_sensitive().owner_id().priority(),
        8
    );
    let planes = selection(&object, 3);
    assert_eq!(planes.entities().len(), 3);
    for entity in planes.entities() {
        assert!(
            !entity
                .base_sensitive()
                .cast::<Select3DSensitiveTriangle>()
                .is_null()
        );
    }
    let xy = object.array_of_primitives(Prs3dDatumParts::XOYAxis);
    assert_eq!(xy.vertex_number(), 4);
    assert_eq!(xy.vertice(2).unwrap(), Point3::new(10., 0., 0.));
    assert_eq!(xy.vertice(3).unwrap(), Point3::new(0., 10., 0.));
    assert!(
        object.trihedron_base().fields.read().part_groups[Prs3dDatumParts::XArrow as usize]
            .is_null()
    );
    object.set_draw_arrows(false).unwrap();
    assert!(!object.to_draw_arrows());
    compute(&object);
    assert!(
        object
            .array_of_primitives(Prs3dDatumParts::XArrow)
            .is_null()
    );
    assert_eq!(selection(&object, 99).entities().len(), 0);
}
#[test]
fn dynamic_part_highlight_uses_cached_array_and_owner_dispatch() {
    let object = object();
    let (manager, _) = compute(&object);
    let owner = AisTrihedronOwner::new(&object.cast(), Prs3dDatumParts::XAxis, 7);
    let style = Handle::new(Prs3dDrawer::new());
    style.setup_own_defaults().unwrap();
    owner.hilight_with_color(&manager, &style, 99).unwrap();
    let highlight = object.get_hilight_presentation(&manager).unwrap();
    assert_eq!(highlight.groups().len(), 1);
    assert!(
        object
            .trihedron_base()
            .fields
            .read()
            .selected_parts
            .is_empty()
    );
    object.hilight_selected(&manager, &[owner.cast()]).unwrap();
    assert!(
        object
            .trihedron_base()
            .fields
            .read()
            .selected_parts
            .is_empty()
    );
    object.clear_selected();
    let detached = AisTrihedronOwner::new(&Handle::default(), Prs3dDatumParts::Origin, 8);
    assert!(!detached.is_hilighted(&manager, 0));
    detached.unhilight(&manager, 0);
    assert!(detached.hilight_with_color(&manager, &style, 0).is_err());
}
#[test]
fn json_dump_reports_native_flags_without_base_at_depth_zero() {
    let object = object();
    object.set_size(5.).unwrap();
    let mut stream = rustycad_tkernel::standard::dump::DumpStream::default();
    object.dump_json(&mut stream, 0).unwrap();
    let text = String::from_utf8(stream.bytes().to_vec()).unwrap();
    assert!(text.contains("\"className\": \"AIS_Trihedron\""));
    assert!(text.contains("\"HasOwnSize\": 1"));
    assert!(!text.contains("AIS_InteractiveObject"));
}
#[test]
fn selected_parts_deduplicate_and_restore_live_axis_and_hidden_plane_aspects() {
    let object = object();
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    object.set_context(&context);
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    object.compute(&manager, &prs.cast(), 0).unwrap();
    let style = Handle::new(Prs3dDrawer::new());
    style.setup_own_defaults().unwrap();
    style.line_aspect().set_color(Color::named(NamedColor::RED));
    object.set_hilight_attributes(&style);
    let axis = AisTrihedronOwner::new(&object.cast(), Prs3dDatumParts::XAxis, 7);
    let plane = AisTrihedronOwner::new(&object.cast(), Prs3dDatumParts::XOYAxis, 5);
    object
        .hilight_selected(&manager, &[axis.cast(), axis.cast(), plane.cast()])
        .unwrap();
    let fields = object.trihedron_base().fields.read();
    assert_eq!(
        fields.selected_parts,
        vec![Prs3dDatumParts::XAxis, Prs3dDatumParts::XOYAxis]
    );
    assert_eq!(
        fields.part_groups[Prs3dDatumParts::XAxis as usize].aspects(),
        style
            .line_aspect()
            .aspect()
            .cast::<dyn Graphic3dAspectsSource>()
    );
    drop(fields);
    object.clear_selected();
    let fields = object.trihedron_base().fields.read();
    assert!(fields.selected_parts.is_empty());
    assert_eq!(
        fields.part_groups[Prs3dDatumParts::XAxis as usize].aspects(),
        object
            .attributes()
            .datum_aspect()
            .line_aspect(Prs3dDatumParts::XAxis)
            .aspect()
            .cast::<dyn Graphic3dAspectsSource>()
    );
    assert_eq!(
        fields.part_groups[Prs3dDatumParts::XOYAxis as usize].aspects(),
        fields
            .hidden_line_aspect
            .cast::<dyn Graphic3dAspectsSource>()
    );
}
#[test]
fn copied_datum_is_independent_and_ownerless_highlight_uses_object_mode() {
    let object = object();
    let linked = object.attributes().link();
    linked.datum_aspect().set_axis_length(2., 3., 4.);
    object.set_own_datum_aspect().unwrap();
    let own = object.attributes().datum_aspect();
    object.set_own_datum_aspect().unwrap();
    assert_eq!(object.attributes().datum_aspect(), own);
    linked.datum_aspect().set_axis_length(20., 30., 40.);
    // Native CopyAspectsFrom copies appearance, leaving default lengths intact.
    assert_eq!(object.size(), 100.);
    let (manager, prs) = compute(&object);
    let old_groups = prs.groups().len();
    object.compute(&manager, &prs, 1).unwrap();
    assert_eq!(prs.groups().len(), old_groups);
    let style = Handle::new(Prs3dDrawer::new());
    style.setup_own_defaults().unwrap();
    let whole = SelectMgrEntityOwner::with_selectable(&object.cast(), 5);
    object
        .hilight_owner_with_color(&manager, &style, &whole.cast())
        .unwrap();
    assert!(manager.is_highlighted(&object.cast(), 0));
    let part_owner = AisTrihedronOwner::new(&object.cast(), Prs3dDatumParts::XAxis, 7);
    assert!(part_owner.is_hilighted(&manager, 0));
    part_owner.unhilight(&manager, 99);
    assert!(!part_owner.is_hilighted(&manager, 0));
}
