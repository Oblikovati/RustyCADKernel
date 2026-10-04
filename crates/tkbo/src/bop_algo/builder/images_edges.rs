use super::BOPAlgoBuilder;
use crate::bop_ds::{DataStructure, PaveBlockHandle};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{ncollection::List, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BOPAlgoBuilder {
    /// Builds edge images and their origins from prepared DS pave blocks.
    /// `builder.fill_images_edges(&structure)?` records each source edge's real blocks.
    pub fn fill_images_edges(&mut self, structure: &DataStructure) -> Result<(), Failure> {
        for index in 0..structure.nb_source_shapes() {
            if !is_referenced_edge(structure, index)? {
                continue;
            }
            self.append_edge_images(structure, index)?;
        }
        Ok(())
    }

    fn append_edge_images(&mut self, structure: &DataStructure, index: i32) -> Result<(), Failure> {
        let original = structure.shape(index)?.clone();
        self.images.try_bound(original.clone(), List::default());
        for block in structure.pave_blocks(index)? {
            self.append_edge_block_image(structure, &original, block)?;
        }
        Ok(())
    }

    fn append_edge_block_image(
        &mut self,
        structure: &DataStructure,
        original: &Shape,
        block: &PaveBlockHandle,
    ) -> Result<(), Failure> {
        let real = structure.real_pave_block(block)?;
        let image = structure.shape(pave_block_edge(&real))?.clone();
        self.append_image(original, &image);
        self.append_origin(&image, original);
        self.bind_common_block_edge_image(structure, block, &image)?;
        Ok(())
    }

    fn bind_common_block_edge_image(
        &mut self,
        structure: &DataStructure,
        block: &PaveBlockHandle,
        image: &Shape,
    ) -> Result<(), Failure> {
        if !structure.is_common_block_on_edge(block) {
            return Ok(());
        }
        let split = structure.shape(pave_block_edge(block))?.clone();
        self.shapes_sd.bind(split, image.clone());
        Ok(())
    }
}

fn is_referenced_edge(structure: &DataStructure, index: i32) -> Result<bool, Failure> {
    let info = structure.shape_info(index)?;
    Ok(info.shape_type() == ShapeEnum::Edge && info.has_reference())
}

fn pave_block_edge(block: &PaveBlockHandle) -> i32 {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .edge()
}
