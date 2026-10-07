//! `BOPAlgo_Builder::FillSameDomainFaces` and `FillInternalVertices` (BOPAlgo_Builder_2.cxx).
use super::images::append_listed;
use super::{BOPAlgoBuilder, SameDomainShapes, required_context};
use crate::bop_algo::tools::{ConnectionMap, fill_map, make_blocks};
use crate::bop_ds::DataStructure;
use crate::bop_tools::{ShapeSet, are_faces_same_domain};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape, Vertex};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;

/// Faces by their set of edges (`anESetFaces`).
type EdgeSetFaces = IndexedDataMap<ShapeSet, Vec<Shape>>;

impl BOPAlgoBuilder {
    /// Groups the splits of intersected faces with equal edge sets that are same-domain, and
    /// makes the face of minimal DS index (or the first split) the image of the group
    /// (`FillSameDomainFaces`); `builder.fill_same_domain_faces(&ds)?`.
    pub fn fill_same_domain_faces(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        if structure.interf_ff_records().is_empty() {
            return Ok(());
        }
        let parents = self.face_parents(structure)?;
        let handle = self.context.clone();
        let context = required_context(&handle)?;
        let (sets, planar) = self.edge_set_faces(structure, context)?;
        let mut links: ConnectionMap<Shape, Shape, ShapeMapHasher> =
            IndexedDataMap::with_hasher(ShapeMapHasher, 1);
        let mut pairs = Vec::new();
        for position in 1..=sets.extent() {
            let faces = sets.find_from_index(position)?;
            link_planar_pairs(faces, &planar, &parents, &mut links, &mut pairs);
        }
        let fuzzy = self.options().fuzzy_value();
        for (first, second) in pairs {
            let first_face = Face::unchecked(first.clone());
            if are_faces_same_domain(
                &first_face,
                &Face::unchecked(second.clone()),
                context,
                fuzzy,
            )? {
                fill_map(&mut links, first, second);
            }
        }
        for group in make_blocks(&links, ShapeMapHasher)? {
            self.bind_same_domain_group(structure, &group);
        }
        self.update_face_images_with_same_domain(structure)
    }

    /// The solid each source face, and each split of it, belongs to: two faces of one solid
    /// cannot be same-domain (`aFaceToParent`).
    fn face_parents(&self, structure: &DataStructure) -> Result<SameDomainShapes, Failure> {
        let mut parents: SameDomainShapes = DataMap::with_hasher(ShapeMapHasher, 1);
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Solid {
                continue;
            }
            for face in Explorer::new(info.shape(), ShapeEnum::Face, ShapeEnum::Shape)? {
                if !parents.is_bound(&face) {
                    parents.bind(face, info.shape().clone());
                }
            }
        }
        let mut propagation: SameDomainShapes = DataMap::with_hasher(ShapeMapHasher, 1);
        for (face, pieces) in self.images.items() {
            let Some(parent) = parents.seek(face) else {
                continue;
            };
            for piece in pieces.iter() {
                if !parents.is_bound(piece) {
                    propagation.bind(piece.clone(), parent.clone());
                }
            }
        }
        for (piece, parent) in propagation.items() {
            parents.bind(piece.clone(), parent.clone());
        }
        Ok(parents)
    }

    /// The splits (or the faces) of the intersected faces, sorted by DS index, by edge set;
    /// with the bounded planar ones.
    fn edge_set_faces(
        &self,
        structure: &DataStructure,
        context: &Context,
    ) -> Result<(EdgeSetFaces, Map<Shape, ShapeMapHasher>), Failure> {
        let mut sets: EdgeSetFaces = IndexedDataMap::new(1);
        let mut planar: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        for index in intersected_faces(structure)? {
            let info = structure.shape_info(index)?;
            let face = info.shape();
            let check_planar = context
                .surface_adaptor(&Face::unchecked(face.clone()))?
                .read()
                .get_type()?
                == SurfaceType::Plane
                && !is_open_box(info.box_bounds());
            let pieces: Vec<Shape> = match self.images.seek(face) {
                Some(pieces) => pieces.iter().cloned().collect(),
                None => vec![face.clone()],
            };
            for piece in pieces {
                add_edge_set(&piece, &mut sets)?;
                if check_planar {
                    planar.add(piece);
                }
            }
        }
        Ok((sets, planar))
    }

    /// The face of minimal DS index (else the first face) represents the group; original
    /// faces become their own image.
    fn bind_same_domain_group(&mut self, structure: &DataStructure, group: &[Shape]) {
        let mut representative: Option<&Shape> = None;
        let mut minimal = i32::MAX;
        for face in group {
            let index = structure.index(face);
            if index < 0 {
                continue;
            }
            // An original face has no split but a same-domain face: it is its own split.
            let mut image = List::default();
            image.append(face.clone());
            self.images.bound(face.clone(), image);
            if index < minimal {
                minimal = index;
                representative = Some(face);
            }
        }
        let representative = representative.unwrap_or(&group[0]).clone();
        for face in group {
            self.shapes_sd.bind(face.clone(), representative.clone());
        }
    }

    /// Replaces the face images by their same-domain face and fills the origins.
    fn update_face_images_with_same_domain(
        &mut self,
        structure: &DataStructure,
    ) -> Result<(), Failure> {
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Face {
                continue;
            }
            let face = info.shape().clone();
            let Some(images) = self.images.change_seek(&face) else {
                continue;
            };
            let mut updated = Vec::new();
            for image in images.iter_mut() {
                if let Some(same_domain) = self.shapes_sd.seek(image) {
                    *image = same_domain.clone();
                }
                updated.push(image.clone());
            }
            for image in updated {
                append_listed(&mut self.origins, &image, &face);
            }
        }
        Ok(())
    }

    /// Puts the alone vertices of each face into the splits that contain them, as INTERNAL
    /// vertices (`FillInternalVertices`); `builder.fill_internal_vertices(&ds)?`.
    pub fn fill_internal_vertices(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        let handle = self.context.clone();
        let context = required_context(&handle)?;
        let fuzzy = self.options().fuzzy_value();
        let mut pairs: Vec<(Shape, Shape)> = Vec::new();
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Face {
                continue;
            }
            let Some(splits) = self.images.seek(info.shape()) else {
                continue;
            };
            for vertex in structure.alone_vertices(index)? {
                let vertex = structure.shape(vertex)?.oriented(Orientation::Internal);
                for split in splits.iter() {
                    pairs.push((vertex.clone(), split.clone()));
                }
            }
        }
        // All pairs are classified before any vertex is added, as in the native parallel run.
        let mut inside = Vec::new();
        for (vertex, split) in pairs {
            let (mut u, mut v, mut tolerance) = (0., 0., 0.);
            let flag = context.compute_vf(
                &Vertex::unchecked(vertex.clone()),
                &Face::unchecked(split.clone()),
                &mut u,
                &mut v,
                &mut tolerance,
                fuzzy,
            )?;
            if flag == 0 {
                inside.push((vertex, split));
            }
        }
        let builder = BRepBuilder::default();
        for (vertex, mut split) in inside {
            builder.add(&mut split, &vertex)?;
        }
        Ok(())
    }
}

