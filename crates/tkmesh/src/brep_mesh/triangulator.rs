// Rust translation of BRepMesh_Triangulator.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMesh_Triangulator`: triangles of planar polygons given by 3d nodes.
use super::{
    BRepMeshDataStructureOfDelaun, BRepMeshDelaun, BRepMeshEdge, BRepMeshVertex, DegreeOfFreedom,
};
use rustycad_tkernel::message::{Gravity, Message, Messenger};
use rustycad_tkernel::ncollection::{
    Array1, DataMap, DefaultHashPolicy, IncrementalAllocatorObject,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::proj_lib::project_point_on_plane;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3, Xyz};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use std::sync::Arc;

/// `BRepMesh_Triangulator(theXYZs, theWires, theNorm)`: the wires are 0-based indices
/// into the nodes; the output triangles use 1-based node indices.
///
/// ```
/// use rustycad_tkmath::gp::{Direction3, Xyz};
/// use rustycad_tkmesh::brep_mesh::BRepMeshTriangulator;
/// let nodes = [Xyz::new(0.0, 0.0, 0.0), Xyz::new(1.0, 0.0, 0.0), Xyz::new(0.0, 1.0, 0.0)];
/// let wires = [vec![0, 1, 2]];
/// let mut triangulator = BRepMeshTriangulator::new(&nodes, &wires, Direction3::new(0.0, 0.0, 1.0)?)?;
/// let mut triangles = Vec::new();
/// assert!(triangulator.perform(&mut triangles)?);
/// assert_eq!(triangles.len(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepMeshTriangulator<'a> {
    xyzs: &'a [Xyz],
    wires: &'a [Vec<i32>],
    plane: Plane3,
    messenger: Option<Arc<Messenger>>,
    mesh_structure: Handle<BRepMeshDataStructureOfDelaun>,
    tmp_map: DataMap<i32, i32>,
    indices: Vec<i32>,
}

impl<'a> BRepMeshTriangulator<'a> {
    /// `BRepMesh_Triangulator(theXYZs, theWires, theNorm)`: the projection plane
    /// through the origin; `BRepMeshTriangulator::new(&nodes, &wires, normal)?`.
    pub fn new(
        xyzs: &'a [Xyz],
        wires: &'a [Vec<i32>],
        normal: Direction3,
    ) -> Result<Self, Failure> {
        Ok(Self {
            xyzs,
            wires,
            plane: Plane3::new(Point3::default(), normal)?,
            messenger: None,
            mesh_structure: Handle::default(),
            tmp_map: DataMap::with_hasher(DefaultHashPolicy, 1),
            indices: Vec::new(),
        })
    }

    /// `ToPolyTriangulation(theNodes, thePolyTriangles)`: a triangulation of the nodes
    /// and triangles; `BRepMeshTriangulator::to_poly_triangulation(&nodes, &triangles)?`.
    pub fn to_poly_triangulation(
        nodes: &Array1<'_, Point3>,
        triangles: &[Triangle],
    ) -> Result<Handle<Triangulation>, Failure> {
        let mut array: Array1<'_, Triangle> =
            Array1::with_bounds(1, triangles.len() as i32).map_err(|error| {
                Failure::with_message("Standard_RangeError", &format!("BRepMesh_Triangulator::ToPolyTriangulation: {error}; expected {} triangles", triangles.len()))
            })?;
        for (offset, triangle) in triangles.iter().enumerate() {
            array[offset as i32 + 1] = *triangle;
        }
        Ok(Handle::new(Triangulation::from_nodes(nodes, &array)?))
    }

    /// `SetMessenger(theMess)`; `triangulator.set_messenger(messenger)`.
    pub fn set_messenger(&mut self, messenger: Arc<Messenger>) {
        self.messenger = Some(messenger);
    }

    /// `Perform(thePolyTriangles)`: a single wire of 3 or 4 nodes gets its direct
    /// triangles; otherwise the Delaunay triangulation of the projected wires. False
    /// (with a Message_Fail) when building or meshing the structure fails; a bad node
    /// index of a short wire is a Standard_OutOfRange failure (not caught natively).
    pub fn perform(&mut self, triangles: &mut Vec<Triangle>) -> Result<bool, Failure> {
        if self.messenger.is_none() {
            self.messenger = Some(Message::default_messenger().clone());
        }
        if let [wire] = self.wires
            && wire.len() < 5
        {
            self.add_triangle_34(wire, triangles)?;
            return Ok(true);
        }
        if !self.prepare_mesh_structure() {
            return Ok(false);
        }
        Ok(self.triangulate(triangles))
    }

