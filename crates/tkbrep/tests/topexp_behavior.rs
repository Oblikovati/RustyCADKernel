//! TopExp behaviour beyond the upstream suite: explorer quirks, vertex queries and the
//! maps. The native kernel comparison is the br_explore, br_maps and br_vertices probes.
mod support;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Builder, Compound, Shape, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::toploc::Location;
use support::{edge_between, make_box, make_vertex};

fn point(x: f64) -> Point3 {
    Point3::new(x, 0.0, 0.0)
}

fn compound_of(shapes: &[Shape]) -> Compound {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    for shape in shapes {
        Builder.add(&mut compound, shape).unwrap();
    }
    compound
}

#[test]
fn explorer_restarts_after_the_end_and_reports_the_root() {
    let solid = make_box(1.0, 1.0, 1.0);
    let mut explorer = Explorer::new(&solid, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let first = explorer.current().clone();
    // The solid and shell iterators are stacked above the found face.
    assert_eq!(explorer.depth(), 2);
    while explorer.more() {
        explorer.next().unwrap();
    }
    assert!(explorer.current().is_equal(&solid));
    assert_eq!(explorer.depth(), 0);
    explorer.next().unwrap();
    assert!(explorer.more() && explorer.current().is_equal(&first));
    explorer.clear();
    assert!(!explorer.more() && explorer.explored_shape().is_equal(&solid));
}

#[test]
fn explorer_finds_the_root_itself_and_honours_avoid() {
    let solid = make_box(1.0, 1.0, 1.0);
    let mut root = Explorer::new(&solid, ShapeEnum::Solid, ShapeEnum::Shape).unwrap();
    assert!(root.more() && root.current().is_equal(&solid));
    root.next().unwrap();
    assert!(!root.more());
    let under_faces = Explorer::new(&solid, ShapeEnum::Edge, ShapeEnum::Face).unwrap();
    // The only edge outside the faces is the internal edge of the fake box: none here.
    assert_eq!(under_faces.count(), 0);
    let too_complex = Explorer::new(&solid, ShapeEnum::Compound, ShapeEnum::Shape).unwrap();
    assert!(!too_complex.more());
    assert_eq!(
        Explorer::new(&solid, ShapeEnum::Vertex, ShapeEnum::Shape)
            .unwrap()
            .count(),
        48
    );
}

#[test]
fn explorer_on_a_null_shape_reports_null_object_on_next() {
    let mut explorer = Explorer::new(&Shape::default(), ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    assert!(!explorer.more());
    assert_eq!(
        explorer.next().unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert!(Explorer::default().next().is_err());
}

#[test]
fn edge_vertices_follow_the_orientation_codes() {
    let (a, b, c) = (
        make_vertex(point(0.0)),
        make_vertex(point(1.0)),
        make_vertex(point(2.0)),
    );
    let edge = edge_between(&a, &b);
    assert!(topexp::first_vertex(&edge, false).is_same(&a));
    assert!(topexp::last_vertex(&edge.reversed().into_edge(), true).is_same(&a));
    let mut twice = edge.clone();
    Builder.make_shape(&mut twice, &edge.empty_copied().unwrap().tshape().clone());
    Builder
        .add(&mut twice, &a.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(&mut twice, &c.oriented(Orientation::Forward))
        .unwrap();
    // FirstVertex stops at the first FORWARD vertex, Vertices keeps the last one.
    assert!(topexp::first_vertex(&twice, false).is_same(&a));
    let (first, last) = topexp::vertices(&twice, false);
    assert!(first.is_same(&c) && last.is_null());
    // Vertices(twice) is (c, null), so only an edge through c meets it.
    assert!(topexp::common_vertex(&twice, &edge).is_none());
    let through_c = topexp::common_vertex(&edge_between(&c, &b), &twice).unwrap();
    assert!(through_c.is_same(&c));
    assert!(topexp::common_vertex(&edge_between(&c, &c), &edge).is_none());
}

trait IntoEdge {
    fn into_edge(self) -> rustycad_tkbrep::topods::Edge;
}
impl IntoEdge for Shape {
    fn into_edge(self) -> rustycad_tkbrep::topods::Edge {
        rustycad_tkbrep::topods::Edge::unchecked(self)
    }
}

fn wire_of(edges: &[(Shape, Orientation)]) -> Wire {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for (edge, orientation) in edges {
        Builder
            .add(&mut wire, &edge.oriented(*orientation))
            .unwrap();
    }
    wire
}

#[test]
fn wire_vertices_of_open_closed_and_branching_wires() {
    let v: Vec<_> = (0..4).map(|x| make_vertex(point(f64::from(x)))).collect();
    let e01: Shape = edge_between(&v[0], &v[1]).into();
    let e12: Shape = edge_between(&v[1], &v[2]).into();
    let e20: Shape = edge_between(&v[2], &v[0]).into();
    let open = wire_of(&[
        (e01.clone(), Orientation::Forward),
        (e12.clone(), Orientation::Forward),
    ]);
    let (first, last) = topexp::wire_vertices(&open);
    assert!(first.is_same(&v[0]) && last.is_same(&v[2]));
    let closed = wire_of(&[
        (e01.clone(), Orientation::Forward),
        (e12, Orientation::Forward),
        (e20, Orientation::Forward),
    ]);
    let (first, last) = topexp::wire_vertices(&closed);
    assert!(first.is_same(&v[0]) && last.is_same(&v[0]));
    assert_eq!(
        (first.orientation(), last.orientation()),
        (Orientation::Forward, Orientation::Reversed)
    );
    let apart: Shape = edge_between(&v[2], &v[3]).into();
    let broken = wire_of(&[(e01, Orientation::Forward), (apart, Orientation::Forward)]);
    let (first, last) = topexp::wire_vertices(&broken);
    assert!(first.is_null() && last.is_null());
}

#[test]
fn ancestor_maps_list_each_ancestor_once_when_asked() {
    let vertex = make_vertex(point(0.0));
    let edge: Shape = edge_between(&vertex, &vertex).into();
    let wire = wire_of(&[
        (edge.clone(), Orientation::Forward),
        (edge.clone(), Orientation::Reversed),
    ]);
    let compound = compound_of(&[wire.into()]);
    let mut all = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&compound, ShapeEnum::Vertex, ShapeEnum::Edge, &mut all)
        .unwrap();
    assert_eq!((all.extent(), all[1].extent()), (1, 4));
    let mut same = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_unique_ancestors(
        &compound,
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut same,
        false,
    )
    .unwrap();
    assert_eq!(same[1].extent(), 1);
    let mut oriented = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_unique_ancestors(
        &compound,
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut oriented,
        true,
    )
    .unwrap();
    assert_eq!(oriented[1].extent(), 2);
}

#[test]
fn ancestor_maps_bind_free_sub_shapes_to_empty_lists() {
    let loose = make_vertex(point(3.0));
    let compound = compound_of(&[loose.clone().into()]);
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&compound, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map)
        .unwrap();
    assert_eq!((map.extent(), map[1].extent()), (1, 0));
    assert!(map.find_key(1).unwrap().is_same(&loose));
}

#[test]
fn map_shapes_use_the_cumulation_flags_on_the_first_level_only() {
    let offset = Location::new(Transform3::translation([1.0, 0.0, 0.0]));
    let vertex = make_vertex(point(0.0));
    let mut compound = compound_of(&[vertex.clone().into()]);
    compound.set_location(&offset, false).unwrap();
    let mut located = IndexedMapOfShape::default();
    topexp::map_shapes(&compound, &mut located, true, true);
    let mut raw = IndexedMapOfShape::default();
    topexp::map_shapes(&compound, &mut raw, true, false);
    assert!(!located[2].is_same(&vertex) && raw[2].is_same(&vertex));
    let mut plain = MapOfShape::default();
    topexp::map_shapes_into_map(&compound, &mut plain, true, true);
    assert!(plain.contains(&located[2]) && plain.extent() == 2);
    let mut typed = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&compound, ShapeEnum::Shape, &mut typed).unwrap();
    assert_eq!(typed.extent(), 0);
}
