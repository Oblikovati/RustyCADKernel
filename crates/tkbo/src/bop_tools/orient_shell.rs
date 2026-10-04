use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::{self, Explorer},
    topods::{Edge, Face, Shape, Shell},
    toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape},
};
use rustycad_tkernel::{ncollection::List, standard::Failure};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
struct ShellOrienter {
    result: Shell,
    processed: IndexedMapOfShape,
}
/// Rebuild shell faces from the native edge-adjacency passes; `orient_faces_on_shell(&mut shell)?`.
pub fn orient_faces_on_shell(shell: &mut Shape) -> Result<(), Failure> {
    let mut adjacent = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shell, ShapeEnum::Edge, ShapeEnum::Face, &mut adjacent)?;
    let mut result = Shell::default();
    BRepBuilder::default().make_shell(&mut result);
    let mut orienter = ShellOrienter {
        result,
        processed: IndexedMapOfShape::default(),
    };
    visit_face_adjacency(&adjacent, &mut orienter, true)?;
    visit_face_adjacency(&adjacent, &mut orienter, false)?;
    *shell = orienter.result.into_shape();
    Ok(())
}
fn unique_faces(faces: &List<Shape>) -> Vec<Face> {
    let mut fence = IndexedMapOfShape::default();
    faces
        .iter()
        .filter_map(|face| {
            if fence.contains(face) {
                return None;
            }
            fence.add(face.clone());
            Some(Face::unchecked(face.clone()))
        })
        .collect()
}
fn visit_face_adjacency(
    adjacent: &IndexedDataMapOfShapeListOfShape,
    orienter: &mut ShellOrienter,
    paired: bool,
) -> Result<(), Failure> {
    for index in 1..=adjacent.extent() {
        let edge = Edge::unchecked(adjacent.find_key(index)?.clone());
        if BRepTool::degenerated(&edge)? {
            continue;
        }
        let faces = unique_faces(adjacent.find_from_index(index)?);
        if paired && faces.len() == 2 {
            orienter.connect(&edge, &faces[0], &faces[1])?;
        }
        if !paired && faces.len() != 2 {
            for face in faces {
                orienter.append(&face)?;
            }
        }
    }
    Ok(())
}
impl ShellOrienter {
    fn append(&mut self, face: &Face) -> Result<(), Failure> {
        if self.processed.contains(face) {
            return Ok(());
        }
        self.processed.add(face.clone().into_shape());
        BRepBuilder::default().add(&mut self.result, face)
    }
    fn connect(&mut self, edge: &Edge, first: &Face, second: &Face) -> Result<(), Failure> {
        let seen_first = self.processed.contains(first);
        let seen_second = self.processed.contains(second);
        if seen_first && seen_second {
            return Ok(());
        }
        if !seen_first && !seen_second {
            self.append(first)?;
        }
        let (known, new) = if seen_second {
            (second, first)
        } else {
            (first, second)
        };
        let index = self.processed.find_index(known);
        let known_oriented = Face::unchecked(self.processed.find_key(index)?.clone());
        let mut next = new.clone();
        if edge_orientation(edge, &known_oriented)? == edge_orientation(edge, new)?
            && !BRepTool::is_closed_on_face(edge, first)?
            && !BRepTool::is_closed_on_face(edge, second)?
        {
            next.reverse();
        }
        self.append(&next)
    }
}
fn edge_orientation(edge: &Edge, face: &Face) -> Result<Orientation, Failure> {
    for candidate in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if candidate.is_same(edge) {
            return Ok(candidate.orientation());
        }
    }
    Ok(Orientation::Internal)
}
