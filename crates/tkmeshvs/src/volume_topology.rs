// SPDX-License-Identifier: GPL-2.0-only
//! Immutable Martensite visualization connectivity, with historical face ordering from
//! OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 MeshVS_DataSource3D static factories.
//! These constructors provide exact combinatorial connectivity only, not geometry,
//! solid validation, tolerances, history, cached instance getters, inheritance or RTTI.
use std::collections::TryReserveError;
use std::fmt;

/// The requested connectivity construction; no geometry is supplied or inferred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeshVSTopologyOperation {
    /// Two polygonal bases and quadrilateral side faces.
    Prism,
    /// One polygonal base and triangular faces sharing apex zero.
    Pyramid,
}

/// Failed construction, including the requested operation and offending base-point count.
/// No partial topology is returned. Allocation failure retains its standard-library cause.
#[derive(Debug)]
pub enum MeshVSTopologyError {
    /// A polygonal base requires three or more points.
    TooFewBasePoints {
        /// Requested connectivity construction.
        operation: MeshVSTopologyOperation,
        /// Offending requested base-point count.
        base_points: i32,
    },
    /// Face/node identifiers or addressable storage exceed their integer domain.
    IndexOverflow {
        /// Requested connectivity construction.
        operation: MeshVSTopologyOperation,
        /// Offending requested base-point count.
        base_points: i32,
    },
    /// The allocator refused storage before any result was published.
    AllocationFailure {
        /// Requested connectivity construction.
        operation: MeshVSTopologyOperation,
        /// Offending requested base-point count.
        base_points: i32,
        /// Original allocation or capacity failure.
        cause: TryReserveError,
    },
}

impl MeshVSTopologyError {
    fn request(&self) -> (MeshVSTopologyOperation, i32) {
        match self {
            Self::TooFewBasePoints {
                operation,
                base_points,
            }
            | Self::IndexOverflow {
                operation,
                base_points,
            }
            | Self::AllocationFailure {
                operation,
                base_points,
                ..
            } => (*operation, *base_points),
        }
    }
}

impl fmt::Display for MeshVSTopologyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (operation, base_points) = self.request();
        let expected = match self {
            Self::TooFewBasePoints { .. } => "at least 3",
            Self::IndexOverflow { .. } => {
                "face/node indices fitting i32 and storage sizes fitting isize"
            }
            Self::AllocationFailure { .. } => "allocatable connectivity storage",
        };
        write!(
            formatter,
            "{operation:?} base_points={base_points}; expected {expected}"
        )?;
        if let Self::AllocationFailure { cause, .. } = self {
            write!(formatter, ": {cause}")?;
        }
        Ok(())
    }
}

impl std::error::Error for MeshVSTopologyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AllocationFailure { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

/// Owned, immutable connectivity; reads take no locks and clones own separate storage.
/// Face identifiers remain one-based; slice positions are zero-based. Node identifiers
/// retain the historical prism numbering (0..2n) or pyramid numbering (apex 0, base 1..=n).
/// Equality compares connectivity, not allocation identity. No mutable views are exposed.
///
/// ```
/// use martensite_tkmeshvs::create_prism_topology;
/// let topology = create_prism_topology(3).unwrap();
/// assert_eq!(topology.face(1), Some([2, 1, 0].as_slice()));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshVSVolumeTopology {
    nodes: Vec<i32>,
    face_offsets: Vec<usize>,
}

impl MeshVSVolumeTopology {
    /// Count complete faces, whose identifiers run from 1 through this count.
    ///
    /// ```
    /// assert_eq!(martensite_tkmeshvs::create_prism_topology(3).unwrap().face_count(), 5);
    /// ```
    pub fn face_count(&self) -> usize {
        self.face_offsets.len() - 1
    }

    /// Borrow a face's ordered nodes; invalid one-based face identifiers return `None`.
    ///
    /// ```
    /// let topology = martensite_tkmeshvs::create_pyramid_topology(3).unwrap();
    /// assert_eq!(topology.face(0), None);
    /// assert_eq!(topology.face(2), Some([0, 1, 2].as_slice()));
    /// ```
    pub fn face(&self, face_id: i32) -> Option<&[i32]> {
        let position = usize::try_from(face_id.checked_sub(1)?).ok()?;
        let start = *self.face_offsets.get(position)?;
        let end = *self.face_offsets.get(position + 1)?;
        Some(&self.nodes[start..end])
    }
}

