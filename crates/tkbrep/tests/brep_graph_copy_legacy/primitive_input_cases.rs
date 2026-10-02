use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPICopy, BRepBuilderAPIMakeShape,
};

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyCylinder_FaceCount() {
    let mut maker = BRepPrimAPIMakeCylinder::new(5.0, 20.0);
    let cylinder = maker.shape().unwrap().clone();
    let mut source = BRepGraph::new();
    source.shapes_mut().add(&cylinder).unwrap();
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    assert!(!target.is_empty());
    assert_eq!(target.topo().faces().nb(), source.topo().faces().nb());
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyFacesOnly_Compound() {
    let compound = compound_of_copied_faces();
    let mut source = BRepGraph::new();
    source.shapes_mut().add(&compound).unwrap();
    assert_face_only_counts(&source);
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    assert_face_only_counts(&target);
}

fn compound_of_copied_faces() -> Compound {
    let box_shape = support::make_box(10.0, 20.0, 30.0);
    let mut faces = Explorer::new(&box_shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let builder = Builder;
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    while faces.more() {
        add_deep_copy_face(&builder, &mut compound, faces.current());
        faces.next().unwrap();
    }
    compound
}

fn add_deep_copy_face(builder: &Builder, compound: &mut Compound, face: &Shape) {
    let mut copier = BRepBuilderAPICopy::with_shape(face, true, false).unwrap();
    builder.add(compound, copier.shape().unwrap()).unwrap();
}

fn assert_face_only_counts(graph: &BRepGraph) {
    assert!(!graph.is_empty());
    assert_eq!(graph.topo().faces().nb(), 6);
    assert_eq!(graph.topo().solids().nb(), 0);
    assert_eq!(graph.topo().shells().nb(), 0);
}
