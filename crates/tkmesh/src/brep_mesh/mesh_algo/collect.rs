// Rust translation of BRepMesh_BaseMeshAlgo::collectTriangles and collectNodes at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::base::{BaseMeshAlgoData, BaseMeshAlgoHooks, used_nodes, used_nodes_mut};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::poly::{Triangle, Triangulation};

/// `collectTriangles`: the domain triangles in packed-map order with their nodes
/// renumbered compactly (pre-bound frontier nodes keep their index, the others get
/// the next one on first use); `None` without domain triangle.
pub(super) fn collect_triangles(
    data: &mut BaseMeshAlgoData,
) -> Result<Option<Handle<Triangulation>>, Failure> {
    let structure = data.get_structure().clone();
    let triangles = structure.elements_of_domain();
    if triangles.is_empty() {
        return Ok(None);
    }
    let result = Handle::new(Triangulation::new());
    result.resize_triangles(triangles.extent(), false)?;
    for (offset, element_id) in triangles.iter().enumerate() {
        let element = structure.get_element(element_id)?;
        let mut nodes = structure.element_nodes(&element)?;
        for node in &mut nodes {
            *node = compact_index(used_nodes_mut(data), *node)?;
        }
        result.set_triangle(
            offset as i32 + 1,
            Triangle::new(nodes[0], nodes[1], nodes[2]),
        )?;
    }
    result.resize_nodes(used_nodes(data).extent() as i32, false)?;
    result.add_uv_nodes()?;
    Ok(Some(result))
}

/// The compact index of a structure node, bound to the next index on first use.
fn compact_index(
    used: &mut rustycad_tkernel::ncollection::DataMap<i32, i32>,
    node: i32,
) -> Result<i32, Failure> {
    if !used.is_bound(&node) {
        let next = used.extent() as i32 + 1;
        used.bind(node, next);
    }
    used.find(&node).copied()
}

/// `collectNodes`: the 3d point and the (virtual) 2d point of every used node.
pub(super) fn collect_nodes<A: BaseMeshAlgoHooks + ?Sized>(
    algo: &A,
    triangulation: &Handle<Triangulation>,
) -> Result<(), Failure> {
    let data = algo.base_data();
    let structure = data.get_structure();
    for index in 1..=data.get_nodes_map().len() as i32 {
        let Ok(node_index) = used_nodes(data).find(&index).copied() else {
            continue;
        };
        let vertex = structure.get_node(index)?;
        let point = node_point(data, vertex.location_3d())?;
        triangulation.set_node(node_index, point)?;
        triangulation.set_uv_node(node_index, algo.get_node_point2d(&vertex))?;
    }
    Ok(())
}

/// `myNodesMap->Value(theLocation3d)`, Standard_OutOfRange outside the map.
fn node_point(
    data: &BaseMeshAlgoData,
    location: i32,
) -> Result<rustycad_tkmath::gp::Point3, Failure> {
    usize::try_from(location)
        .ok()
        .and_then(|index| data.get_nodes_map().get(index).copied())
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "BRepMesh_BaseMeshAlgo::collectNodes: Location3d {location}; expected 0..{}",
                    data.get_nodes_map().len()
                ),
            )
        })
}
