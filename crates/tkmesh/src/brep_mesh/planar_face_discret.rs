//! Boundary-constrained analytic face meshing, following `BRepMesh_BaseMeshAlgo`'s data flow.
use super::{
    BRepMeshDataStructureOfDelaun, BRepMeshDelaun, BRepMeshEdge, BRepMeshVertex, ShapeTool,
};
use crate::brep_mesh::{DefaultRangeSplitter, GeomTool};
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
    geom_abs::SurfaceType,
    gp::{Point2, Xy},
    poly::{Triangle, Triangulation},
};
#[path = "planar_face_discret/interior_nodes.rs"]
mod interior_nodes;
#[path = "planar_face_discret/sphere_nodes.rs"]
mod sphere_nodes;

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

    fn discretize_face(face: &FaceHandle, parameters: &MeshParameters) -> Result<(), Failure> {
        if !matches!(
            face.get_surface().get_type()?,
            SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere
        ) {
            return Err(Failure::with_message(
                "Standard_NotImplemented",
                "BRepMesh boundary face strategy requires a plane, cylinder, cone, or sphere surface",
            ));
        }
        let allocator = Handle::new(IncrementalAllocatorObject::new(
            crate::imesh_data::MEMORY_BLOCK_SIZE_HUGE,
        ));
        let mesh = Handle::new(BRepMeshDataStructureOfDelaun::new(&allocator, 16));
        let splitter = Self::range_splitter(face, parameters)?;
        interior_nodes::configure_vertex_tool(&splitter, &mesh)?;
        let mut points = Vec::new();
        if parameters.internal_vertices_mode {
            interior_nodes::register_internal_vertices(face, &splitter, &mesh, &mut points)?;
        }
        Self::register_boundaries(face, &splitter, &mesh, &mut points)?;
        let mut vertices = (1..=mesh.nb_nodes()).collect::<Vec<_>>();
        let (cells_u, cells_v) = GeomTool::cells_count(
            &face.get_surface().cast(),
            mesh.nb_nodes(),
            face.get_deflection(),
            Some(&splitter),
        )?;
        let mut delaun = BRepMeshDelaun::from_mesh(&mesh, cells_u, cells_v, true)?;
        delaun.triangulate_vertices_probe(&mut vertices)?;
        let mut surface_nodes =
            sphere_nodes::register_interior(face, parameters, &splitter, &mesh, &mut points)?;
        if !surface_nodes.is_empty() {
            delaun.insert_vertices_probe(&mut surface_nodes)?;
        }
        Self::publish_mesh(face, &splitter, &mesh, &points)
    }

    fn range_splitter(
        face: &FaceHandle,
        parameters: &MeshParameters,
    ) -> Result<DefaultRangeSplitter, Failure> {
        let mut splitter = DefaultRangeSplitter::default();
        splitter.reset(face, parameters);
        Self::collect_splitter_boundary(face, &mut splitter)?;
        splitter.adjust_range()?;
        Self::scale_cylinder_range(face, parameters, &mut splitter)?;
        Ok(splitter)
    }

    fn collect_splitter_boundary(
        face: &FaceHandle,
        splitter: &mut DefaultRangeSplitter,
    ) -> Result<(), Failure> {
        for wire_index in 0..face.wires_nb() {
            let wire = face.get_wire(wire_index)?;
            for edge_index in 0..wire.edges_nb() {
                let edge = wire.get_edge(edge_index)?.upgrade();
                let orientation = wire.get_edge_orientation(edge_index)?;
                let pcurve = edge.get_pcurve_for(&face.downgrade(), orientation)?;
                Self::add_pcurve_bounds(splitter, &pcurve)?;
            }
        }
        Ok(())
    }

    fn add_pcurve_bounds(
        splitter: &mut DefaultRangeSplitter,
        pcurve: &crate::imesh_data::PCurveHandle,
    ) -> Result<(), Failure> {
        for sample in 0..pcurve.parameters_nb() {
            splitter.add_point(pcurve.get_point(sample)?);
        }
        Ok(())
    }

    fn scale_cylinder_range(
        face: &FaceHandle,
        parameters: &MeshParameters,
        splitter: &mut DefaultRangeSplitter,
    ) -> Result<(), Failure> {
        if face.get_surface().get_type()? != rustycad_tkmath::geom_abs::SurfaceType::Cylinder {
            return Ok(());
        }
        let radius = face.get_surface().cylinder()?.radius();
        let step = rustycad_tkgeombase::gcpnts::TangentialDeflection::arc_angular_step(
            radius,
            face.get_deflection(),
            parameters.angle,
            parameters.min_size,
        )?;
        let range_v = splitter.get_range_v();
        splitter.delta = (step / (range_v.1 - range_v.0), 1.0);
        Ok(())
    }

    fn register_boundaries(
        face: &FaceHandle,
        splitter: &DefaultRangeSplitter,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<(), Failure> {
        for wire_index in 0..face.wires_nb() {
            Self::register_wire(
                face,
                &face.get_wire(wire_index)?,
                wire_index,
                splitter,
                mesh,
                points,
            )?;
        }
        Ok(())
    }

    fn register_wire(
        face: &FaceHandle,
        wire: &Handle<dyn MeshWire>,
        wire_index: i32,
        splitter: &DefaultRangeSplitter,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<(), Failure> {
        for edge_index in 0..wire.edges_nb() {
            let edge = wire.get_edge(edge_index)?.upgrade();
            let orientation = wire.get_edge_orientation(edge_index)?;
            let pcurve = edge.get_pcurve_for(&face.downgrade(), orientation)?;
            let seam_orientation =
                Self::fix_seam_edge_orientation(&edge, &pcurve, &face.downgrade())?;
            Self::register_pcurve(
                &edge,
                &pcurve,
                seam_orientation,
                wire_index,
                splitter,
                mesh,
                points,
            )?;
            Self::register_other_pcurve_nodes(
                &edge,
                &pcurve,
                &face.downgrade(),
                splitter,
                mesh,
                points,
            )?;
        }
        Ok(())
    }

    fn fix_seam_edge_orientation(
        edge: &crate::imesh_data::EdgeHandle,
        pcurve: &crate::imesh_data::PCurveHandle,
        face: &crate::imesh_data::FacePtr,
    ) -> Result<Orientation, Failure> {
        for index in edge.get_pcurves(face)? {
            let candidate = edge.get_pcurve(index)?;
            if candidate.object_address() == pcurve.object_address() {
                continue;
            }
            if Self::pcurves_share_endpoints(pcurve, &candidate)? {
                return Ok(Orientation::Internal);
            }
        }
        Ok(pcurve.get_orientation())
    }

    fn pcurves_share_endpoints(
        first: &crate::imesh_data::PCurveHandle,
        second: &crate::imesh_data::PCurveHandle,
    ) -> Result<bool, Failure> {
        if first.parameters_nb() == 0 || second.parameters_nb() == 0 {
            return Ok(false);
        }
        let first_start = first.get_point(0)?;
        let first_end = first.get_point(first.parameters_nb() - 1)?;
        let second_start = second.get_point(0)?;
        let second_end = second.get_point(second.parameters_nb() - 1)?;
        Ok(endpoints_share(
            first_start,
            first_end,
            second_start,
            second_end,
        ))
    }

    fn register_pcurve(
        edge: &crate::imesh_data::EdgeHandle,
        pcurve: &crate::imesh_data::PCurveHandle,
        orientation: Orientation,
        wire_index: i32,
        splitter: &DefaultRangeSplitter,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<(), Failure> {
        let curve = edge.get_curve();
        let mut previous = None;
        for sample in 0..pcurve.parameters_nb() {
            let node = Self::register_sample(&curve, pcurve, sample, splitter, mesh, points)?;
            pcurve.set_index(sample, node)?;
            if let Some(first) = previous {
                Self::register_link(mesh, first, node, orientation, wire_index)?;
            }
            previous = Some(node);
        }
        Ok(())
    }

    fn register_other_pcurve_nodes(
        edge: &crate::imesh_data::EdgeHandle,
        selected: &crate::imesh_data::PCurveHandle,
        face: &crate::imesh_data::FacePtr,
        splitter: &DefaultRangeSplitter,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<(), Failure> {
        let curve = edge.get_curve();
        // OCCT indexes same-face alternatives for polygon output without constraining them.
        for index in edge.get_pcurves(face)? {
            let pcurve = edge.get_pcurve(index)?;
            if pcurve.object_address() == selected.object_address() {
                continue;
            }
            Self::register_pcurve_nodes(&curve, &pcurve, splitter, mesh, points)?;
        }
        Ok(())
    }

    fn register_pcurve_nodes(
        curve: &crate::imesh_data::CurveHandle,
        pcurve: &crate::imesh_data::PCurveHandle,
        splitter: &DefaultRangeSplitter,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<(), Failure> {
        for sample in 0..pcurve.parameters_nb() {
            let node = Self::register_sample(curve, pcurve, sample, splitter, mesh, points)?;
            pcurve.set_index(sample, node)?;
        }
        Ok(())
    }

    fn register_sample(
        curve: &crate::imesh_data::CurveHandle,
        pcurve: &crate::imesh_data::PCurveHandle,
        sample: i32,
        splitter: &DefaultRangeSplitter,
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        points: &mut Vec<rustycad_tkmath::gp::Point3>,
    ) -> Result<i32, Failure> {
        let point_3d = curve.get_point(sample)?;
        let point_2d = splitter.scale(pcurve.get_point(sample)?, true);
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
        splitter: &DefaultRangeSplitter,
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
        Self::copy_nodes(mesh, splitter, points, &triangulation)?;
        Self::copy_triangles(mesh, &elements, &triangulation)?;
        ShapeTool::add_in_face(&face.get_face()?, &triangulation)
    }

    fn copy_nodes(
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        splitter: &DefaultRangeSplitter,
        points: &[rustycad_tkmath::gp::Point3],
        output: &Handle<Triangulation>,
    ) -> Result<(), Failure> {
        for index in 1..=points.len() as i32 {
            let vertex = mesh.get_node(index)?;
            let uv = vertex.coord();
            output.set_node(index, points[vertex.location_3d() as usize - 1])?;
            output.set_uv_node(index, splitter.scale(Point2::new(uv.x(), uv.y()), false))?;
        }
        Ok(())
    }

    fn copy_triangles(
        mesh: &Handle<BRepMeshDataStructureOfDelaun>,
        elements: &rustycad_tkernel::ncollection::Map<i32>,
        output: &Handle<Triangulation>,
    ) -> Result<(), Failure> {
        let mut element_ids: Vec<i32> = elements.iter().copied().collect();
        element_ids.sort_unstable();
        for (offset, element_id) in element_ids.into_iter().enumerate() {
            let element = mesh.get_element(element_id)?;
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
            Self::discretize_face(&model.get_face(index)?, _parameters)?;
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

fn endpoint_distance(point: &Point2, first: &Point2, second: &Point2) -> f64 {
    point
        .square_distance(*first)
        .min(point.square_distance(*second))
}

fn endpoints_share(
    first_start: Point2,
    first_end: Point2,
    second_start: Point2,
    second_end: Point2,
) -> bool {
    let tolerance = rustycad_tkernel::precision::square_confusion();
    endpoint_distance(&first_start, &second_start, &second_end) < tolerance
        && endpoint_distance(&first_end, &second_start, &second_end) < tolerance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        brep_mesh::{BRepMeshEdgeDiscret, BRepMeshModelBuilder},
        imesh_data::Status,
        imesh_tools::{ModelAlgo, ModelBuilder},
    };
    use rustycad_tkernel::message::ProgressRange;
    use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

    #[test]
    fn box_face_statuses_remain_native_no_error() {
        let shape = box_shape();
        let parameters = MeshParameters {
            deflection: 0.5,
            min_size: 0.05,
            ..Default::default()
        };
        let model = BRepMeshModelBuilder::new().perform(&shape, &parameters);
        assert!(BRepMeshEdgeDiscret::new().perform(&model, &parameters, &ProgressRange::default()));
        assert!(
            BRepMeshPlanarFaceDiscret::new()
                .perform_internal(&model, &parameters, &ProgressRange::default())
                .unwrap()
        );
        for index in 0..model.faces_nb() {
            assert!(
                model
                    .get_face(index)
                    .unwrap()
                    .status()
                    .is_equal(Status::NoError)
            );
        }
    }

    #[test]
    fn seam_endpoints_match_in_either_direction() {
        let first = Point2::new(0.0, 1.0);
        let second = Point2::new(4.0, 1.0);
        assert!(endpoints_share(first, second, second, first));
        assert!(!endpoints_share(
            first,
            second,
            Point2::new(8.0, 1.0),
            Point2::new(12.0, 1.0)
        ));
    }

    fn box_shape() -> rustycad_tkbrep::topods::Shape {
        BRepPrimAPIMakeBox::from_sizes(10.0, 10.0, 10.0)
            .unwrap()
            .shape()
            .unwrap()
            .clone()
    }
}