/// Build prism connectivity with reversed lower base, forward upper base, then side faces.
/// Counts below 3 fail explicitly (historical native calls returned null); overflow and
/// allocation failures are typed errors. No empty or partial success is possible.
///
/// ```
/// use martensite_tkmeshvs::create_prism_topology;
/// assert!(create_prism_topology(2).is_err());
/// assert_eq!(create_prism_topology(3).unwrap().face_count(), 5);
/// ```
pub fn create_prism_topology(
    base_points: i32,
) -> Result<MeshVSVolumeTopology, MeshVSTopologyError> {
    let mut topology = allocate_volume_faces(base_points, MeshVSTopologyOperation::Prism)?;
    append_volume_face(&mut topology, (0..base_points).rev());
    append_volume_face(&mut topology, base_points..=base_points + (base_points - 1));
    for index in 0..base_points {
        let next = (index + 1) % base_points;
        // Historical prepend order fixes the winding in the attributed owned fixture.
        let nodes = [next, next + base_points, index + base_points, index];
        append_volume_face(&mut topology, nodes);
    }
    Ok(topology)
}

/// Build pyramid connectivity with reversed base and triangular sides using apex zero.
/// Counts below 3, overflowing indices, and failed allocation return typed errors.
///
/// ```
/// use martensite_tkmeshvs::create_pyramid_topology;
/// assert!(create_pyramid_topology(-1).is_err());
/// assert_eq!(create_pyramid_topology(4).unwrap().face_count(), 5);
/// ```
pub fn create_pyramid_topology(
    base_points: i32,
) -> Result<MeshVSVolumeTopology, MeshVSTopologyError> {
    let mut topology = allocate_volume_faces(base_points, MeshVSTopologyOperation::Pyramid)?;
    append_volume_face(&mut topology, (1..=base_points).rev());
    for index in 1..=base_points {
        append_volume_face(&mut topology, [0, index, index % base_points + 1]);
    }
    Ok(topology)
}

fn topology_sizes(
    base_points: i32,
    operation: MeshVSTopologyOperation,
) -> Result<(usize, usize), MeshVSTopologyError> {
    if base_points < 3 {
        return Err(MeshVSTopologyError::TooFewBasePoints {
            operation,
            base_points,
        });
    }
    checked_topology_sizes(base_points, operation).ok_or(MeshVSTopologyError::IndexOverflow {
        operation,
        base_points,
    })
}

fn checked_topology_sizes(
    base_points: i32,
    operation: MeshVSTopologyOperation,
) -> Option<(usize, usize)> {
    let (extra_offsets, entries_per_point, max_points) = match operation {
        MeshVSTopologyOperation::Prism => (3, 6, i32::MAX / 2 + 1),
        MeshVSTopologyOperation::Pyramid => (2, 4, i32::MAX - 1),
    };
    if base_points > max_points {
        return None;
    }
    let count = usize::try_from(base_points).ok()?;
    let nodes = count.checked_mul(entries_per_point)?;
    let offsets = count.checked_add(extra_offsets)?;
    (nodes <= isize::MAX as usize / size_of::<i32>()
        && offsets <= isize::MAX as usize / size_of::<usize>())
    .then_some((nodes, offsets))
}

fn allocate_volume_faces(
    base_points: i32,
    operation: MeshVSTopologyOperation,
) -> Result<MeshVSVolumeTopology, MeshVSTopologyError> {
    let (node_count, offset_count) = topology_sizes(base_points, operation)?;
    let mut topology = MeshVSVolumeTopology {
        nodes: reserve_topology_buffer(node_count, base_points, operation)?,
        face_offsets: reserve_topology_buffer(offset_count, base_points, operation)?,
    };
    topology.face_offsets.push(0);
    Ok(topology)
}

fn reserve_topology_buffer<T>(
    capacity: usize,
    base_points: i32,
    operation: MeshVSTopologyOperation,
) -> Result<Vec<T>, MeshVSTopologyError> {
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(capacity)
        .map_err(|cause| MeshVSTopologyError::AllocationFailure {
            operation,
            base_points,
            cause,
        })?;
    Ok(buffer)
}

fn append_volume_face(topology: &mut MeshVSVolumeTopology, nodes: impl IntoIterator<Item = i32>) {
    topology.nodes.extend(nodes);
    topology.face_offsets.push(topology.nodes.len());
}

#[cfg(test)]
#[path = "volume_topology_tests.rs"]
mod tests;
