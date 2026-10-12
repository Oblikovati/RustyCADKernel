//! The actual native ColoredShape oracle is projected only onto the owned map prerequisite.
use super::shape_drawer_oracle_rows::shape_drawer_row;
use super::{AisColoredDrawer, AisShapeDrawerMap, shape_drawer_oracle_inputs::shape_drawer_vertex};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use std::fmt::Write;
struct ShapeDrawerLifecycleInputs {
    key: Shape,
    reversed: Shape,
    outside: Shape,
    parent: Handle<Prs3dDrawer>,
    baseline_keys: i32,
    baseline_parent: i32,
}
impl ShapeDrawerLifecycleInputs {
    fn new() -> Self {
        let key = shape_drawer_vertex(Point3::new(1., 2., 3.));
        let reversed = key.reversed();
        let outside = shape_drawer_vertex(Point3::new(1., 2., 3.));
        let parent = Handle::new(Prs3dDrawer::new());
        let baseline_keys = key.tshape().ref_count();
        let baseline_parent = parent.ref_count();
        Self {
            key,
            reversed,
            outside,
            parent,
            baseline_keys,
            baseline_parent,
        }
    }
}
struct ShapeDrawerLifecycleReplay<'a> {
    output: &'a mut String,
    map: &'a AisShapeDrawerMap,
    input: &'a ShapeDrawerLifecycleInputs,
    children: [Handle<AisColoredDrawer>; 4],
}
impl ShapeDrawerLifecycleReplay<'_> {
    fn record(&mut self, label: &str) {
        let key_extra = self.input.key.tshape().ref_count() - self.input.baseline_keys;
        let link_extra = self.input.parent.ref_count() - self.input.baseline_parent;
        let prefix = format!("{label} {} {key_extra} {link_extra}", self.map.len());
        write!(self.output, "{prefix}").unwrap();
        for child in &self.children {
            write!(self.output, " {}", child.ref_count()).unwrap();
        }
        writeln!(self.output).unwrap();
    }
    fn missing_custom(&mut self) {
        assert!(
            !self
                .map
                .unset_custom_aspects(&self.input.outside, false, &self.input.parent)
                .unwrap()
        );
        self.record("missing-replace");
        assert!(
            !self
                .map
                .unset_custom_aspects(&self.input.outside, true, &self.input.parent)
                .unwrap()
        );
        self.record("missing-unregister");
    }
    fn initial_custom(&mut self) {
        let map = self.map;
        let retained_map = map.as_const();
        let (first, created) = map
            .custom_aspects(&self.input.key, &self.input.parent)
            .unwrap();
        assert!(created);
        self.children[0] = first;
        let node = retained_map.node_ref(&self.input.key).unwrap();
        let address = node.value_reference_address().unwrap();
        self.record("created");
        self.repeat_custom(&retained_map);
        self.children[0].set_hidden(true);
        self.replacement_custom(&node, address);
    }
    fn repeat_custom(&mut self, retained_map: &super::AisShapeDrawerMapRef<'_>) {
        let (repeat, created) = self
            .map
            .custom_aspects(&self.input.reversed, &self.input.parent)
            .unwrap();
        assert!(!created);
        let same = i32::from(repeat == self.children[0]);
        let extent = i32::try_from(retained_map.len()).unwrap();
        shape_drawer_row(self.output, "reused", &[same, extent]);
    }
    fn replacement_custom(&mut self, node: &super::AisShapeDrawerNodeRef<'_>, address: *const ()) {
        assert!(
            self.map
                .unset_custom_aspects(&self.input.key, false, &self.input.parent)
                .unwrap()
        );
        self.record_replacement(node, address);
        self.children[1] = self
            .map
            .custom_aspects(&self.input.reversed, &self.input.parent)
            .unwrap()
            .0;
        self.record("replacement-counts");
        assert!(
            self.map
                .unset_custom_aspects(&self.input.key, true, &self.input.parent)
                .unwrap()
        );
        self.record("unregistered-unused-alias");
    }
    fn record_replacement(&mut self, node: &super::AisShapeDrawerNodeRef<'_>, address: *const ()) {
        let current = self.map.node_ref(&self.input.key).unwrap();
        let same = i32::from(current.value_reference_address().unwrap() == address);
        let replaced =
            i32::from(node.drawer_address().unwrap() != self.children[0].object_address());
        let hidden = i32::from(node.clone_drawer().unwrap().is_hidden());
        shape_drawer_row(self.output, "replaced", &[same, replaced, hidden]);
    }
    fn null_value_custom(&mut self) {
        let map = self.map;
        let mutable_map = map.as_mut();
        mutable_map
            .bind(&self.input.reversed, &Handle::default())
            .unwrap();
        let mutable_value = mutable_map.change_value_ref(&self.input.key).unwrap();
        mutable_value.assign(&Handle::default()).unwrap();
        let node = mutable_value.as_const();
        let address = node.value_reference_address().unwrap();
        self.children[2] = map
            .custom_aspects(&self.input.key, &self.input.parent)
            .unwrap()
            .0;
        self.record_null_replacement(&node, address);
        self.record("null-replacement-counts");
        self.outside_custom();
    }
    fn record_null_replacement(
        &mut self,
        node: &super::AisShapeDrawerNodeRef<'_>,
        address: *const (),
    ) {
        let orientation = node.orientation().unwrap() as i32;
        let replaced =
            i32::from(node.drawer_address().unwrap() == self.children[2].object_address());
        let current = self.map.node_ref(&self.input.key).unwrap();
        let same = i32::from(current.value_reference_address().unwrap() == address);
        shape_drawer_row(self.output, "null-replaced", &[orientation, replaced, same]);
    }
    fn outside_custom(&mut self) {
        self.children[3] = self
            .map
            .custom_aspects(&self.input.outside, &self.input.parent)
            .unwrap()
            .0;
        let outside = i32::from(!self.input.key.is_same(&self.input.outside));
        let owners = self.input.outside.tshape().ref_count();
        shape_drawer_row(self.output, "outside", &[outside, owners]);
        self.record("outside-counts");
    }
    fn release_custom(&mut self) {
        let map = self.map;
        let retained_map = map.as_const();
        let unused_node = retained_map.node_ref(&self.input.key).unwrap();
        let storage_lease = unused_node.live_node().unwrap();
        assert!(map.clear_custom_aspects());
        assert!(storage_lease.fields.read().is_none());
        let empty = i32::from(retained_map.is_empty());
        let same = i32::from(retained_map.is_same_reference(&map.as_mut().as_const()));
        shape_drawer_row(self.output, "map-reference", &[empty, same]);
        self.record("clear-unused-alias");
        self.release_empty_and_outputs();
    }
    fn release_empty_and_outputs(&mut self) {
        assert!(!self.map.clear_custom_aspects());
        self.record("clear-empty");
        for child in &mut self.children {
            child.nullify();
        }
        self.record("released-outputs");
    }
}
#[test]
fn shape_drawer_actual_colored_shape_custom_lifecycle_map_projection() {
    let input = ShapeDrawerLifecycleInputs::new();
    let map = AisShapeDrawerMap::default();
    let mut output = String::new();
    let mut replay = ShapeDrawerLifecycleReplay {
        output: &mut output,
        map: &map,
        input: &input,
        children: std::array::from_fn(|_| Handle::default()),
    };
    replay.record("initial");
    replay.missing_custom();
    replay.initial_custom();
    replay.null_value_custom();
    replay.release_custom();
    let fixture = include_str!("../../tests/fixtures/shape-drawer-map/native-lifecycle.txt");
    assert_eq!(output, lifecycle_map_projection(fixture));
}
fn lifecycle_map_projection(fixture: &str) -> String {
    // Real AIS_Shape retains its own shape/drawer; this isolated prerequisite claims only added map ownership.
    let projected = fixture
        .lines()
        .filter(|line| !line.starts_with("native-baseline "))
        .collect::<Vec<_>>();
    projected.join("\n") + "\n"
}
