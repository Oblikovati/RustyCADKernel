use super::BOPAlgoBuilder;
use crate::bop_ds::DataStructure;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{ncollection::List, standard::Failure};

impl BOPAlgoBuilder {
    /// Builds vertex image, same-domain, and origin maps from prepared DS links.
    /// `builder.fill_images_vertices(&data_structure)?`.
    pub fn fill_images_vertices(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        for (source_index, representative_index) in structure.shape_sd_items() {
            self.append_vertex_image(structure, *source_index, *representative_index)?;
        }
        Ok(())
    }

    fn append_vertex_image(
        &mut self,
        structure: &DataStructure,
        source_index: i32,
        representative_index: i32,
    ) -> Result<(), Failure> {
        let source = structure.shape(source_index)?.clone();
        let representative = structure.shape(representative_index)?.clone();
        self.append_image(&source, &representative);
        self.shapes_sd.bind(source.clone(), representative.clone());
        self.append_origin(&representative, &source);
        Ok(())
    }

    fn append_image(&mut self, source: &Shape, representative: &Shape) {
        self.images
            .try_bound(source.clone(), List::default())
            .append(representative.clone());
    }

    fn append_origin(&mut self, representative: &Shape, source: &Shape) {
        self.origins
            .try_bound(representative.clone(), List::default())
            .append(source.clone());
    }

    /// Borrows split images grouped by original shape identity.
    /// `builder.images().find(&source)`.
    pub fn images(
        &self,
    ) -> &rustycad_tkernel::ncollection::DataMap<
        Shape,
        List<Shape>,
        rustycad_tkbrep::toptools::ShapeMapHasher,
    > {
        &self.images
    }

    /// Borrows same-domain shape representatives by original shape identity.
    /// `builder.shapes_sd().find(&source)`.
    pub fn shapes_sd(
        &self,
    ) -> &rustycad_tkernel::ncollection::DataMap<
        Shape,
        Shape,
        rustycad_tkbrep::toptools::ShapeMapHasher,
    > {
        &self.shapes_sd
    }

    /// Borrows origin lists grouped by resulting shape identity.
    /// `builder.origins().find(&representative)`.
    pub fn origins(
        &self,
    ) -> &rustycad_tkernel::ncollection::DataMap<
        Shape,
        List<Shape>,
        rustycad_tkbrep::toptools::ShapeMapHasher,
    > {
        &self.origins
    }
}
