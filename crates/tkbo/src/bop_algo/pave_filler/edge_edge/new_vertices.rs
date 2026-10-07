//! `BOPAlgo_PaveFiller::PerformNewVertices` and `TreatNewVertices`.
use super::super::{BOPAlgoPaveFiller, missing_structure};
use super::VertexCouples;
use crate::bop_algo::tools::{fill_map_one, intersect_vertices, pave_block_index_links};
use crate::bop_ds::ShapeInfo;
use crate::bop_tools::make_vertex;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

impl BOPAlgoPaveFiller {
    /// Fuses the new intersection vertices, appends them to the DS (setting the new index of
    /// their EE or EF interferences) and puts them on their pave blocks through IntersectVE.
    /// `filler.perform_new_vertices(&mut couples, true)?`.
    pub(in crate::bop_algo::pave_filler) fn perform_new_vertices(
        &mut self,
        couples: &mut VertexCouples,
        edge_edge: bool,
    ) -> Result<(), Failure> {
        if couples.extent() == 0 {
            return Ok(());
        }
        let additional = self.options.fuzzy_value() / 2.;
        for (vertex, sources) in self.treat_new_vertices(couples)? {
            let index = self.append_new_vertex(&vertex, additional)?;
            for source in sources {
                let couple = couples.change_from_key(&source)?;
                couple.set_index(index);
                let interference = couple.interference_index() as usize;
                let structure = self.structure_mut()?;
                if edge_edge {
                    structure.interf_ee()[interference].set_index_new(index);
                } else {
                    structure.interf_ef()[interference].set_index_new(index);
                }
            }
        }
        let mut block_vertices = pave_block_index_links();
        for position in 1..=couples.extent() {
            let couple = couples.find_from_index(position)?;
            let (first, second) = (couple.pave_block1(), couple.pave_block2());
            let (Some(first), Some(second)) = (first, second) else {
                continue;
            };
            fill_map_one(&mut block_vertices, Arc::clone(first), couple.index());
            if !Arc::ptr_eq(first, second) {
                fill_map_one(&mut block_vertices, Arc::clone(second), couple.index());
            }
        }
        self.intersect_ve(&block_vertices, false)
    }

    /// Groups the new vertices whose tolerances interfere and makes one vertex per group;
    /// returns each made vertex with the source vertices it replaces (`TreatNewVertices`).
    fn treat_new_vertices(
        &self,
        couples: &VertexCouples,
    ) -> Result<Vec<(Vertex, Vec<Shape>)>, Failure> {
        let mut vertices: Vec<(Shape, f64)> = Vec::with_capacity(couples.extent());
        for position in 1..=couples.extent() {
            let shape = couples.find_key(position)?.clone();
            vertices.push((shape, couples.find_from_index(position)?.tolerance()));
        }
        let mut images = Vec::new();
        for chain in intersect_vertices(&vertices, self.options.fuzzy_value())? {
            let list: List<Shape> = chain.iter().cloned().collect();
            let mut vertex = Vertex::default();
            make_vertex(&list, &mut vertex)?;
            images.push((vertex, chain));
        }
        Ok(images)
    }

    fn append_new_vertex(&mut self, vertex: &Vertex, additional: f64) -> Result<i32, Failure> {
        let structure = self.data_structure.as_mut().ok_or_else(missing_structure)?;
        let mut info = ShapeInfo::default();
        info.set_shape_type(ShapeEnum::Vertex);
        info.set_shape(&vertex.clone().into_shape());
        let index = structure.append_shape_info(&info);
        let point = BRepTool::pnt(vertex)?;
        let gap = BRepTool::vertex_tolerance(vertex)? + additional;
        let bounds = structure.change_shape_info(index)?.change_box();
        bounds.add_point(point);
        bounds.set_gap(gap);
        Ok(index)
    }
}
