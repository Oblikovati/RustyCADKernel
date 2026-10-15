// SPDX-License-Identifier: GPL-2.0-only
//! The primitive helpers of `MeshVS_MeshPrsBuilder`: links, face edges and triangles, volume
//! triangles and edges, primitive counts and element centers.
use super::MeshVSMeshPrsBuilder;
use crate::MeshVSTool;
use crate::data_source::MeshVSTopologyHandle;
use crate::data_source::normals::{coord, face_polygon};
use crate::node_pairs::{MeshVSNodePair, MeshVSSymmetricPairHasher};
use rustycad_tkernel::ncollection::{Array1, Map, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Vector3;
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPrimitives, Graphic3dArrayOfSegments, Graphic3dArrayOfTriangles,
};

/// The point `index` (one-based) of a coordinate array.
pub(crate) fn node_xyz(coords: &Array1<'_, f64>, index: i32) -> Result<[f64; 3], Failure> {
    Ok([
        coord(coords, 3 * index - 2)?,
        coord(coords, 3 * index - 1)?,
        coord(coords, 3 * index)?,
    ])
}

/// `p` moved toward `center` by the shrink coefficient: `(p - c) * k + c`.
fn shrink_toward(p: [f64; 3], center: [f64; 3], k: f64) -> [f64; 3] {
    std::array::from_fn(|axis| (p[axis] - center[axis]) * k + center[axis])
}

/// `p` moved toward `center` the other way round: `c + k * (p - c)` (ProcessFace and the
/// volume edges keep this association).
fn shrink_from(p: [f64; 3], center: [f64; 3], k: f64) -> [f64; 3] {
    std::array::from_fn(|axis| center[axis] + k * (p[axis] - center[axis]))
}

fn add_xyz(array: &Graphic3dArrayOfPrimitives, p: [f64; 3]) -> Result<(), Failure> {
    array.add_vertex_f64(p[0], p[1], p[2]).map(|_| ())
}

/// The triangle fan of polygon node `node` and sub-vertex `sub` (0 is always the first node).
pub(crate) fn fan_index(node: i32, sub: i32) -> i32 {
    if sub == 0 { 0 } else { node + sub }
}

impl MeshVSMeshPrsBuilder {
    /// `CalculateCenter`: the mean of up to three nodes, else the middle of nodes 1 and 3.
    /// Example: `MeshVSMeshPrsBuilder::calculate_center(&coords, 4)?`.
    pub fn calculate_center(coords: &Array1<'_, f64>, nb_nodes: i32) -> Result<[f64; 3], Failure> {
        if nb_nodes >= 4 {
            let (a, b) = (node_xyz(coords, 1)?, node_xyz(coords, 3)?);
            return Ok(std::array::from_fn(|axis| (a[axis] + b[axis]) / 2.0));
        }
        let mut center = [0.0; 3];
        for k in 1..=nb_nodes {
            let p = node_xyz(coords, k)?;
            for axis in 0..3 {
                center[axis] += p[axis];
            }
        }
        Ok(center.map(|c| c / f64::from(nb_nodes)))
    }
    /// `HowManyPrimitives`: adds the vertices and bounds of a volume, as polygons or
    /// selected faces, else as its Euler edge count. Example: `how_many_primitives(&topo, true, false, 8, &mut v, &mut b)`.
    pub fn how_many_primitives(
        topo: &MeshVSTopologyHandle,
        as_polygons: bool,
        is_select: bool,
        nb_nodes: i32,
        vertices: &mut i32,
        bounds: &mut i32,
    ) {
        let Some(topo) = topo.get() else {
            return;
        };
        let faces = topo.array1();
        let face_count = faces.length();
        if as_polygons || is_select {
            *bounds += face_count;
            *vertices += faces.iter().map(|f| f.length()).sum::<i32>();
            if is_select {
                *vertices += face_count;
            }
            return;
        }
        // The edge count of a polyhedron from Euler's formula.
        let edges = nb_nodes + face_count - 2;
        *bounds += edges;
        *vertices += 2 * edges;
    }
    /// `AddVolumePrs`: the triangles of every face (an array of triangles), the shrunk face
    /// edges (selection), else every edge once in the native map order.
    #[allow(clippy::too_many_arguments)]
    pub fn add_volume_prs(
        topo: &MeshVSTopologyHandle,
        nodes: &Array1<'_, f64>,
        nb_nodes: i32,
        array: &Handle<Graphic3dArrayOfPrimitives>,
        is_shaded: bool,
        is_shrinked: bool,
        is_selected: bool,
        shrink_coef: f64,
    ) -> Result<(), Failure> {
        let (Some(faces), Some(target)) = (topo.get(), array.get()) else {
            return Ok(());
        };
        let center = if is_shrinked {
            volume_center(nodes, nb_nodes)?
        } else {
            [0.0; 3]
        };
        let faces = faces.array1();
        if !array.cast::<Graphic3dArrayOfTriangles>().is_null() {
            let face_shape = FaceShape {
                center,
                shrink_coef,
                is_shrinked,
                is_shaded,
            };
            for face in faces.iter() {
                process_face(face, nodes, &face_shape, target)?;
            }
            return Ok(());
        }
        if is_selected {
            for face in faces.iter() {
                add_shrunk_face_edges(face, nodes, shrink_coef, target)?;
            }
            return Ok(());
        }
        add_volume_edges(
            faces.iter(),
            nodes,
            (is_shrinked, center, shrink_coef),
            target,
        )
    }
}

