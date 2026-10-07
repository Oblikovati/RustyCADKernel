//! The fixture `DNaming::LoadImportedShape` of the TNaming GTests, translated from
//! src/Draw/TKDCAF/DNaming/DNaming.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871. DNaming is
//! a DRAW package (TKDCAF) the port does not provide; the GTests use only this loader, which
//! names an imported shape and its sub-shapes with TNaming builders.
use super::explore;
use rustycad_tkbrep::brep_tools::map_3d_edges;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkcaf::tnaming::TNamingBuilder;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::tdf::{TdfLabel, TdfTagSource};

type NeighbourFaces = DataMap<Shape, Vec<Shape>, ShapeMapHasher>;

/// `DNaming::LoadImportedShape(theResultLabel, theShape)`.
pub fn load_imported_shape(label: &TdfLabel, shape: &Shape) {
    label.forget_all_attributes(true).unwrap();
    TNamingBuilder::new(label)
        .unwrap()
        .generated(shape)
        .unwrap();
    let tagger = TdfTagSource::set(label).unwrap();
    tagger.set_value(0).unwrap();
    load_first_level(shape, &tagger);
    load_c0_edges(shape, &tagger);
    load_c0_vertices(shape, &tagger);
}

/// `TNaming_Builder(Tagger->NewChild()).Generated(shape)`.
fn generated_child(tagger: &Handle<TdfTagSource>, shape: &Shape) {
    let child = tagger.new_child().unwrap();
    TNamingBuilder::new(&child)
        .unwrap()
        .generated(shape)
        .unwrap();
}

/// The faces around each sub-shape of a type, each face once.
fn neighbour_faces(shape: &Shape, kind: ShapeEnum) -> NeighbourFaces {
    let mut neighbours = NeighbourFaces::default();
    for face in explore(shape, ShapeEnum::Face) {
        for sub_shape in explore(&face, kind) {
            if !neighbours.is_bound(&sub_shape) {
                neighbours.bind(sub_shape.clone(), Vec::new());
            }
            let faces = neighbours.change_find(&sub_shape).unwrap();
            if !faces.iter().any(|known| known.is_same(&face)) {
                faces.push(face.clone());
            }
        }
    }
    neighbours
}

/// `LoadC0Vertices`: the vertices with fewer than three faces around.
fn load_c0_vertices(shape: &Shape, tagger: &Handle<TdfTagSource>) {
    let neighbours = neighbour_faces(shape, ShapeEnum::Vertex);
    for (vertex, faces) in neighbours.items() {
        if faces.len() < 3 {
            generated_child(tagger, vertex);
        }
    }
}

/// `LoadC0Edges`: the edges that share their neighbour faces with another edge.
fn load_c0_edges(shape: &Shape, tagger: &Handle<TdfTagSource>) {
    let mut neighbours = neighbour_faces(shape, ShapeEnum::Edge);
    let mut to_delete = MapOfShape::default();
    // mpv: a new explorer keeps the order of the edges.
    for edge in explore(shape, ShapeEnum::Edge) {
        let mut c0 = false;
        let Some(faces) = neighbours.seek(&edge).cloned() else {
            continue;
        };
        if faces.len() < 2 {
            // mpv (06.09.2002): these edges were already loaded.
            continue;
        }
        let others: Vec<(Shape, Vec<Shape>)> = neighbours
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (other, other_faces) in others {
            if to_delete.contains(&other)
                || edge.is_same(&other)
                || other_faces.len() != faces.len()
            {
                continue;
            }
            let matches = faces
                .iter()
                .map(|face| other_faces.iter().filter(|o| face.is_same(o)).count())
                .sum::<usize>();
            if matches == faces.len() {
                c0 = true;
                generated_child(tagger, &other);
                to_delete.add(other);
            }
        }
        // VUN (10/2/2005): no UnBind while iterating.
        for deleted in to_delete.iter() {
            neighbours.unbind(deleted);
        }
        neighbours.unbind(&edge);
        if c0 {
            generated_child(tagger, &edge);
        }
    }
}

