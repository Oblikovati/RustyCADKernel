//! Populate discrete topology using native geometric wire analysis and edge identities.
use crate::{
    imesh_data::{FaceHandle, MEMORY_BLOCK_SIZE_HUGE, MeshModel, Status},
    imesh_tools::ShapeVisitor,
};
use rustycad_tkbrep::{
    brep_tools,
    topexp::Explorer,
    topods::{self, Edge, Face, Shape, Wire},
    toptools::ShapeMapHasher,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{BaseAllocator, DataMap},
    precision,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysis, ShapeAnalysisWire, ShapeAnalysisWireOrder},
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
/// Native visitor building face/wire/pcurve incidence; `BRepMeshShapeVisitor::new(&model)`.
pub struct BRepMeshShapeVisitor {
    transient: StandardTransient,
    model: Handle<dyn MeshModel>,
    edge_indices: SharedState<DataMap<Shape, i32, ShapeMapHasher>>,
}
impl_transient!(BRepMeshShapeVisitor,"BRepMesh_ShapeVisitor",dyn ShapeVisitor,transient,interfaces:[dyn ShapeVisitor]);
impl BRepMeshShapeVisitor {
    /// Retain the model and allocate the native edge lookup arena; `BRepMeshShapeVisitor::new(&model)`.
    pub fn new(model: &Handle<dyn MeshModel>) -> Self {
        let indices = DataMap::with_allocator(
            ShapeMapHasher,
            1,
            BaseAllocator::incremental(MEMORY_BLOCK_SIZE_HUGE),
        );
        Self {
            transient: StandardTransient::default(),
            model: model.clone(),
            edge_indices: SharedState::new(indices),
        }
    }
    fn add_wire(&self, wire: &Wire, face: &FaceHandle) -> Result<bool, Failure> {
        if wire.is_null() {
            return Ok(false);
        }
        let wire_data = Handle::new(ShapeExtendWireData::from_wire(wire, true, false)?);
        let analysis = ShapeAnalysisWire::from_wire_data(
            &wire_data,
            &face.get_face()?,
            precision::confusion(),
        )?;
        let mut order = ShapeAnalysisWireOrder::new();
        analysis.check_order_with(&mut order, true, false, false)?;
        if analysis.last_check_status(ShapeExtendStatus::Fail) {
            return Ok(false);
        }
        if analysis.last_check_status(ShapeExtendStatus::Done3) {
            face.status().set_status(Status::UnorientedWire);
        }
        if order.nb_edges() != wire_data.nb_edges() {
            return Ok(false);
        }
        self.append_ordered_wire(wire, face, &wire_data, &order)?;
        Ok(true)
    }
    fn append_ordered_wire(
        &self,
        wire: &Wire,
        face: &FaceHandle,
        wire_data: &Handle<ShapeExtendWireData>,
        order: &ShapeAnalysisWireOrder,
    ) -> Result<(), Failure> {
        let discrete_wire = face.add_wire(wire, order.nb_edges());
        for index in 1..=order.nb_edges() {
            let edge = wire_data.edge(order.ordered(index)?)?;
            if edge.orientation() == Orientation::External {
                continue;
            }
            let edge_index = *self.edge_indices.read().find(&edge)?;
            let discrete_edge = self.model.get_edge(edge_index)?;
            discrete_edge.add_pcurve(face.downgrade(), edge.orientation());
            discrete_wire.add_edge(discrete_edge.downgrade(), edge.orientation());
        }
        Ok(())
    }
}
impl ShapeVisitor for BRepMeshShapeVisitor {
    fn visit_edge(&self, edge: &Edge) -> Result<(), Failure> {
        if self.edge_indices.read().is_bound(edge) {
            return Ok(());
        }
        self.model.add_edge(edge)?;
        self.edge_indices
            .write()
            .bind(edge.clone().into_shape(), self.model.edges_nb() - 1);
        Ok(())
    }
    fn visit_face(&self, face: &Face) -> Result<(), Failure> {
        brep_tools::update_face(face)?;
        let discrete_face = self.model.add_face(face)?;
        let outer = ShapeAnalysis::outer_wire(face)?;
        if !self.add_wire(&outer, &discrete_face)? {
            discrete_face.status().set_status(Status::Failure);
            return Ok(());
        }
        for candidate in Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            let wire = topods::wire(&candidate)?;
            if wire.is_same(&outer) {
                continue;
            }
            if !self.add_wire(&wire, &discrete_face)? {
                discrete_face.status().set_status(Status::UnorientedWire);
            }
        }
        Ok(())
    }
}