/// The intersected faces with face information, by increasing DS index.
fn intersected_faces(structure: &DataStructure) -> Result<Vec<i32>, Failure> {
    let mut fence: Map<i32> = Map::new(1);
    let mut faces = Vec::new();
    for record in structure.interf_ff_records().iter() {
        let (first, second) = record.indices();
        for face in [first, second] {
            if structure.has_face_info(face)? && fence.add(face) {
                faces.push(face);
            }
        }
    }
    faces.sort_unstable();
    Ok(faces)
}

fn is_open_box(bounds: &rustycad_tkmath::bnd::BoundingBox3) -> bool {
    bounds.is_open_xmin()
        || bounds.is_open_xmax()
        || bounds.is_open_ymin()
        || bounds.is_open_ymax()
        || bounds.is_open_zmin()
        || bounds.is_open_zmax()
}

/// Adds the face under its set of edges (`AddEdgeSet`).
fn add_edge_set(face: &Shape, sets: &mut EdgeSetFaces) -> Result<(), Failure> {
    let mut set = ShapeSet::new();
    set.add(face, ShapeEnum::Edge)?;
    if let Some(faces) = sets.change_seek(&set) {
        faces.push(face.clone());
        return Ok(());
    }
    sets.add(set, vec![face.clone()]);
    Ok(())
}

/// Links the pairs of bounded planar faces of one edge set, which are same-domain without
/// a check, and collects the other pairs for `AreFacesSameDomain`; faces of one solid are
/// never paired.
fn link_planar_pairs(
    faces: &[Shape],
    planar: &Map<Shape, ShapeMapHasher>,
    parents: &SameDomainShapes,
    links: &mut ConnectionMap<Shape, Shape, ShapeMapHasher>,
    pairs: &mut Vec<(Shape, Shape)>,
) {
    if faces.len() < 2 {
        return;
    }
    for (position, first) in faces.iter().enumerate() {
        let first_planar = planar.contains(first);
        let first_parent = parents.seek(first);
        for second in &faces[position + 1..] {
            if let (Some(a), Some(b)) = (first_parent, parents.seek(second))
                && a.is_same(b)
            {
                continue;
            }
            if first_planar && planar.contains(second) {
                fill_map(links, first.clone(), second.clone());
                continue;
            }
            pairs.push((first.clone(), second.clone()));
        }
    }
}
