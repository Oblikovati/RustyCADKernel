// SPDX-License-Identifier: GPL-2.0-only
//! The native bodies of the virtual normal and bound members of `MeshVS_DataSource`.
//!
//! An index outside the caller's array is undefined natively (MeshVS_Buffer storage); the
//! port reports it as a failed query instead of reading foreign memory.
use super::{MeshVSDataSource, MeshVSTopologyHandle};
use crate::MeshVSTool;
use crate::flags::{MESHVS_ET_FACE, MESHVS_ET_NONE, MESHVS_ET_VOLUME};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point3, Vector3, resolution};

/// Reads `coords(index)` or fails like an out-of-range native access.
pub(crate) fn coord(coords: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    coords
        .value(index)
        .copied()
        .map_err(|error| error.into_failure("MeshVS coordinates"))
}

/// A one-based array of `3 * max_nodes` coordinates, as the native MeshVS buffers.
pub(crate) fn coords_buffer(max_nodes: i32) -> Result<Array1<'static, f64>, Failure> {
    Array1::with_bounds(1, 3 * max_nodes).map_err(|error| error.into_failure("MeshVS_Buffer"))
}

/// The native `MeshVS_DataSource::GetNormal` body: the unit normal of a face from its first,
/// second and last nodes; a degenerate face writes zeros and fails.
/// Example: `meshvs_data_source_get_normal(&*source, 7, 10, [&mut x, &mut y, &mut z])`.
pub fn meshvs_data_source_get_normal<D: MeshVSDataSource + ?Sized>(
    source: &D,
    id: i32,
    max: i32,
    normal: [&mut f64; 3],
) -> bool {
    if max <= 0 {
        return false;
    }
    face_normal(source, id, max, normal).unwrap_or(false)
}

fn face_normal<D: MeshVSDataSource + ?Sized>(
    source: &D,
    id: i32,
    max: i32,
    normal: [&mut f64; 3],
) -> Result<bool, Failure> {
    let mut coords = coords_buffer(max)?;
    let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
    if !source.get_geom(id, true, &mut coords, &mut nb_nodes, &mut kind) {
        return Ok(false);
    }
    if kind != MESHVS_ET_FACE || nb_nodes < 3 {
        return Ok(false);
    }
    let last = (nb_nodes - 1) * 3;
    let p = point_delta(&coords, 1, 4)?;
    let q = point_delta(&coords, 1, last + 1)?;
    let [nx, ny, nz] = normal;
    *nx = p[1] * q[2] - p[2] * q[1];
    *ny = p[2] * q[0] - p[0] * q[2];
    *nz = p[0] * q[1] - p[1] * q[0];
    let len = (*nx * *nx + *ny * *ny + *nz * *nz).sqrt();
    if len <= resolution() {
        (*nx, *ny, *nz) = (0.0, 0.0, 0.0);
        return Ok(false);
    }
    (*nx, *ny, *nz) = (*nx / len, *ny / len, *nz / len);
    Ok(true)
}

/// The vector from the point at `from` to the point at `to` (one-based coordinate indices).
fn point_delta(coords: &Array1<'_, f64>, from: i32, to: i32) -> Result<[f64; 3], Failure> {
    let mut delta = [0.0; 3];
    for (axis, value) in delta.iter_mut().enumerate() {
        let axis = axis as i32;
        *value = coord(coords, to + axis)? - coord(coords, from + axis)?;
    }
    Ok(delta)
}

/// The native `MeshVS_DataSource::GetNormalsByElement` body: nodal normals of a face when
/// asked and available, else its face normal for every node; the average normal of each face
/// of a volume. Example: `meshvs_data_source_get_normals_by_element(&*source, 7, true, 10, &mut normals)`.
pub fn meshvs_data_source_get_normals_by_element<D: MeshVSDataSource + ?Sized>(
    source: &D,
    id: i32,
    is_nodal: bool,
    max_nodes: i32,
    normals: &mut Handle<HArray1<f64>>,
) -> bool {
    if max_nodes <= 0 {
        return false;
    }
    match element_normals(source, id, is_nodal, max_nodes) {
        Ok(Some(computed)) => {
            *normals = Handle::new(computed);
            true
        }
        _ => false,
    }
}

fn element_normals<D: MeshVSDataSource + ?Sized>(
    source: &D,
    id: i32,
    is_nodal: bool,
    max_nodes: i32,
) -> Result<Option<HArray1<f64>>, Failure> {
    let mut coords = coords_buffer(max_nodes)?;
    let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
    if !source.get_geom(id, true, &mut coords, &mut nb_nodes, &mut kind) {
        return Ok(None);
    }
    let mut topo = MeshVSTopologyHandle::default();
    let mut nb_normals = nb_nodes;
    if kind == MESHVS_ET_VOLUME {
        if !source.get_3d_geom(id, &mut nb_nodes, &mut topo) {
            return Ok(None);
        }
        nb_normals = topo.get().map_or(0, |t| t.array1().length());
    }
    let normals = HArray1::<f64>::with_bounds(1, 3 * nb_normals)
        .map_err(|error| error.into_failure("NCollection_HArray1"))?;
    let all_nodal =
        kind == MESHVS_ET_FACE && is_nodal && nodal_normals(source, id, nb_nodes, &normals);
    if all_nodal {
        return Ok(Some(normals));
    }
    let computed = match kind {
        MESHVS_ET_FACE => repeated_face_normal(source, id, max_nodes, nb_nodes, &normals)?,
        MESHVS_ET_VOLUME => volume_face_normals(&topo, &coords, &normals)?,
        _ => false,
    };
    Ok(computed.then_some(normals))
}

