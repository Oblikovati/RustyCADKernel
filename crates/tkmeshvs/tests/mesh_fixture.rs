// SPDX-License-Identifier: GPL-2.0-only
//! A named in-memory mesh data source for the Martensite MeshVS tests: a unit square split
//! in two triangles, a link, a unit-cube prism over the first triangle and a 0D element.
use martensite_tkmeshvs::flags::*;
use martensite_tkmeshvs::{
    MeshVSDataSource, MeshVSDataSource3D, MeshVSDataSource3DBase, MeshVSEntityType,
    MeshVSPackedMap, MeshVSTopologyHandle,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::StandardTransient;

/// Node coordinates (ids 1..=8).
pub const NODES: [[f64; 3]; 8] = [
    [0., 0., 0.],
    [1., 0., 0.],
    [1., 1., 0.],
    [0., 1., 0.],
    [0., 0., 1.],
    [1., 0., 1.],
    [1., 1., 1.],
    [0.5, 0.5, 0.5],
];
/// Elements (ids 11..=16): two triangles, a link, a prism, a 0D element, a degenerate face.
pub const ELEMENTS: [(MeshVSEntityType, &[i32]); 6] = [
    (MESHVS_ET_FACE, &[1, 2, 3]),
    (MESHVS_ET_FACE, &[1, 3, 4]),
    (MESHVS_ET_LINK, &[2, 3]),
    (MESHVS_ET_VOLUME, &[1, 2, 3, 5, 6, 7]),
    (MESHVS_ET_0D, &[8]),
    (MESHVS_ET_FACE, &[1, 2, 2]),
];

/// The fake data source of the tests; `UnitSquareSource::default()`.
#[derive(Default)]
pub struct UnitSquareSource {
    transient: StandardTransient,
    base: MeshVSDataSource3DBase,
}
rustycad_tkernel::impl_transient!(UnitSquareSource, "UnitSquareSource", dyn MeshVSDataSource3D, transient,
    interfaces: [dyn MeshVSDataSource, dyn MeshVSDataSource3D]);

fn element(id: i32) -> Option<(MeshVSEntityType, &'static [i32])> {
    ELEMENTS.get(usize::try_from(id - 11).ok()?).copied()
}

fn write(coords: &mut Array1<'_, f64>, ids: &[i32]) -> bool {
    if coords.length() < 3 * ids.len() as i32 {
        return false;
    }
    let low = coords.lower();
    for (k, id) in ids.iter().enumerate() {
        for a in 0..3 {
            coords[low + 3 * k as i32 + a as i32] = NODES[(*id - 1) as usize][a];
        }
    }
    true
}

impl MeshVSDataSource for UnitSquareSource {
    fn get_geom(&self, id: i32, is_element: bool, coords: &mut Array1<'_, f64>, nb_nodes: &mut i32, kind: &mut MeshVSEntityType) -> bool {
        if !is_element {
            if !(1..=8).contains(&id) {
                return false;
            }
            (*nb_nodes, *kind) = (1, MESHVS_ET_NODE);
            return write(coords, &[id]);
        }
        let Some((element_kind, nodes)) = element(id) else {
            return false;
        };
        (*nb_nodes, *kind) = (nodes.len() as i32, element_kind);
        write(coords, nodes)
    }
    fn get_geom_type(&self, id: i32, is_element: bool, kind: &mut MeshVSEntityType) -> bool {
        if !is_element {
            *kind = MESHVS_ET_NODE;
            return (1..=8).contains(&id);
        }
        element(id).map(|(k, _)| *kind = k).is_some()
    }
    fn get_3d_geom(&self, id: i32, nb_nodes: &mut i32, data: &mut MeshVSTopologyHandle) -> bool {
        if id != 14 {
            return false;
        }
        *nb_nodes = 6;
        *data = self.get_prism_topology(3);
        true
    }
    fn get_addr(&self, id: i32, _: bool) -> usize {
        id as usize
    }
    fn get_nodes_by_element(&self, id: i32, node_ids: &mut Array1<'_, i32>, nb_nodes: &mut i32) -> bool {
        let Some((_, nodes)) = element(id) else {
            return false;
        };
        if node_ids.length() < nodes.len() as i32 {
            return false;
        }
        *nb_nodes = nodes.len() as i32;
        let low = node_ids.lower();
        for (k, node) in nodes.iter().enumerate() {
            node_ids[low + k as i32] = *node;
        }
        true
    }
    fn get_all_nodes(&self) -> MeshVSPackedMap {
        let mut map = MeshVSPackedMap::default();
        for id in 1..=8 {
            map.add(id);
        }
        map
    }
    fn get_all_elements(&self) -> MeshVSPackedMap {
        let mut map = MeshVSPackedMap::default();
        for id in 11..=16 {
            map.add(id);
        }
        map
    }
}

impl MeshVSDataSource3D for UnitSquareSource {
    fn data_source_3d_base(&self) -> &MeshVSDataSource3DBase {
        &self.base
    }
}
