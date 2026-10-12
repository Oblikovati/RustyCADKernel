//! Native type aliases project the same real parent and grouping storage.
use rustycad_tkbrep::topods::{Builder, Compound};
use rustycad_tkernel::standard::Handle;
use rustycad_tkv3d::{ais::*, prs_mgr::PrsMgrPresentableObjectMembers};

#[test]
fn native_colored_parent_and_group_aliases_preserve_ownership_and_identity() {
    let mut shape = Compound::default();
    Builder.make_compound(&mut shape);
    let object = AisColoredShape::new(&shape);
    let mut rows = Vec::new();
    record_colored_parent_alias(&object, &shape, &mut rows);
    let drawer = Handle::new(AisColoredDrawer::new(&object.attributes()));
    let groups = AisColoredShapeDrawerCompounds::default();
    groups.add(&drawer, &shape).unwrap();
    record_colored_group_alias(&groups, &drawer, &shape, &mut rows);
    groups.clear(false);
    record_colored_alias_finish(&object, &drawer, &shape, &mut rows);
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("fixtures/colored-shape-compute/native-type-aliases.txt")
    );
}
fn record_colored_parent_alias(
    object: &Handle<AisColoredShape>,
    shape: &Compound,
    rows: &mut Vec<String>,
) {
    let parent: Handle<AisColoredShapeParent> = object.cast();
    rows.push(format!(
        "parent-alias {} {} {}",
        i32::from(parent.object_address() == object.object_address()),
        object.ref_count(),
        i32::from(parent.shape_ref().clone_shape().is_equal(shape))
    ));
}

fn record_colored_group_alias(
    groups: &AisColoredShapeDrawerCompounds,
    drawer: &Handle<AisColoredDrawer>,
    shape: &Compound,
    rows: &mut Vec<String>,
) {
    let node = groups.node_ref(1).unwrap();
    rows.push(format!(
        "group-alias {} {} {} {}",
        groups.len(),
        i32::from(groups.add(drawer, shape).unwrap() == 1),
        i32::from(node.clone_compound().unwrap().is_equal(shape)),
        drawer.ref_count()
    ));
}

fn record_colored_alias_finish(
    object: &Handle<AisColoredShape>,
    drawer: &Handle<AisColoredDrawer>,
    shape: &Compound,
    rows: &mut Vec<String>,
) {
    rows.push(format!(
        "aliases-finished {} {} {}",
        object.ref_count(),
        drawer.ref_count(),
        shape.tshape().ref_count()
    ));
}