/// Asks the node normals in order, stopping at the first missing one.
fn nodal_normals<D: MeshVSDataSource + ?Sized>(
    source: &D,
    id: i32,
    nb_nodes: i32,
    normals: &HArray1<f64>,
) -> bool {
    let mut array = normals.change_array1();
    for k in 1..=nb_nodes {
        let [mut x, mut y, mut z] = slots(&array, k);
        let found = source.get_node_normal(k, id, &mut x, &mut y, &mut z);
        store(&mut array, k, [x, y, z]);
        if !found {
            return false;
        }
    }
    true
}

/// The current values of the three slots of a normal (natively written in place).
fn slots(array: &Array1<'_, f64>, k: i32) -> [f64; 3] {
    std::array::from_fn(|axis| array.value(3 * k - 2 + axis as i32).copied().unwrap_or(0.0))
}

fn store(array: &mut Array1<'_, f64>, k: i32, value: [f64; 3]) {
    for (axis, component) in value.into_iter().enumerate() {
        if let Ok(slot) = array.change_value(3 * k - 2 + axis as i32) {
            *slot = component;
        }
    }
}

/// `GetNormal` (virtually) into the first normal, copied to every node on success.
fn repeated_face_normal<D: MeshVSDataSource + ?Sized>(
    source: &D,
    id: i32,
    max_nodes: i32,
    nb_nodes: i32,
    normals: &HArray1<f64>,
) -> Result<bool, Failure> {
    let mut array = normals.change_array1();
    let [mut x, mut y, mut z] = slots(&array, 1);
    let found = source.get_normal(id, max_nodes, &mut x, &mut y, &mut z);
    store(&mut array, 1, [x, y, z]);
    if found {
        for k in 2..=nb_nodes {
            store(&mut array, k, [x, y, z]);
        }
    }
    Ok(found)
}

/// The average normal of every face of a volume, the zero vector for a face of fewer than
/// three nodes (MeshVS_Tool::GetAverageNormal keeps its output then).
fn volume_face_normals(
    topo: &MeshVSTopologyHandle,
    coords: &Array1<'_, f64>,
    normals: &HArray1<f64>,
) -> Result<bool, Failure> {
    let Some(topo) = topo.get() else {
        return Ok(true);
    };
    let faces = topo.array1();
    let mut array = normals.change_array1();
    for (i, face) in faces.iter().enumerate() {
        let poly = face_polygon(face.iter().copied(), coords)?;
        let mut norm = Vector3::new(0.0, 0.0, 0.0);
        MeshVSTool::get_average_normal(&poly, &mut norm)?;
        store(&mut array, i as i32 + 1, [norm.x(), norm.y(), norm.z()]);
    }
    Ok(true)
}

/// The zero-based polygon array of MeshVS_Tool: the node count, then the coordinates of the
/// face nodes (zero-based node indices into `coords`).
pub(crate) fn face_polygon(
    face: impl ExactSizeIterator<Item = i32>,
    coords: &Array1<'_, f64>,
) -> Result<Array1<'static, f64>, Failure> {
    let count = face.len() as i32;
    let mut poly = Array1::<f64>::with_bounds(0, 3 * count)
        .map_err(|error| error.into_failure("MeshVS polygon"))?;
    poly[0] = f64::from(count);
    let low = coords.lower();
    for (j, index) in face.enumerate() {
        for axis in 0..3 {
            poly[3 * j as i32 + 1 + axis] = coord(coords, low + 3 * index + axis)?;
        }
    }
    Ok(poly)
}

/// The native `MeshVS_DataSource::GetBoundingBox` body: the box of every node whose
/// geometry is known. Example: `meshvs_data_source_get_bounding_box(&*source).is_void()`.
pub fn meshvs_data_source_get_bounding_box<D: MeshVSDataSource + ?Sized>(
    source: &D,
) -> BoundingBox3 {
    let mut bounds = BoundingBox3::default();
    let mut coords = Array1::<f64>::with_size(3);
    coords.update_lower_bound(1);
    for key in source.get_all_nodes().iter() {
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if !source.get_geom(key, false, &mut coords, &mut nb_nodes, &mut kind) {
            continue;
        }
        bounds.add_point(Point3::new(coords[1], coords[2], coords[3]));
    }
    bounds
}
