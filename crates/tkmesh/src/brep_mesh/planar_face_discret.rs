//! Boundary constrained plane meshing, matching `BRepMesh_BaseMeshAlgo`'s data flow.
use super::{
    BRepMeshDataStructureOfDelaun, BRepMeshDelaun, BRepMeshEdge, BRepMeshVertex, ShapeTool,
};
use crate::{
    imesh_data::{FaceHandle, MeshModel, MeshWire},
    imesh_tools::{MeshParameters, ModelAlgo},
};
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    ncollection::IncrementalAllocatorObject,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::Orientation};
use rustycad_tkmath::{
    gp::{Point2, Xy},
    poly::{Triangle, Triangulation},
};

/// Mesh planar faces from registered boundary samples; `BRepMeshPlanarFaceDiscret::new()`.
#[derive(Default)]
pub(super) struct BRepMeshPlanarFaceDiscret {
    transient: StandardTransient,
}
impl_transient!(BRepMeshPlanarFaceDiscret, "BRepMesh_PlanarFaceDiscret", dyn ModelAlgo, transient, interfaces: [dyn ModelAlgo]);

impl BRepMeshPlanarFaceDiscret {
    /// Create the constrained plane strategy; `BRepMeshPlanarFaceDiscret::new()`.
    pub(super) fn new() -> Self {
        Self::default()
    }

    fn discretize_face(face: &FaceHandle) -> Result<(), Failure> {
        if face.get_surface().get_type()? != rustycad_tkmath::geom_abs::SurfaceType::Plane {
            return Err(Failure::with_message(
                "Standard_NotImplemented",
                "BRepMesh planar face strategy requires a plane surface",
            ));
        }
        let allocator = Handle::new(IncrementalAllocatorObject::new(
            crate::imesh_data::MEMORY_BLOCK_SIZE_HUGE,
        ));
        let mesh = Handle::new(BRepMeshDataStructureOfDelaun::new(&allocator, 16));
        let mut points = Vec::new();
        Self::register_boundaries(face, &mesh, &mut points)?;
        let mut vertices = (1..=mesh.nb_nodes()).collect::<Vec<_>>();
        BRepMeshDelaun::from_mesh(&mesh, -1, -1, true)?
            .triangulate_vertices_probe(&mut vertices)?;
        Self::publish_mesh(face, &mesh, &points)
    }

    fn register_boundaries(
        face: &FaceHandle,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<(), Failure> {
        for wire_index in 0..face.wires_nb() {
            Self::register_wire(face, &face.get_wire(wire_index)?, wire_index, mesh, points)?;
        }
        Ok(())
    }

    fn register_wire(
        face: &FaceHandle,
        wire: &Handle<dyn MeshWire>,
        wire_index: i32,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<(), Failure> {
        for edge_index in 0..wire.edges_nb() {
            let edge = wire.get_edge(edge_index)?.upgrade();
            let orientation = wire.get_edge_orientation(edge_index)?;
            let pcurve = edge.get_pcurve_for(&face.downgrade(), orientation)?;
            Self::register_pcurve(&edge, &pcurve, wire_index, mesh, points)?;
        }
        Ok(())
    }

    fn register_pcurve(
        edge: &crate::imesh_data::EdgeHandle,
        pcurve: &crate::imesh_data::PCurveHandle,
        wire_index: i32,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<(), Failure> {
        let curve = edge.get_curve();
        let mut previous = None;
        for sample in 0..pcurve.parameters_nb() {
            let node = Self::register_sample(&curve, pcurve, sample, mesh, points)?;
            pcurve.set_index(sample, node)?;
            if let Some(first) = previous {
                Self::register_link(mesh, first, node, pcurve.get_orientation(), wire_index)?;
            }
            previous = Some(node);
        }
        Ok(())
    }

    fn register_sample(
        curve: &crate::imesh_data::CurveHandle,
        pcurve: &crate::imesh_data::PCurveHandle,
        sample: i32,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<i32, Failure> {
        let point_3d = curve.get_point(sample)?;
        let point_2d = pcurve.get_point(sample)?;
        let location = points.len() as i32 + 1;
        let vertex = BRepMeshVertex::new(
            Xy::new(point_2d.x(), point_2d.y()),
            location,
            super::DegreeOfFreedom::Frontier,
        );
        let index = mesh.add_node(&vertex, false)?;
        if index == location {
            points.push(point_3d);
        }
        Ok(index)
    }

    fn register_link(
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        first: i32,
        last: i32,
        orientation: Orientation,
        wire_index: i32,
    ) -> Result<(), Failure> {
        let prior_links = mesh.nb_links();
        let (first, last, freedom) = link_order(first, last, orientation);
        let link = mesh.add_link(&BRepMeshEdge::new(first, last, freedom))?;
        if wire_index > 0 && link.abs() <= prior_links {
            let mut duplicate = mesh.get_link(link.abs())?;
            duplicate.set_movability(super::DegreeOfFreedom::Fixed);
            mesh.substitute_link(link.abs(), &duplicate)?;
        }
        Ok(())
    }

    fn publish_mesh(
        face: &FaceHandle,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &[rustycad_tkmath::gp::Point3],
    ) -> Result<(), Failure> {
        let elements = mesh.elements_of_domain();
        if elements.is_empty() {
            face.status().set_status(crate::imesh_data::Status::Failure);
            return Ok(());
        }
        let triangulation = Handle::new(Triangulation::with_sizes(
            points.len() as i32,
            elements.extent() as i32,
            true,
            false,
        )?);
        Self::copy_nodes(mesh, points, &triangulation)?;
        Self::copy_triangles(mesh, &elements, &triangulation)?;
        ShapeTool::add_in_face(&face.get_face()?, &triangulation)
    }

    fn copy_nodes(
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &[rustycad_tkmath::gp::Point3],
        output: &Handle<Triangulation>,
    ) -> Result<(), Failure> {
        for index in 1..=points.len() as i32 {
            let vertex = mesh.get_node(index)?;
            let uv = vertex.coord();
            output.set_node(index, points[vertex.location_3d() as usize - 1])?;
            output.set_uv_node(index, Point2::new(uv.x(), uv.y()))?;
        }
        Ok(())
    }

    fn copy_triangles(
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        elements: &rustycad_tkernel::ncollection::Map<i32>,
        output: &Handle<Triangulation>,
    ) -> Result<(), Failure> {
        for (offset, element_id) in elements.iter().enumerate() {
            let element = mesh.get_element(*element_id)?;
            let nodes = mesh.element_nodes(&element)?;
            output.set_triangle(
                offset as i32 + 1,
                Triangle::new(nodes[0], nodes[1], nodes[2]),
            )?;
        }
        Ok(())
    }
}

impl ModelAlgo for BRepMeshPlanarFaceDiscret {
    fn perform_internal(
        &self,
        model: &Handle<dyn MeshModel>,
        _parameters: &MeshParameters,
        _range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        for index in 0..model.faces_nb() {
            Self::discretize_face(&model.get_face(index)?)?;
        }
        Ok(true)
    }
}

fn link_order(
    first: i32,
    last: i32,
    orientation: Orientation,
) -> (i32, i32, super::DegreeOfFreedom) {
    match orientation {
        Orientation::Reversed => (last, first, super::DegreeOfFreedom::Frontier),
        Orientation::Internal => (first, last, super::DegreeOfFreedom::Fixed),
        _ => (first, last, super::DegreeOfFreedom::Frontier),
    }
}
