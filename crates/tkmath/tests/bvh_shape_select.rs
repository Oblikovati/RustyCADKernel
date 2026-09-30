//! BVH_ShapeSelect_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Original solid, topology maps, BRep bounds and typed/explicit-tree traversals.
#![allow(non_snake_case)]
use rustycad_tkbrep::{topexp, topods::Shape, toptools::IndexedMapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    bnd::{BoundingBox3, box3_to_bvh},
    bvh::{
        BvhBaseTraverse, BvhBox, BvhBoxSet, BvhLinearBuilder, BvhPairTraverse,
        BvhPrimitiveSetAccess, BvhSet, BvhSetTraverse, BvhTraverse, BvhVec3d,
    },
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use std::sync::Arc;

type ShapeBoxSet = BvhBoxSet<BvhVec3d, Shape>;
fn shape_selection_bounds(shape: &Shape) -> BvhBox<BvhVec3d> {
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add(shape, &mut bounds, true).unwrap();
    box3_to_bvh(&bounds).unwrap()
}
fn make_shape_box_set(shape: &Shape) -> ShapeBoxSet {
    let mut set = ShapeBoxSet::new(Some(Arc::new(BvhLinearBuilder::default())));
    let mut shapes = IndexedMapOfShape::default();
    for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
        topexp::map_shapes_of_type(shape, kind, &mut shapes).unwrap();
    }
    for index in 1..=shapes.extent() {
        let shape = shapes.find_key(index).unwrap();
        set.add(shape.clone(), shape_selection_bounds(shape));
    }
    set.build().unwrap();
    set
}
struct ShapeSelector<'a> {
    set: &'a mut ShapeBoxSet,
    selection: BvhBox<BvhVec3d>,
    accepted: i32,
}
impl BvhBaseTraverse<bool> for ShapeSelector<'_> {}
impl BvhTraverse<BvhVec3d, bool> for ShapeSelector<'_> {
    fn reject_node(&self, low: &BvhVec3d, high: &BvhVec3d, inside: &mut bool) -> bool {
        let (contains, overlap) = self.selection.contains_corners(low, high);
        *inside = contains;
        !overlap
    }
    fn accept_metric(&self, inside: &bool) -> bool {
        *inside
    }
    fn accept(&mut self, index: i32, inside: &bool) -> Result<bool, Failure> {
        let accept = *inside || !self.selection.is_out(&self.set.element_box(index)?);
        if accept {
            self.accepted += 1;
        }
        Ok(accept)
    }
}
impl BvhSetTraverse<BvhVec3d, bool> for ShapeSelector<'_> {
    type Set = ShapeBoxSet;
    fn bvh_set_mut(&mut self) -> Option<&mut ShapeBoxSet> {
        Some(self.set)
    }
}
fn check_shape_selection(set: &mut ShapeBoxSet, shape: &Shape, expected: i32) {
    let selection = shape_selection_bounds(shape);
    let mut typed = ShapeSelector {
        set,
        selection,
        accepted: 0,
    };
    assert_eq!(typed.select().unwrap(), expected);
    assert_eq!(typed.accepted, expected);
    let tree = set.bvh().unwrap().clone();
    let mut explicit = ShapeSelector {
        set,
        selection,
        accepted: 0,
    };
    assert_eq!(explicit.select_tree(Some(&tree)).unwrap(), expected);
    assert_eq!(explicit.accepted, expected);
}
fn check_selection_kind(
    shape: &Shape,
    set: &mut ShapeBoxSet,
    kind: ShapeEnum,
    expected: i32,
    count: usize,
) {
    let mut shapes = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut shapes).unwrap();
    for index in 1..=shapes.extent() {
        check_shape_selection(set, shapes.find_key(index).unwrap(), expected);
    }
    assert_eq!(shapes.extent(), count);
}
#[test]
fn BVH_ShapeSelectTest__Bug30655_ShapeSelectionTypedAndVoid() {
    let shape = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .solid()
        .unwrap();
    let mut set = make_shape_box_set(&shape);
    check_selection_kind(&shape, &mut set, ShapeEnum::Vertex, 7, 8);
    check_selection_kind(&shape, &mut set, ShapeEnum::Edge, 11, 12);
    check_selection_kind(&shape, &mut set, ShapeEnum::Face, 17, 6);
}
struct PairSelector<'a> {
    set: &'a ShapeBoxSet,
    accepted: i32,
}
impl BvhBaseTraverse<bool> for PairSelector<'_> {}
impl BvhPairTraverse<BvhVec3d, bool> for PairSelector<'_> {
    fn reject_node(
        &self,
        low1: &BvhVec3d,
        high1: &BvhVec3d,
        low2: &BvhVec3d,
        high2: &BvhVec3d,
        metric: &mut bool,
    ) -> bool {
        *metric = !BvhBox::from_corners(*low1, *high1).is_out_corners(low2, high2);
        !*metric
    }
    fn accept(&mut self, first: i32, second: i32) -> Result<bool, Failure> {
        let accept = !self
            .set
            .element_box(first)?
            .is_out(&self.set.element_box(second)?);
        if accept {
            self.accepted += 1;
        }
        Ok(accept)
    }
}
#[test]
fn BVH_ShapeSelectTest__Bug30655_PairSelectionTypedAndVoid() {
    let shape = BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
        .unwrap()
        .solid()
        .unwrap();
    let mut set = make_shape_box_set(&shape);
    let tree = set.bvh().unwrap().clone();
    // BvhPairSetTraverse documents explicit trees for a set paired with itself,
    // because two simultaneous mutable set borrows would violate Rust's alias rules.
    let mut typed = PairSelector {
        set: &set,
        accepted: 0,
    };
    assert_eq!(typed.select_trees(Some(&tree), Some(&tree)).unwrap(), 290);
    assert_eq!(typed.accepted, 290);
    let mut explicit = PairSelector {
        set: &set,
        accepted: 0,
    };
    assert_eq!(
        explicit.select_trees(Some(&tree), Some(&tree)).unwrap(),
        290
    );
    assert_eq!(explicit.accepted, 290);
}
