//! `BOPAlgo_Builder::FillImagesEdges` (BOPAlgo_Builder_1.cxx).
use super::BOPAlgoBuilder;
use super::images::append_listed;
use crate::bop_ds::DataStructure;
use rustycad_tkernel::{ncollection::List, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BOPAlgoBuilder {
    /// Each referenced source edge gets the edges of the real pave blocks of its pave blocks
    /// as images; small edges without pave blocks get an empty list and leave the result
    /// (`FillImagesEdges`); `builder.fill_images_edges(&structure)?`.
    pub fn fill_images_edges(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Edge || !info.has_reference() {
                continue;
            }
            let edge = info.shape().clone();
            self.images.bound(edge.clone(), List::default());
            for block in structure.pave_blocks(index)? {
                let real = structure.real_pave_block(block)?;
                let split = structure
                    .shape(real.lock().expect("pave-block lock").edge())?
                    .clone();
                if let Some(images) = self.images.change_seek(&edge) {
                    images.append(split.clone());
                }
                append_listed(&mut self.origins, &split, &edge);
                if structure.is_common_block_on_edge(block) {
                    let own = structure.shape(block.lock().expect("pave-block lock").edge())?;
                    self.shapes_sd.bind(own.clone(), split);
                }
            }
        }
        Ok(())
    }
}
