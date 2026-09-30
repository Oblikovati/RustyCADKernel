use super::*;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools,
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkernel::{
    message::ProgressRange,
    osd::{FileSystem, default_file_system},
    tcollection::AsciiString,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
impl BRepMeshDataStructureOfDelaun {
    /// Write the debug compound, swallowing native write failures; `mesh.dump("mesh.brep")`.
    pub fn dump(&self, filename: &str) {
        self.dump_on(&*default_file_system(), filename);
    }
    /// Inject debug output storage while retaining failure swallowing; `mesh.dump_on(&filesystem,"mesh.brep")`.
    pub fn dump_on(&self, filesystem: &dyn FileSystem, filename: &str) {
        let _ = self.dump_shape().and_then(|shape| {
            brep_tools::write_file_on(
                filesystem,
                &shape,
                &AsciiString::from(filename),
                brep_tools::BRepWriteOptions::default(),
                &ProgressRange::default(),
            )
        });
    }
    fn dump_shape(&self) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut compound = Compound::default();
        builder.make_compound(&mut compound);
        let domain = self.links_of_domain();
        if domain.is_empty() {
            self.dump_vertices(&builder, &mut compound)?;
        } else {
            for index in domain.iter() {
                self.dump_link(&builder, &mut compound, *index)?;
            }
        }
        Ok(compound.into())
    }
    fn dump_vertices(&self, builder: &BRepBuilder, compound: &mut Compound) -> Result<(), Failure> {
        for index in 1..=self.nb_nodes() {
            let mut vertex = Vertex::default();
            builder.make_vertex_at(
                &mut vertex,
                self.dump_point(index)?,
                rustycad_tkernel::precision::confusion(),
            )?;
            builder.add(compound, &vertex)?;
        }
        Ok(())
    }
    fn dump_link(
        &self,
        builder: &BRepBuilder,
        compound: &mut Compound,
        index: i32,
    ) -> Result<(), Failure> {
        let edge = self.get_link(index)?;
        let first = self.dump_point(edge.first_node())?;
        let last = self.dump_point(edge.last_node())?;
        if first.square_distance(&last) < rustycad_tkernel::precision::square_confusion() {
            return Ok(());
        }
        let maker = BRepBuilderAPIMakeEdge::from_points(first, last)?;
        builder.add(compound, &maker.edge()?.into())
    }
    fn dump_point(&self, index: i32) -> Result<Point3, Failure> {
        let vertex = self.get_node(index)?;
        Ok(Point3::new(vertex.coord().x(), vertex.coord().y(), 0.))
    }
}
