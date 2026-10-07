//! `BOPAlgo_Builder::FillImagesVertices` (BOPAlgo_Builder_1.cxx).
use super::{BOPAlgoBuilder, ShapeImages};
use crate::bop_ds::DataStructure;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{ncollection::List, standard::Failure};

impl BOPAlgoBuilder {
    /// Each vertex of a same-domain group gets its representative as image, same-domain
    /// shape and origin (`FillImagesVertices`); `builder.fill_images_vertices(&structure)?`.
    pub fn fill_images_vertices(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        for (source, representative) in structure.shape_sd_items() {
            let vertex = structure.shape(*source)?.clone();
            let representative = structure.shape(*representative)?.clone();
            let mut image = List::default();
            image.append(representative.clone());
            self.images.bound(vertex.clone(), image);
            self.shapes_sd.bind(vertex.clone(), representative.clone());
            append_listed(&mut self.origins, &representative, &vertex);
        }
        Ok(())
    }
}

/// Appends `value` to the list of `key`, binding an empty list first.
pub(super) fn append_listed(map: &mut ShapeImages, key: &Shape, value: &Shape) {
    if let Some(list) = map.change_seek(key) {
        list.append(value.clone());
        return;
    }
    map.bound(key.clone(), List::default())
        .append(value.clone());
}