    /// `addTriange34`: one triangle, or the two triangles of the shorter diagonal of a
    /// quadrangle unless that diagonal leaves an inner corner.
    fn add_triangle_34(&self, wire: &[i32], triangles: &mut Vec<Triangle>) -> Result<(), Failure> {
        if wire.len() == 3 {
            triangles.push(Triangle::new(wire[0] + 1, wire[1] + 1, wire[2] + 1));
            return Ok(());
        }
        if wire.len() != 4 {
            return Ok(());
        }
        let d13 = (self.node(wire, 1)? - self.node(wire, 3)?).square_modulus();
        let d24 = (self.node(wire, 2)? - self.node(wire, 4)?).square_modulus();
        let use13 = if d24 < d13 {
            self.check_condition([2, 4, 1, 3], wire)?
        } else {
            !self.check_condition([1, 3, 2, 4], wire)?
        };
        let corners: [[usize; 3]; 2] = if use13 {
            [[1, 2, 3], [3, 4, 1]]
        } else {
            [[1, 2, 4], [2, 3, 4]]
        };
        for [a, b, c] in corners {
            triangles.push(Triangle::new(
                wire[a - 1] + 1,
                wire[b - 1] + 1,
                wire[c - 1] + 1,
            ));
        }
        Ok(())
    }

    /// The node at 1-based position `position` of the wire.
    fn node(&self, wire: &[i32], position: usize) -> Result<Xyz, Failure> {
        let index = wire[position - 1];
        usize::try_from(index)
            .ok()
            .and_then(|index| self.xyzs.get(index).copied())
            .ok_or_else(|| {
                Failure::with_message(
                    "Standard_OutOfRange",
                    &format!(
                        "BRepMesh_Triangulator: node {index}; expected 0..{}",
                        self.xyzs.len()
                    ),
                )
            })
    }

    /// `checkCondition`: the two triangles of the pivot diagonal are degenerate or face
    /// the same way (normals within 0.01 rad).
    fn check_condition(&self, nodes: [usize; 4], wire: &[i32]) -> Result<bool, Failure> {
        let origin = self.node(wire, nodes[0])?;
        let v0 = self.node(wire, nodes[1])? - origin;
        let v1 = self.node(wire, nodes[2])? - origin;
        let v2 = self.node(wire, nodes[3])? - origin;
        let (cross1, cross2) = (v0.crossed(v1), v0.crossed(v2));
        let tolerance = precision::square_confusion();
        if cross1.square_modulus() < tolerance || cross2.square_modulus() < tolerance {
            return Ok(true);
        }
        Ok(Direction3::from_xyz(cross1)?.is_equal(&Direction3::from_xyz(cross2)?, 0.01))
    }

    /// `prepareMeshStructure`: frontier nodes at the projected wire points and the
    /// closed frontier links of each wire.
    fn prepare_mesh_structure(&mut self) -> bool {
        let allocator = Handle::new(IncrementalAllocatorObject::new(0));
        self.mesh_structure = Handle::new(BRepMeshDataStructureOfDelaun::new(&allocator, 100));
        self.indices = Vec::with_capacity(self.wires.iter().map(Vec::len).sum());
        if self.fill_structure().is_err() {
            self.send_fail("makeTrianglesUsingBRepMesh: Exception raised during filling of BRepMesh_DataStructureOfDelaun");
            return false;
        }
        true
    }

    fn fill_structure(&mut self) -> Result<(), Failure> {
        let mut node_number = 0;
        for wire in self.wires {
            for &index in wire {
                let point = self.xyzs.get(index as usize).copied().ok_or_else(|| {
                    Failure::with_message(
                        "Standard_OutOfRange",
                        &format!(
                            "BRepMesh_Triangulator: node {index}; expected below {}",
                            self.xyzs.len()
                        ),
                    )
                })?;
                let projected = project_point_on_plane(&self.plane, Point3::from(point));
                let vertex =
                    BRepMeshVertex::new(projected.xy(), node_number, DegreeOfFreedom::Frontier);
                self.indices
                    .push(self.mesh_structure.add_node(&vertex, false)?);
                self.tmp_map.bind(node_number + 1, index + 1);
                node_number += 1;
            }
            self.add_wire_links(wire.len(), node_number as usize)?;
        }
        Ok(())
    }