/// The node of zero-based index `index` in a coordinate array of any lower bound.
fn volume_node(nodes: &Array1<'_, f64>, index: i32) -> Result<[f64; 3], Failure> {
    let low = nodes.lower();
    Ok([
        coord(nodes, low + 3 * index)?,
        coord(nodes, low + 3 * index + 1)?,
        coord(nodes, low + 3 * index + 2)?,
    ])
}

fn volume_center(nodes: &Array1<'_, f64>, nb_nodes: i32) -> Result<[f64; 3], Failure> {
    let mut center = [0.0; 3];
    let low = nodes.lower();
    for index in 0..3 * nb_nodes {
        center[(index % 3) as usize] += coord(nodes, low + index)?;
    }
    Ok(center.map(|c| c / f64::from(nb_nodes)))
}

struct FaceShape {
    center: [f64; 3],
    shrink_coef: f64,
    is_shrinked: bool,
    is_shaded: bool,
}

/// The anonymous `ProcessFace` of MeshVS_MeshPrsBuilder.cxx: the fan triangles of a face,
/// with the average normal (or +Z) when shaded.
fn process_face(
    face: &Sequence<i32>,
    nodes: &Array1<'_, f64>,
    shape: &FaceShape,
    array: &Graphic3dArrayOfPrimitives,
) -> Result<(), Failure> {
    let count = face.length();
    let mut points = Vec::with_capacity(count as usize);
    for &index in face.iter() {
        let p = volume_node(nodes, index)?;
        points.push(if shape.is_shrinked {
            shrink_from(p, shape.center, shape.shrink_coef)
        } else {
            p
        });
    }
    let normal = if shape.is_shaded {
        face_normal(&points)?
    } else {
        [0.0; 3]
    };
    for node in 0..count - 2 {
        for sub in 0..3 {
            let p = points[fan_index(node, sub) as usize];
            if shape.is_shaded {
                array.add_vertex_normal_f64(p, normal)?;
            } else {
                add_xyz(array, p)?;
            }
        }
    }
    Ok(())
}

/// MeshVS_Tool::GetAverageNormal of the polygon, +Z when it is not planar.
fn face_normal(points: &[[f64; 3]]) -> Result<[f64; 3], Failure> {
    let flat: Vec<f64> = points.iter().flatten().copied().collect();
    let all = Array1::from_shared(&flat, 1);
    let poly = face_polygon(0..points.len() as i32, &all)?;
    let mut norm = Vector3::new(0.0, 0.0, 0.0);
    if !MeshVSTool::get_average_normal(&poly, &mut norm)? {
        return Ok([0.0, 0.0, 1.0]);
    }
    Ok([norm.x(), norm.y(), norm.z()])
}

