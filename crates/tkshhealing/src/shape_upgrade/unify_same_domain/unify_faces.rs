//! `UnifyFaces` and `IntUnifyFaces` of ShapeUpgrade_UnifySameDomain.cxx: the faces of
//! each shell, then the faces out of shells, merged group by group.
use super::face_groups::{FaceGroup, GlobalMaps};
use super::face_wires::build_new_boundary;
use super::geometry::clear_rts;
use super::internal_wires::build_internal_wires;
use super::{DataMapOfShapeMapOfShape, ShapeUpgradeUnifySameDomain, UnifyState};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The shells of each face of the shape.
fn face_shells(shape: &Shape) -> Result<DataMapOfShapeMapOfShape, Failure> {
    let mut map = DataMapOfShapeMapOfShape::default();
    for shell in Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape)? {
        for face in ShapeIterator::new(&shell, true, true) {
            match map.change_seek(&face) {
                Some(shells) => {
                    shells.add(shell.clone());
                }
                None => {
                    let mut shells = MapOfShape::default();
                    shells.add(shell.clone());
                    map.bind(face, shells);
                }
            }
        }
    }
    Ok(map)
}

/// The non-degenerated edges with one face in the shells out of solids.
fn free_boundaries(shape: &Shape) -> Result<MapOfShape, Failure> {
    let mut free = MapOfShape::default();
    for shell in Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Solid)? {
        let mut map = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&shell, ShapeEnum::Edge, ShapeEnum::Face, &mut map)?;
        for index in 1..=map.extent() {
            let edge = map.find_key(index)?;
            if !BRepTool::degenerated(&topods::edge(edge)?)?
                && map.find_from_index(index)?.extent() == 1
            {
                free.add(edge.clone());
            }
        }
    }
    Ok(free)
}

impl UnifyState {
    /// `UnifyFaces()`.
    pub(super) fn unify_faces(&mut self) -> Result<(), Failure> {
        let mut faces = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&self.shape, ShapeEnum::Face, &mut faces)?;
        let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
        for index in 1..=faces.extent() {
            topexp::map_shapes_and_ancestors(
                faces.find_key(index)?,
                ShapeEnum::Edge,
                ShapeEnum::Face,
                &mut edge_faces,
            )?;
        }
        let shells = face_shells(&self.shape)?;
        let free_bounds = free_boundaries(&self.shape)?;
        let maps = GlobalMaps {
            edge_faces: &edge_faces,
            face_shells: &shells,
            free_bounds: &free_bounds,
        };
        // "unify faces in each shell separately".
        for shell in Explorer::new(&self.shape.clone(), ShapeEnum::Shell, ShapeEnum::Shape)? {
            self.int_unify_faces(&shell, maps)?;
        }
        // "gather all faces out of shells in one compound and unify them at once".
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let mut count = 0;
        for face in Explorer::new(&self.shape.clone(), ShapeEnum::Face, ShapeEnum::Shell)? {
            builder.add(&mut compound, &face)?;
            count += 1;
        }
        if count > 0 {
            let empty = DataMapOfShapeMapOfShape::default();
            let no_shells = GlobalMaps {
                face_shells: &empty,
                ..maps
            };
            self.int_unify_faces(compound.as_ref(), no_shells)?;
        }
        self.shape = self.apply(&self.shape)?;
        Ok(())
    }

    /// `IntUnifyFaces(theInpShape, theGMapEdgeFaces, theGMapFaceShells, theFreeBoundMap)`.
    pub(super) fn int_unify_faces(
        &mut self,
        shape: &Shape,
        maps: GlobalMaps<'_>,
    ) -> Result<(), Failure> {
        let mut local = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut local)?;
        let mut processed = MapOfShape::default();
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if processed.contains(&face) {
                continue;
            }
            let base = clear_rts(&BRepTool::surface(&topods::face(&face)?)?);
            // "Bug 33894: Prevent crash when face has no surface".
            if base.is_null() {
                continue;
            }
            let face = topods::face(&face)?;
            let mut group = self.collect_face_group(&face, base, maps, &local, &mut processed)?;
            self.unify_group(&mut group, maps)?;
        }
        Ok(())
    }

    /// The rest of the processing of one face: the group merged into new faces when it
    /// has more than one face.
    fn unify_group(&mut self, group: &mut FaceGroup, maps: GlobalMaps<'_>) -> Result<(), Failure> {
        if group.faces.len() > 1 {
            self.prepare_face_group(group, maps)?;
        }
        let map_ef = self.orient_group_edges(group)?;
        if group.faces.len() <= 1 {
            return Ok(());
        }
        let mut frame = self.prepare_periodic(group, &map_ef, maps)?;
        // The native FaceUmin and FaceVmin are computed here but never used.
        let Some(boundary) = build_new_boundary(group, &mut frame)? else {
            return Ok(()); // isWireBuildingFailed
        };
        let internal_wires = build_internal_wires(&mut group.internal)?;
        self.insert_new_faces(group, boundary, internal_wires)
    }
}

impl ShapeUpgradeUnifySameDomain {
    /// `IntUnifyFaces(theInpShape, theGMapEdgeFaces, theGMapFaceShells,
    /// theFreeBoundMap)` (protected): the faces of `shape` merged with the global maps of
    /// the shape; `unifier.int_unify_faces(&shell, &edge_faces, &face_shells, &free)?`.
    pub fn int_unify_faces(
        &self,
        shape: &Shape,
        edge_faces: &IndexedDataMapOfShapeListOfShape,
        face_shells: &DataMapOfShapeMapOfShape,
        free_bounds: &MapOfShape,
    ) -> Result<(), Failure> {
        let maps = GlobalMaps {
            edge_faces,
            face_shells,
            free_bounds,
        };
        self.state.write().int_unify_faces(shape, maps)
    }
}