    /// The closed loop of frontier links over the last `length` structure nodes.
    fn add_wire_links(&self, length: usize, end: usize) -> Result<(), Failure> {
        let offset = end - length;
        for position in 1..=length {
            let first = self.indices[offset + position - 1];
            let last = self.indices[offset + position % length];
            self.mesh_structure.add_link(&BRepMeshEdge::new(
                first,
                last,
                DegreeOfFreedom::Frontier,
            ))?;
        }
        Ok(())
    }

    /// `triangulate`: the Delaunay mesh of the structure nodes, its non-deleted domain
    /// triangles in packed-map order with the original 1-based node indices.
    fn triangulate(&mut self, triangles: &mut Vec<Triangle>) -> bool {
        match self.collect_triangles(triangles) {
            Ok(found) => found,
            Err(failure) => {
                let text = format!(
                    "makeTrianglesUsingBRepMesh: Exception raised during polygon triangulation: {}",
                    failure.message()
                );
                self.send_fail(&text);
                false
            }
        }
    }

    fn collect_triangles(&mut self, triangles: &mut Vec<Triangle>) -> Result<bool, Failure> {
        BRepMeshDelaun::with_indices(&self.mesh_structure, &mut self.indices, (-1, -1))?;
        let domain = self.mesh_structure.elements_of_domain();
        if domain.extent() < 1 {
            return Ok(false);
        }
        for triangle_id in domain.iter() {
            let triangle = self.mesh_structure.get_element(triangle_id)?;
            if triangle.movability() == DegreeOfFreedom::Deleted {
                continue;
            }
            let nodes = self.mesh_structure.element_nodes(&triangle)?;
            triangles.push(Triangle::new(
                *self.tmp_map.find(&nodes[0])?,
                *self.tmp_map.find(&nodes[1])?,
                *self.tmp_map.find(&nodes[2])?,
            ));
        }
        Ok(true)
    }

    fn send_fail(&self, text: &str) {
        if let Some(messenger) = &self.messenger {
            messenger.send_c_str(text.as_bytes(), Gravity::Fail);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Xyz> {
        vec![
            Xyz::new(0.0, 0.0, 0.0),
            Xyz::new(1.0, 0.0, 0.0),
            Xyz::new(1.0, 1.0, 0.0),
            Xyz::new(0.0, 1.0, 0.0),
            Xyz::new(0.5, 1.5, 0.0),
        ]
    }

    #[test]
    fn quadrangle_splits_along_the_shorter_diagonal() {
        let nodes = square();
        let wires = [vec![0, 1, 2, 3]];
        let normal = Direction3::new(0.0, 0.0, 1.0).unwrap();
        let mut triangulator = BRepMeshTriangulator::new(&nodes, &wires, normal).unwrap();
        let mut triangles = Vec::new();
        assert!(triangulator.perform(&mut triangles).unwrap());
        assert_eq!(
            triangles,
            vec![Triangle::new(1, 2, 3), Triangle::new(3, 4, 1)]
        );
    }

    #[test]
    fn pentagon_goes_through_the_delaunay_mesh() {
        let nodes = square();
        let wires = [vec![0, 1, 2, 4, 3]];
        let normal = Direction3::new(0.0, 0.0, 1.0).unwrap();
        let mut triangulator = BRepMeshTriangulator::new(&nodes, &wires, normal).unwrap();
        let mut triangles = Vec::new();
        assert!(triangulator.perform(&mut triangles).unwrap());
        assert_eq!(triangles.len(), 3);
        let mut points: Array1<'_, Point3> = Array1::with_bounds(1, nodes.len() as i32).unwrap();
        for (offset, xyz) in nodes.iter().enumerate() {
            points[offset as i32 + 1] = Point3::from(*xyz);
        }
        let poly = BRepMeshTriangulator::to_poly_triangulation(&points, &triangles).unwrap();
        assert_eq!(poly.nb_triangles(), 3);
    }
}
