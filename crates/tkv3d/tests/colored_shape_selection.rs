//! Real native entities and owner policies are replayed with actual axis selecting volumes.
#[allow(dead_code)]
#[path = "../src/ais/colored_shape_expansion_inputs.rs"]
mod selection_geometry;
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{Builder, Compound, Shape},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::*;
use rustycad_tkv3d::{
    ais::*, prs_mgr::PrsMgrPresentableObjectMembers, select_basics::SelectBasicsPickResult,
    select_mgr::*, select3d::*, std_select::*,
};
#[test]
fn native_colored_selection_replays_constructed_owners_and_real_axis_picks() {
    let native = include_str!("fixtures/colored-shape-compute/native-selection.txt");
    for row in native.lines() {
        replay_colored_selection_row(row);
    }
}
fn replay_colored_selection_row(row: &str) {
    let mut fields = row.split_whitespace();
    assert_eq!(fields.next(), Some("selection"));
    let scenario = next_selection_usize(&mut fields);
    let style = next_selection_usize(&mut fields);
    let mode = fields.next().unwrap().parse::<i32>().unwrap();
    let (object, shape, selection) = constructed_colored_selection(scenario, style, mode);
    let result = object.compute_selection(&selection, mode);
    assert_eq!(result.is_err(), fields.next() == Some("1"), "{row}");
    let entities = selection.entities();
    assert_eq!(entities.len(), next_selection_usize(&mut fields), "{row}");
    for wrapper in entities {
        check_colored_selection_entity(&wrapper.base_sensitive(), &object, &shape, &mut fields);
    }
    assert!(fields.next().is_none(), "{row}");
}
fn next_selection_usize(fields: &mut std::str::SplitWhitespace<'_>) -> usize {
    let token = fields.next().unwrap();
    token.parse::<usize>().unwrap()
}
fn constructed_colored_selection(
    scenario: usize,
    style: usize,
    mode: i32,
) -> (Handle<AisColoredShape>, Shape, Handle<SelectMgrSelection>) {
    let input = selection_geometry::ColoredExpansionInputs::new();
    let shape = colored_selection_input(&input, scenario);
    let object = AisColoredShape::new(&shape);
    object.attributes().set_auto_triangulation(false);
    configure_selection_custom(&object, &input, &shape, style);
    let selection = Handle::new(SelectMgrSelection::new(mode));
    let sentinel = SelectMgrEntityOwner::new(17).into_cast();
    selection
        .add(
            &Handle::new(Select3DSensitivePoint::new(
                &sentinel,
                Point3::new(9., 8., 7.),
            ))
            .cast(),
        )
        .unwrap();
    (object, shape, selection)
}
fn colored_selection_input(
    input: &selection_geometry::ColoredExpansionInputs,
    scenario: usize,
) -> Shape {
    match scenario {
        0 => Shape::default(),
        1 => empty_selection_compound(),
        2 => input.vertex.clone(),
        3 => input.edges[0].clone(),
        4 => input.wire.clone(),
        5 => input.faces[0].clone(),
        6 => BRepPrimAPIMakeCylinder::new(2., 4.)
            .solid()
            .unwrap()
            .into_shape(),
        7 => BRepPrimAPIMakeCone::new(2., 0., 4.)
            .unwrap()
            .solid()
            .unwrap()
            .into_shape(),
        8 => BRepPrimAPIMakeCone::new(2., 1., 4.)
            .unwrap()
            .solid()
            .unwrap()
            .into_shape(),
        9 => BRepPrimAPIMakeSphere::new(2.)
            .unwrap()
            .solid()
            .unwrap()
            .into_shape(),
        10 => BRepPrimAPIMakeBox::from_sizes(2., 3., 4.)
            .unwrap()
            .solid()
            .unwrap()
            .into_shape(),
        _ => input.base.clone(),
    }
}
fn empty_selection_compound() -> Shape {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    compound.into_shape()
}
fn configure_selection_custom(
    object: &AisColoredShape,
    input: &selection_geometry::ColoredExpansionInputs,
    shape: &Shape,
    style: usize,
) {
    let target = match style {
        0 => return,
        1 => &input.external,
        2 => shape,
        _ => {
            let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
            if faces.more() {
                object
                    .custom_aspects(faces.current())
                    .unwrap()
                    .set_hidden(true);
                return;
            }
            &input.edges[0]
        }
    };
    object.custom_aspects(target).unwrap().set_hidden(true);
}
fn check_colored_selection_entity(
    entity: &Handle<dyn Select3DSensitiveEntity>,
    object: &Handle<AisColoredShape>,
    shape: &Shape,
    fields: &mut std::str::SplitWhitespace<'_>,
) {
    assert_eq!(fields.next(), Some("entity"));
    assert_eq!(
        entity.dynamic_type().unwrap().name(),
        fields.next().unwrap()
    );
    assert_eq!(
        entity.nb_sub_elements(),
        fields.next().unwrap().parse::<i32>().unwrap()
    );
    let owner = entity.owner_id();
    assert_eq!(
        owner.priority(),
        fields.next().unwrap().parse::<i32>().unwrap()
    );
    assert_eq!(owner.comes_from_decomposition(), fields.next() == Some("1"));
    assert_eq!(
        owner.selectable().object_address() == object.object_address(),
        fields.next() == Some("1")
    );
    let brep = owner.cast::<StdSelectBRepOwner>();
    assert_eq!(!brep.is_null(), fields.next() == Some("1"));
    if !brep.is_null() {
        check_brep_selection_shape(&brep, shape, fields);
    }
    check_colored_selection_rays(entity, fields);
}
fn check_brep_selection_shape(
    owner: &StdSelectBRepOwner,
    shape: &Shape,
    fields: &mut std::str::SplitWhitespace<'_>,
) {
    let stored = owner.shape_ref().clone_shape();
    assert_eq!(
        stored.shape_type().unwrap() as usize,
        next_selection_usize(fields)
    );
    assert_eq!(stored.is_equal(shape), fields.next() == Some("1"));
}
fn check_colored_selection_rays(
    entity: &Handle<dyn Select3DSensitiveEntity>,
    fields: &mut std::str::SplitWhitespace<'_>,
) {
    for origin in [
        Point3::new(0., 0., 20.),
        Point3::new(1., 2., 20.),
        Point3::new(50., 50., 20.),
    ] {
        assert_eq!(fields.next(), Some("pick"));
        let mut volume = SelectMgrSelectingVolumeManager::new();
        volume.init_axis_selecting_volume(Axis1::new(origin, -Direction3::Z));
        volume.build_selecting_volume().unwrap();
        let mut pick = SelectBasicsPickResult::default();
        let hit = entity.matches(&volume, &mut pick).unwrap();
        assert_eq!(hit, fields.next() == Some("1"));
        if hit {
            check_selection_pick_values(&pick, fields);
        }
    }
}
fn check_selection_pick_values(
    pick: &SelectBasicsPickResult,
    fields: &mut std::str::SplitWhitespace<'_>,
) {
    let depth = fields.next().unwrap().parse::<f64>().unwrap();
    let center = fields.next().unwrap().parse::<f64>().unwrap();
    assert!(
        (pick.depth() - depth).abs() < 1e-9,
        "actual depth={}; expected {depth}",
        pick.depth()
    );
    assert!(
        (pick.dist_to_geom_center() - center).abs() < 1e-9,
        "actual center={}; expected {center}",
        pick.dist_to_geom_center()
    );
}
