// SPDX-License-Identifier: GPL-2.0-only
//! MeshVS_DataSource3D static factories, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! The data-source inheritance, RTTI and cached instance getters remain separate ports.
use rustycad_tkernel::{
    ncollection::{Array1, HArray1, Sequence},
    standard::Handle,
};

/// Shared, mutable one-based faces containing one-based sequences of node indices.
///
/// ```
/// use martensite_tkmeshvs::{MeshVSVolumeTopology, create_prism_topology};
/// let faces: MeshVSVolumeTopology = create_prism_topology(3);
/// assert_eq!(faces.array1().lower(), 1);
/// ```
pub type MeshVSVolumeTopology = Handle<HArray1<Sequence<i32>>>;

/// Preserve OCCT's prism face order and winding; fewer than three base points return null.
///
/// ```
/// use martensite_tkmeshvs::create_prism_topology;
/// assert!(create_prism_topology(2).is_null());
/// assert_eq!(create_prism_topology(3).array1().upper(), 5);
/// ```
pub fn create_prism_topology(base_points: i32) -> MeshVSVolumeTopology {
    if base_points < 3 {
        return Handle::default();
    }
    let topology = allocate_volume_faces(base_points, 2);
    {
        let mut faces = topology.change_array1();
        for index in 0..base_points {
            fill_prism_faces(&mut faces, base_points, index);
        }
    }
    topology
}

/// Preserve OCCT's pyramid face order with apex zero; fewer than three base points return null.
///
/// ```
/// use martensite_tkmeshvs::create_pyramid_topology;
/// assert!(create_pyramid_topology(-1).is_null());
/// assert_eq!(create_pyramid_topology(4).array1().upper(), 5);
/// ```
pub fn create_pyramid_topology(base_points: i32) -> MeshVSVolumeTopology {
    if base_points < 3 {
        return Handle::default();
    }
    let topology = allocate_volume_faces(base_points, 1);
    {
        let mut faces = topology.change_array1();
        for index in 1..=base_points {
            fill_pyramid_faces(&mut faces, base_points, index);
        }
    }
    topology
}

fn allocate_volume_faces(base_points: i32, extra_faces: i32) -> MeshVSVolumeTopology {
    // Native signed overflow has no defined result; reject it before allocating the array.
    let upper = base_points.checked_add(extra_faces).unwrap_or_else(|| {
        panic!("base points={base_points}; expected count + {extra_faces} to fit i32")
    });
    let faces = HArray1::with_bounds(1, upper).unwrap_or_else(|cause| {
        panic!("face bounds=[1,{upper}]; expected an allocatable one-based array: {cause}")
    });
    Handle::new(faces)
}

fn fill_prism_faces(faces: &mut Array1<'_, Sequence<i32>>, base_points: i32, index: i32) {
    // Prepend order is observable in face winding and must follow the pinned implementation.
    faces.change_value(1).unwrap().prepend(index);
    faces.change_value(2).unwrap().append(index + base_points);
    let side = faces.change_value(3 + index).unwrap();
    side.prepend(index);
    side.prepend(index + base_points);
    let next = (index + 1) % base_points;
    side.prepend(next + base_points);
    side.prepend(next);
}

fn fill_pyramid_faces(faces: &mut Array1<'_, Sequence<i32>>, base_points: i32, index: i32) {
    faces.change_value(1).unwrap().prepend(index);
    let side = faces.change_value(1 + index).unwrap();
    side.append(0);
    side.append(index);
    side.append(index % base_points + 1);
}
