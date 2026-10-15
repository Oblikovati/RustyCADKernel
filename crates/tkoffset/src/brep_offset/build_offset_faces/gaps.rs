// Rust translation of BRepOffset_BuildOffsetFaces::FillGaps and FillHistory of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The holes of the splits filled by new splits, and the history of the faces and edges.
use super::{BuildOffsetFaces, ShapeShapeMap, build_splits_of_face};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape};
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl BuildOffsetFaces<'_> {
    /// `FillGaps(theRange)`: the splits of each face completed by the new splits
    /// containing its free edges (reversed), so that closed volumes can be built.
    pub(super) fn fill_gaps(&mut self) -> Result<(), Failure> {
        let nb_f = self.my_of_images.extent();
        if nb_f == 0 {
            return Ok(());
        }

        // Check the splits of offset faces on the free edges and fill the gaps (holes)
        // in created splits, otherwise the closed volume will not be possible to create.

        // Map the splits of faces to find free edges
        let mut an_ef_map = IndexedDataMapOfShapeListOfShape::default();
        for i in 1..=nb_f {
            for f in self.my_of_images.find_from_index(i)?.iter() {
                topexp::map_shapes_and_ancestors(
                    f,
                    ShapeEnum::Edge,
                    ShapeEnum::Face,
                    &mut an_ef_map,
                )?;
            }
        }

        // Analyze images of each offset face on the presence of free edges
        // and try to fill the holes
        for i in 1..=nb_f {
            let a_lf_images = self.my_of_images.find_from_index(i)?.clone();
            if a_lf_images.is_empty() {
                continue;
            }

            // Collect all edges from the splits
            let b = BRepBuilder::default();
            let mut an_edges = Compound::default();
            b.make_compound(&mut an_edges);

            // Collect all free edges into a map with reverted orientation
            // (the native map uses the default hasher: orientation matters)
            let mut a_free_edges_map: Map<Shape> = Map::default();
            for f_im in a_lf_images.iter() {
                for e in Explorer::new(f_im, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if e.orientation() != Orientation::Forward
                        && e.orientation() != Orientation::Reversed
                    {
                        // Skip internals
                        continue;
                    }
                    if an_ef_map.find_from_key(&e)?.len() == 1 {
                        a_free_edges_map.add(e.reversed());
                    }
                    b.add(&mut an_edges, &e)?;
                }
            }

            if a_free_edges_map.is_empty() {
                // No free edges
                continue;
            }

            // Free edges are found - fill the gaps by creating new splits
            // of the face using these free edges
            let a_f = topods::face(self.my_of_images.find_key(i)?)?;

            // Build new splits using all kept edges and among new splits
            // find those containing free edges
            let mut a_lf_new = List::default();
            let mut a_dummy = ShapeShapeMap::default();
            build_splits_of_face(&a_f, &an_edges, &mut a_dummy, &mut a_lf_new)?;

            // Find faces filling holes
            let images = self.my_of_images.change_from_index(i)?;
            for f_new in a_lf_new.iter() {
                for e in Explorer::new(f_new, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    if a_free_edges_map.contains(&e) {
                        // Add face to splits
                        images.append(f_new.clone());
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// `FillHistory()`: the splits of the faces and the kept images of the offset edges
    /// recorded in the history.
    pub(super) fn fill_history(&mut self) -> Result<(), Failure> {
        let nb_f = self.my_of_images.extent();
        if nb_f == 0 {
            return Ok(());
        }

        // Map of kept edges
        let mut an_edges_map = IndexedMapOfShape::default();

        // Fill history for faces
        for i in 1..=nb_f {
            let a_lf_images = self.my_of_images.find_from_index(i)?.clone();
            if a_lf_images.is_empty() {
                continue;
            }

            // Add the splits to history map
            let a_f = self.my_of_images.find_key(i)?.clone();
            if self.my_image.has_image(&a_f) {
                self.my_image
                    .add_many(&a_f, &a_lf_images.iter().cloned().collect::<Vec<_>>())?;
            } else {
                self.my_image
                    .bind_many(&a_f, &a_lf_images.iter().cloned().collect::<Vec<_>>())?;
            }

            // Collect edges from splits
            for f_im in a_lf_images.iter() {
                topexp::map_shapes_of_type(f_im, ShapeEnum::Edge, &mut an_edges_map)?;
            }
        }

        // Fill history for edges (iteration by the map is safe because the
        // order is not important here)
        let images: Vec<(Shape, List<Shape>)> = self
            .my_oe_images
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (e, le_im) in &images {
            let mut b_has_image = self.my_image.has_image(e);
            for e_im in le_im.iter() {
                if an_edges_map.contains(e_im) {
                    if b_has_image {
                        self.my_image.add(e, e_im)?;
                    } else {
                        self.my_image.bind(e, e_im)?;
                        b_has_image = true;
                    }
                }
            }
        }
        Ok(())
    }
}
