//! The volume stages of `BOPAlgo_MakerVolume::PerformInternal1`: faces, bounding box, solids,
//! box removal, internal shapes and result.
use super::BOPAlgoMakerVolume;
use crate::bop_algo::alerts::alert;
use crate::bop_algo::tools::fill_internals;
use crate::bop_algo::{AlertSolidBuilderFailed, BuilderSolid};
use crate::bop_ds::DataStructure;
use crate::bop_tools::treat_compound;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;

/// The faces of the bounding box solid, by identity.
type BoxFaces = Map<Shape, ShapeMapHasher>;

impl BOPAlgoMakerVolume {
    /// The faces, the box, the solids without the box solid, the internal shapes and the
    /// result (steps 4 to 9 of `PerformInternal1`); false when the solid builder failed.
    pub(super) fn build_volumes(&mut self, structure: &DataStructure) -> Result<bool, Failure> {
        self.collect_faces(structure)?;
        let box_faces = self.make_box()?;
        let Some(mut solids) = self.build_solids()? else {
            return Ok(false);
        };
        remove_box(&mut solids, &box_faces)?;
        self.fill_internal_shapes(&solids, structure)?;
        self.build_shape(&solids)?;
        Ok(true)
    }

    /// Both orientations of the splits of every source face, each split once, while the
    /// face boxes grow the bounding box (`CollectFaces`).
    fn collect_faces(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Face {
                continue;
            }
            self.bounds.add_box(info.box_bounds());
            let face = info.shape();
            let Some(images) = self.builder.images.seek(face) else {
                add_face(face, &mut self.faces);
                continue;
            };
            for image in images.iter() {
                if fence.add(image.clone()) {
                    add_face(image, &mut self.faces);
                }
            }
        }
        Ok(())
    }

    /// The box of the bounds enlarged by half their diagonal; its faces are appended to the
    /// faces to split (`MakeBox`).
    fn make_box(&mut self) -> Result<BoxFaces, Failure> {
        let extent = self.bounds.square_extent().sqrt() * 0.5;
        self.bounds.enlarge(extent);
        let [xmin, ymin, zmin, xmax, ymax, zmax] = self.bounds.get()?;
        let mut maker = BRepPrimAPIMakeBox::from_points(
            Point3::new(xmin, ymin, zmin),
            Point3::new(xmax, ymax, zmax),
        )?;
        self.bounding_solid = maker.solid()?;
        let mut box_faces: BoxFaces = Map::with_hasher(ShapeMapHasher, 1);
        let mut explorer = Explorer::new(&self.bounding_solid, ShapeEnum::Face, ShapeEnum::Shape)?;
        while explorer.more() {
            self.faces.append(explorer.current().clone());
            box_faces.add(explorer.current().clone());
            explorer.next()?;
        }
        Ok(box_faces)
    }

    /// The solids the faces bound, or None after `AlertSolidBuilderFailed` (`BuildSolids`).
    fn build_solids(&mut self) -> Result<Option<List<Shape>>, Failure> {
        let mut builder = BuilderSolid::new();
        builder.set_shapes(&self.faces);
        builder
            .options_mut()
            .set_run_parallel(self.builder.options().run_parallel());
        builder.set_avoid_internal_shapes(self.avoid_internal_shapes);
        builder.perform()?;
        if builder.options().has_errors() {
            self.builder
                .options()
                .add_error(&alert(AlertSolidBuilderFailed));
            return Ok(None);
        }
        self.builder
            .options()
            .report()
            .merge(builder.options().report());
        Ok(Some(builder.areas().clone()))
    }

    /// The vertices and edges of the arguments (the wires as their edges) put inside the
    /// solids, unless internal shapes are avoided (`FillInternalShapes`).
    fn fill_internal_shapes(
        &self,
        solids: &List<Shape>,
        structure: &DataStructure,
    ) -> Result<(), Failure> {
        if self.avoid_internal_shapes {
            return Ok(());
        }
        let mut fence = MapOfShape::default();
        let mut leaves = Vec::new();
        for argument in structure.arguments().iter() {
            treat_compound(argument, &mut leaves, Some(&mut fence))?;
        }
        let parts = vertices_and_edges(&leaves, &mut fence)?;
        let context = self.builder.context.get().ok_or_else(missing_context)?;
        fill_internals(solids, &parts, &self.builder.images, context)
    }

    /// The single solid, or the compound prepared by `Prepare` holding all solids
    /// (`BuildShape`).
    fn build_shape(&mut self, solids: &List<Shape>) -> Result<(), Failure> {
        if solids.extent() == 1 {
            self.builder.shape = solids.first().cloned().unwrap_or_default();
            return Ok(());
        }
        let builder = BRepBuilder::default();
        for solid in solids.iter() {
            builder.add(&mut self.builder.shape, solid)?;
        }
        Ok(())
    }
}

/// Both orientations of a face (`AddFace`).
fn add_face(face: &Shape, faces: &mut List<Shape>) {
    faces.append(face.oriented(Orientation::Forward));
    faces.append(face.oriented(Orientation::Reversed));
}

/// Removes the first solid holding a face of the box (`RemoveBox`).
fn remove_box(solids: &mut List<Shape>, box_faces: &BoxFaces) -> Result<(), Failure> {
    let mut position = None;
    for (index, solid) in solids.iter().enumerate() {
        let mut explorer = Explorer::new(solid, ShapeEnum::Face, ShapeEnum::Shape)?;
        while explorer.more() && !box_faces.contains(explorer.current()) {
            explorer.next()?;
        }
        if explorer.more() {
            position = Some(index);
            break;
        }
    }
    if let Some(index) = position {
        *solids = solids
            .iter()
            .enumerate()
            .filter(|(position, _)| *position != index)
            .map(|(_, solid)| solid.clone())
            .collect();
    }
    Ok(())
}

/// The vertices and edges among `leaves`, a wire given as its new edges.
fn vertices_and_edges(leaves: &[Shape], fence: &mut MapOfShape) -> Result<List<Shape>, Failure> {
    let mut parts = List::default();
    for leaf in leaves {
        match leaf.shape_type()? {
            ShapeEnum::Wire => {
                for edge in ShapeIterator::new(leaf, true, true) {
                    if fence.add(edge.clone()) {
                        parts.append(edge);
                    }
                }
            }
            ShapeEnum::Vertex | ShapeEnum::Edge => {
                parts.append(leaf.clone());
            }
            _ => {}
        }
    }
    Ok(parts)
}

/// The failure of filling internal shapes without the filler context.
fn missing_context() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BOPAlgo_MakerVolume: the context is null; expected the context of the pave filler",
        "",
    )
}