/// `LoadFirstLevel`: the items of a compound (recursively), else the next levels.
fn load_first_level(shape: &Shape, tagger: &Handle<TdfTagSource>) {
    let kind = shape.shape_type().unwrap();
    if !matches!(kind, ShapeEnum::Compound | ShapeEnum::CompSolid) {
        load_next_levels(shape, tagger);
        return;
    }
    for item in ShapeIterator::new(shape, true, true) {
        generated_child(tagger, &item);
        if matches!(
            item.shape_type().unwrap(),
            ShapeEnum::Compound | ShapeEnum::CompSolid
        ) {
            load_first_level(&item, tagger);
        } else {
            load_next_levels(&item, tagger);
        }
    }
}

/// `LoadNextLevels`: the faces of a solid; the faces and free edges of a shell or face; the
/// edges and vertices of a wire; the vertices of an edge.
fn load_next_levels(shape: &Shape, tagger: &Handle<TdfTagSource>) {
    match shape.shape_type().unwrap() {
        ShapeEnum::Solid => {
            for face in explore(shape, ShapeEnum::Face) {
                generated_child(tagger, &face);
            }
        }
        ShapeEnum::Shell | ShapeEnum::Face => load_faces_and_free_edges(shape, tagger),
        ShapeEnum::Wire => load_wire(shape, tagger),
        ShapeEnum::Edge => {
            for vertex in explore(shape, ShapeEnum::Vertex) {
                generated_child(tagger, &vertex);
            }
        }
        _ => {}
    }
}

/// The shell and face branch of `LoadNextLevels`.
fn load_faces_and_free_edges(shape: &Shape, tagger: &Handle<TdfTagSource>) {
    let faces = explore(shape, ShapeEnum::Face);
    let mut distinct = IndexedMapOfShape::default();
    for face in &faces {
        distinct.add(face.clone());
    }
    let shell = shape.shape_type().unwrap() == ShapeEnum::Shell;
    if distinct.extent() > 1 || (shell && distinct.extent() == 1) {
        for face in &faces {
            generated_child(tagger, face);
        }
    }
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)
        .unwrap();
    for (edge, faces, _) in ancestors.indexed_items() {
        let faces: Vec<&Shape> = faces.iter().collect();
        if faces.len() < 2 || faces[0].is_equal(faces[1]) {
            generated_child(tagger, edge);
        }
    }
}

/// The wire branch of `LoadNextLevels` (with the generated dangle vertices).
fn load_wire(shape: &Shape, tagger: &Handle<TdfTagSource>) {
    let mut edges = IndexedMapOfShape::default();
    map_3d_edges(shape, &mut edges).unwrap();
    if edges.extent() == 1 {
        generated_child(tagger, edges.find_key(1).unwrap());
        for vertex in explore(shape, ShapeEnum::Vertex) {
            generated_child(tagger, &vertex);
        }
        return;
    }
    for edge in explore(shape, ShapeEnum::Edge) {
        generated_child(tagger, &edge);
    }
    let dangles = dangle_shapes(shape);
    if dangles.is_empty() {
        return;
    }
    let builder = TNamingBuilder::new(&tagger.new_child().unwrap()).unwrap();
    for (edge, vertex) in dangles.items() {
        builder.generated_from(edge, vertex).unwrap();
    }
}

/// `GetDangleShapes(S, TopAbs_EDGE, Dangles)`: the vertices with a single edge, by edge.
fn dangle_shapes(shape: &Shape) -> DataMap<Shape, Shape, ShapeMapHasher> {
    let mut dangles = DataMap::<Shape, Shape, ShapeMapHasher>::default();
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Vertex, ShapeEnum::Edge, &mut ancestors)
        .unwrap();
    for (vertex, edges, _) in ancestors.indexed_items() {
        if edges.extent() == 1 {
            dangles.bind(edges.first().unwrap().clone(), vertex.clone());
        }
    }
    dangles
}