/// The edges of a face shrunk around the face center by the coefficient (always applied).
fn add_shrunk_face_edges(
    face: &Sequence<i32>,
    nodes: &Array1<'_, f64>,
    shrink_coef: f64,
    array: &Graphic3dArrayOfPrimitives,
) -> Result<(), Failure> {
    let count = face.length();
    let points: Vec<[f64; 3]> = face
        .iter()
        .map(|&i| volume_node(nodes, i))
        .collect::<Result<_, _>>()?;
    let mut center = [0.0; 3];
    for p in &points {
        for axis in 0..3 {
            center[axis] += p[axis];
        }
    }
    let center = center.map(|c| c / f64::from(count));
    for node in 0..count {
        for sub in 0..2 {
            let p = points[((node + sub) % count) as usize];
            add_xyz(array, shrink_from(p, center, shrink_coef))?;
        }
    }
    Ok(())
}

/// Every edge of the faces once (a symmetric node-pair map), in the native map order.
fn add_volume_edges<'a>(
    faces: impl Iterator<Item = &'a Sequence<i32>>,
    nodes: &Array1<'_, f64>,
    (is_shrinked, center, shrink_coef): (bool, [f64; 3], f64),
    array: &Graphic3dArrayOfPrimitives,
) -> Result<(), Failure> {
    let edges = volume_edge_map(faces);
    for &(first, second) in edges.iter() {
        let mut ends = [volume_node(nodes, first)?, volume_node(nodes, second)?];
        if is_shrinked {
            ends = ends.map(|p| shrink_from(p, center, shrink_coef));
        }
        add_xyz(array, ends[0])?;
        add_xyz(array, ends[1])?;
    }
    Ok(())
}

/// The native `NCollection_Map<MeshVS_NodePair, MeshVS_SymmetricPairHasher>` of the edges.
pub(crate) fn volume_edge_map<'a>(
    faces: impl Iterator<Item = &'a Sequence<i32>>,
) -> Map<MeshVSNodePair, MeshVSSymmetricPairHasher> {
    let mut edges = Map::<MeshVSNodePair, MeshVSSymmetricPairHasher>::default();
    for face in faces {
        let ids: Vec<i32> = face.iter().copied().collect();
        let count = ids.len();
        for node in 0..count {
            edges.add((ids[node], ids[(node + 1) % count]));
        }
    }
    edges
}

impl MeshVSMeshPrsBuilder {
    /// `AddLinkPrs` (protected): a link, shrunk around its middle when asked.
    pub fn add_link_prs(
        coords: &Array1<'_, f64>,
        segments: &Graphic3dArrayOfSegments,
        is_shrinked: bool,
        shrink_coef: f64,
    ) -> Result<(), Failure> {
        let (mut a, mut b) = (node_xyz(coords, 1)?, node_xyz(coords, 2)?);
        if is_shrinked {
            let middle: [f64; 3] = std::array::from_fn(|axis| (a[axis] + b[axis]) * 0.5);
            a = shrink_toward(a, middle, shrink_coef);
            b = std::array::from_fn(|axis| 2.0 * middle[axis] - a[axis]);
        }
        add_xyz(segments, a)?;
        add_xyz(segments, b)
    }
    /// `AddFaceWirePrs` (protected): every edge of a face, shrunk around its center when asked.
    pub fn add_face_wire_prs(
        coords: &Array1<'_, f64>,
        nb_nodes: i32,
        segments: &Graphic3dArrayOfSegments,
        is_shrinked: bool,
        shrink_coef: f64,
    ) -> Result<(), Failure> {
        let points = face_points(coords, nb_nodes, is_shrinked, shrink_coef)?;
        for node in 0..nb_nodes {
            add_xyz(segments, points[node as usize])?;
            add_xyz(segments, points[((node + 1) % nb_nodes) as usize])?;
        }
        Ok(())
    }
}

/// The face nodes, shrunk around the element center when asked.
pub(crate) fn face_points(
    coords: &Array1<'_, f64>,
    nb_nodes: i32,
    is_shrinked: bool,
    shrink_coef: f64,
) -> Result<Vec<[f64; 3]>, Failure> {
    let center = if is_shrinked {
        MeshVSMeshPrsBuilder::calculate_center(coords, nb_nodes)?
    } else {
        [0.0; 3]
    };
    (1..=nb_nodes)
        .map(|k| {
            let p = node_xyz(coords, k)?;
            Ok(if is_shrinked {
                shrink_toward(p, center, shrink_coef)
            } else {
                p
            })
        })
        .collect()
}
