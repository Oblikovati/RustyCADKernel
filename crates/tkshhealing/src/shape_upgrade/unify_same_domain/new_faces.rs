//! "Insert new faces instead of old ones" of `IntUnifyFaces` and `InsertWiresIntoFaces`
//! (ShapeUpgrade_UnifySameDomain.cxx): the merged faces recorded in the context.
use super::UnifyState;
use super::face_bounds::edges_of;
use super::face_groups::FaceGroup;
use super::face_wires::NewBoundary;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// `InsertWiresIntoFaces(theWires, theFaces, theRefFace)`: each wire added to the first
/// face containing the middle of the p-curve of its first edge (a wire classified in no
/// face is left out, the native debug assertion).
fn insert_wires_into_faces(
    wires: &[Shape],
    faces: &[Shape],
    ref_face: &Face,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for wire in wires {
        let first = ShapeIterator::new(wire, true, true)
            .value()
            .map_err(|_| empty_wire())?
            .clone();
        let adaptor = BRepAdaptorCurve2d::with_edge_on_face(&topods::edge(&first)?, ref_face)?;
        let middle = (adaptor.first_parameter()? + adaptor.last_parameter()?) / 2.;
        let point = adaptor.value(middle)?;
        for shape in faces {
            let face = topods::face(shape)?;
            let classifier = BRepTopAdaptorFClass2d::new(&face, precision::confusion())?;
            if classifier.perform(point, true)? == State::In {
                let mut required = shape.oriented(Orientation::Forward);
                builder.add(&mut required, wire)?;
                break;
            }
        }
    }
    Ok(())
}

impl UnifyState {
    /// The new faces recorded as the merge of the faces of the group.
    pub(super) fn insert_new_faces(
        &mut self,
        group: &FaceGroup,
        boundary: NewBoundary,
        internal_wires: Vec<Shape>,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let mut holes = boundary.wires;
        if boundary.faces.len() > 1 {
            insert_wires_into_faces(&holes, &boundary.faces, &group.ref_face)?;
            insert_wires_into_faces(&internal_wires, &boundary.faces, &group.ref_face)?;
            return self.merge_into_new_faces(group, &boundary.faces);
        }
        let result = match boundary.faces.first() {
            Some(face) => face.clone(),
            None => {
                // "one face without seam".
                let (surface, location) = BRepTool::surface_with_location(&group.ref_face)?;
                let mut face = Face::default();
                builder.make_face_on_located_surface(&mut face, &surface, &location, 0.)?;
                face.set_orientation(group.ref_orientation);
                face.into_shape()
            }
        };
        let mut target = result.oriented(Orientation::Forward);
        holes.extend(internal_wires);
        for wire in &holes {
            builder.add(&mut target, wire)?;
        }
        self.record_merge(&group.faces, &result)
    }

    /// `myContext->Merge(faces, newFace)` and the Face-NewFace map.
    fn record_merge(&mut self, faces: &[Shape], new_face: &Shape) -> Result<(), Failure> {
        self.context()?.merge(faces, new_face)?;
        for face in faces {
            self.face_new_face.bind(face.clone(), new_face.clone());
        }
        Ok(())
    }

    /// Several new faces: each one the merge of the old faces owning its edges (neither
    /// degenerated nor closed on the reference face).
    fn merge_into_new_faces(
        &mut self,
        group: &FaceGroup,
        new_faces: &[Shape],
    ) -> Result<(), Failure> {
        let mut edge_maps = Vec::with_capacity(group.faces.len());
        for face in &group.faces {
            let mut map = MapOfShape::default();
            topexp::map_shapes_into_map(face, &mut map, true, true);
            edge_maps.push(map);
        }
        for new_face in new_faces {
            let mut owners = Vec::new();
            let mut used = MapOfShape::default();
            for shape in edges_of(new_face)? {
                let edge = topods::edge(&shape)?;
                if BRepTool::degenerated(&edge)?
                    || BRepTool::is_closed_on_face(&edge, &group.ref_face)?
                {
                    continue;
                }
                let owner = edge_maps
                    .iter()
                    .position(|map| map.contains(&shape))
                    .ok_or_else(|| orphan_edge(group.faces.len()))?;
                if used.add(group.faces[owner].clone()) {
                    owners.push(group.faces[owner].clone());
                }
            }
            self.record_merge(&owners, new_face)?;
        }
        Ok(())
    }
}

fn empty_wire() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "ShapeUpgrade_UnifySameDomain: InsertWiresIntoFaces on an empty wire; expected an edge",
        "",
    )
}

fn orphan_edge(count: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!(
            "ShapeUpgrade_UnifySameDomain: an edge of a new face is in none of the {count} merged faces; expected one"
        ),
        "",
    )
}
