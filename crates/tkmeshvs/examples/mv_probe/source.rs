//! The probe data source, the twin of tools/oracle/tkmeshvs_probe_source.cxx.
use super::print::{Reader, attempt, end, flag, int, real};
use martensite_tkmeshvs::flags::*;
use martensite_tkmeshvs::{
    MeshVSDataSource, MeshVSDataSource3D, MeshVSDataSource3DBase, MeshVSDeformedDataSource,
    MeshVSEntityType, MeshVSPackedMap, MeshVSTopologyHandle,
};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use rustycad_tkmath::gp::Vector3;

pub const MV_NODES: [[f64; 3]; 14] = [
    [0., 0., 0.],
    [1., 0., 0.],
    [1., 1., 0.],
    [0., 1., 0.],
    [2., 0., 0.5],
    [2., 1., 0.25],
    [0.5, 0.5, 1.],
    [3., 0., 0.],
    [3., 1., 0.],
    [4., 0.5, 0.],
    [0., 0., 2.],
    [1., 0., 2.],
    [1., 1., 2.5],
    [5., 5., 5.],
];
pub struct MvElement {
    pub kind: MeshVSEntityType,
    pub nodes: &'static [i32],
}
pub const MV_ELEMENTS: [MvElement; 8] = [
    MvElement {
        kind: MESHVS_ET_FACE,
        nodes: &[1, 2, 3],
    },
    MvElement {
        kind: MESHVS_ET_FACE,
        nodes: &[2, 5, 6, 3],
    },
    MvElement {
        kind: MESHVS_ET_FACE,
        nodes: &[5, 8, 10, 9, 6],
    },
    MvElement {
        kind: MESHVS_ET_LINK,
        nodes: &[3, 4],
    },
    MvElement {
        kind: MESHVS_ET_LINK,
        nodes: &[2, 3],
    },
    MvElement {
        kind: MESHVS_ET_VOLUME,
        nodes: &[1, 2, 3, 11, 12, 13],
    },
    MvElement {
        kind: MESHVS_ET_0D,
        nodes: &[7],
    },
    MvElement {
        kind: MESHVS_ET_FACE,
        nodes: &[1, 3, 4],
    },
];
struct MvGroup {
    id: i32,
    kind: MeshVSEntityType,
    ids: &'static [i32],
}
const MV_GROUPS: [MvGroup; 5] = [
    MvGroup {
        id: 201,
        kind: MESHVS_ET_NODE,
        ids: &[1, 2, 7, 14],
    },
    MvGroup {
        id: 202,
        kind: MESHVS_ET_FACE,
        ids: &[101, 102, 103],
    },
    MvGroup {
        id: 203,
        kind: MESHVS_ET_LINK,
        ids: &[104, 105],
    },
    MvGroup {
        id: 204,
        kind: MESHVS_ET_VOLUME,
        ids: &[106],
    },
    MvGroup {
        id: 205,
        kind: MESHVS_ET_0D,
        ids: &[107],
    },
];

pub struct MvSource {
    transient: StandardTransient,
    base: MeshVSDataSource3DBase,
    variant: i32,
    nodes: MeshVSPackedMap,
    elements: MeshVSPackedMap,
}
rustycad_tkernel::impl_transient!(MvSource, "MvSource", dyn MeshVSDataSource3D, transient,
    interfaces: [dyn MeshVSDataSource, dyn MeshVSDataSource3D]);

impl MvSource {
    pub fn new(variant: i32, node_mask: i32, elem_mask: i32) -> Self {
        let (mut nodes, mut elements) = (MeshVSPackedMap::default(), MeshVSPackedMap::default());
        for i in 0..14 {
            if node_mask & (1 << i) != 0 {
                nodes.add(i + 1);
            }
        }
        for i in 0..8 {
            if elem_mask & (1 << i) != 0 {
                elements.add(101 + i);
            }
        }
        Self {
            transient: StandardTransient::default(),
            base: MeshVSDataSource3DBase::default(),
            variant,
            nodes,
            elements,
        }
    }
    fn fill(coords: &mut Array1<'_, f64>, ids: &[i32]) -> bool {
        for value in coords.iter_mut() {
            *value = 0.;
        }
        if coords.length() < 3 * ids.len() as i32 {
            return false;
        }
        let low = coords.lower();
        for (k, id) in ids.iter().enumerate() {
            for a in 0..3 {
                coords[low + 3 * k as i32 + a as i32] = MV_NODES[(*id - 1) as usize][a];
            }
        }
        true
    }
    fn element(id: i32) -> &'static MvElement {
        &MV_ELEMENTS[(id - 101) as usize]
    }
}

pub fn mv_source(variant: i32) -> Handle<MvSource> {
    Handle::new(MvSource::new(variant, 0x3FFF, 0xFF))
}

impl MeshVSDataSource for MvSource {
    fn get_geom(
        &self,
        id: i32,
        is_element: bool,
        coords: &mut Array1<'_, f64>,
        nb_nodes: &mut i32,
        kind: &mut MeshVSEntityType,
    ) -> bool {
        if !is_element {
            if !self.nodes.contains(id) {
                return false;
            }
            *nb_nodes = 1;
            *kind = MESHVS_ET_NODE;
            return Self::fill(coords, &[id]);
        }
        if !self.elements.contains(id) {
            return false;
        }
        let e = Self::element(id);
        *nb_nodes = e.nodes.len() as i32;
        *kind = e.kind;
        Self::fill(coords, e.nodes)
    }
    fn get_geom_type(&self, id: i32, is_element: bool, kind: &mut MeshVSEntityType) -> bool {
        if !is_element {
            if !self.nodes.contains(id) {
                return false;
            }
            *kind = MESHVS_ET_NODE;
            return true;
        }
        if !self.elements.contains(id) {
            return false;
        }
        *kind = Self::element(id).kind;
        true
    }
    fn get_3d_geom(&self, id: i32, nb_nodes: &mut i32, data: &mut MeshVSTopologyHandle) -> bool {
        if !self.elements.contains(id) || Self::element(id).kind != MESHVS_ET_VOLUME {
            return false;
        }
        *nb_nodes = 6;
        *data = self.get_prism_topology(3);
        true
    }
    fn get_addr(&self, _: i32, _: bool) -> usize {
        0
    }
    fn get_nodes_by_element(
        &self,
        id: i32,
        node_ids: &mut Array1<'_, i32>,
        nb_nodes: &mut i32,
    ) -> bool {
        if !self.elements.contains(id) {
            return false;
        }
        let e = Self::element(id);
        if node_ids.length() < e.nodes.len() as i32 {
            return false;
        }
        *nb_nodes = e.nodes.len() as i32;
        let low = node_ids.lower();
        for (k, node) in e.nodes.iter().enumerate() {
            node_ids[low + k as i32] = *node;
        }
        true
    }
    fn get_all_nodes(&self) -> MeshVSPackedMap {
        let mut copy = MeshVSPackedMap::default();
        copy.assign(&self.nodes);
        copy
    }
    fn get_all_elements(&self) -> MeshVSPackedMap {
        let mut copy = MeshVSPackedMap::default();
        copy.assign(&self.elements);
        copy
    }
    fn get_node_normal(
        &self,
        rank: i32,
        id: i32,
        nx: &mut f64,
        ny: &mut f64,
        nz: &mut f64,
    ) -> bool {
        if self.variant & 2 == 0
            || !self.elements.contains(id)
            || Self::element(id).kind != MESHVS_ET_FACE
        {
            return false;
        }
        let e = Self::element(id);
        if rank < 1 || rank > e.nodes.len() as i32 {
            return false;
        }
        let p = MV_NODES[(e.nodes[(rank - 1) as usize] - 1) as usize];
        (*nx, *ny, *nz) = (0.1 * p[0], 0.2 * p[1], 1. + p[2]);
        true
    }
    fn get_all_groups(&self, ids: &mut MeshVSPackedMap) {
        if self.variant & 1 == 0 {
            return;
        }
        for g in &MV_GROUPS {
            ids.add(g.id);
        }
    }
    fn get_group(&self, id: i32, kind: &mut MeshVSEntityType, ids: &mut MeshVSPackedMap) -> bool {
        if self.variant & 1 == 0 || !(201..=205).contains(&id) {
            *kind = MESHVS_ET_NONE;
            return false;
        }
        let g = &MV_GROUPS[(id - 201) as usize];
        *kind = g.kind;
        for k in g.ids {
            ids.add(*k);
        }
        true
    }
    fn is_advanced_selection_enabled(&self) -> bool {
        self.variant & 4 != 0
    }
}

impl MeshVSDataSource3D for MvSource {
    fn data_source_3d_base(&self) -> &MeshVSDataSource3DBase {
        &self.base
    }
}

pub fn mv_map(map: &MeshVSPackedMap) {
    int(map.extent());
    for key in map.iter() {
        int(key);
    }
}
pub fn mv_hmap(map: &Handle<HPackedMapOfInteger>) {
    flag(map.is_null());
    if let Some(map) = map.get() {
        mv_map(&map.map());
    }
}
fn mv_reals(array: &Handle<HArray1<f64>>) {
    flag(array.is_null());
    if let Some(array) = array.get() {
        let a = array.array1();
        int(a.lower());
        int(a.length());
        for v in a.iter() {
            real(*v);
        }
    }
}
pub fn mv_topo(topo: &MeshVSTopologyHandle) {
    flag(topo.is_null());
    if let Some(topo) = topo.get() {
        let faces = topo.array1();
        int(faces.length());
        for face in faces.iter() {
            int(face.length());
            for v in face.iter() {
                int(*v);
            }
        }
    }
}

/// `mv_ds Variant NodeMask ElemMask MaxNodes`.
pub fn ds_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (variant, node_mask, elem_mask, max_nodes) =
        (r.integer(), r.integer(), r.integer(), r.integer());
    let s = MvSource::new(variant, node_mask, elem_mask);
    for id in 101..=108 {
        let mut n = [-7.0; 3];
        let [x, y, z] = &mut n;
        flag(s.get_normal(id, max_nodes, x, y, z));
        for v in n {
            real(v);
        }
        for nodal in [false, true] {
            let mut a = Handle::default();
            flag(s.get_normals_by_element(id, nodal, max_nodes, &mut a));
            mv_reals(&a);
        }
    }
    let b = s.get_bounding_box();
    flag(b.is_void());
    if let Ok(v) = b.get() {
        for x in v {
            real(x);
        }
    }
    let mut groups = MeshVSPackedMap::default();
    s.get_all_groups(&mut groups);
    mv_map(&groups);
    let (mut t, mut ids) = (MESHVS_ET_ALL, MeshVSPackedMap::default());
    flag(s.get_group(203, &mut t, &mut ids));
    int(t.0);
    mv_map(&ids);
    flag(s.get_group_addr(203) == 0);
    flag(s.is_advanced_selection_enabled());
    end();
}

/// `mv_topology Kind BasePoints`.
pub fn topology_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, n) = (r.integer(), r.integer());
    let s = MvSource::new(0, 0x3FFF, 0xFF);
    mv_topo(&if kind == 0 {
        MeshVSDataSource3DBase::create_prism_topology(n)
    } else {
        MeshVSDataSource3DBase::create_pyramid_topology(n)
    });
    let get = || {
        if kind == 0 {
            s.get_prism_topology(n)
        } else {
            s.get_pyramid_topology(n)
        }
    };
    let (first, second) = (get(), get());
    flag(first.is_null());
    flag(first.object_address() == second.object_address());
    end();
}

/// `mv_deformed Magnify VectorMask Magnify2`.
pub fn deformed_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (magnify, mask, magnify2) = (r.real(), r.integer(), r.real());
    let s: Handle<dyn MeshVSDataSource> = mv_source(0).cast();
    let d = MeshVSDeformedDataSource::new(&s, magnify);
    real(d.get_magnify());
    for i in 1..=14 {
        if mask & (1 << (i - 1)) != 0 {
            let f = f64::from(i);
            d.set_vector(i, &Vector3::new(0.1 * f, -0.2 * f, 0.05 * f * f));
        }
    }
    d.set_vector(1, &Vector3::new(1., 2., 3.));
    if mask & 1 != 0 {
        d.set_vector(1, &Vector3::new(0.1, -0.2, 0.05));
    }
    if magnify2 != 0.0 {
        d.set_magnify(magnify2);
    }
    real(d.get_magnify());
    int(d.get_vectors().extent() as i32);
    let mut c = Array1::<f64>::with_bounds(1, 30).unwrap();
    for (is_element, ids) in [(false, 1..=14), (true, 101..=108)] {
        for id in ids {
            let (mut nb, mut t) = (-1, MESHVS_ET_ALL);
            attempt(|| {
                let ok = d.get_geom(id, is_element, &mut c, &mut nb, &mut t);
                flag(ok);
                if ok {
                    int(nb);
                    int(t.0);
                    for k in 1..=3 * nb {
                        real(c[k]);
                    }
                }
                Ok(())
            });
        }
    }
    let mut t = MESHVS_ET_ALL;
    flag(d.get_geom_type(106, true, &mut t));
    int(t.0);
    let (mut nb, mut topo) = (0, MeshVSTopologyHandle::default());
    flag(d.get_3d_geom(106, &mut nb, &mut topo));
    int(nb);
    mv_topo(&topo);
    mv_map(&d.get_all_nodes());
    mv_map(&d.get_all_elements());
    flag(d.get_non_deformed_data_source().object_address() == s.object_address());
    d.set_non_deformed_data_source(&Handle::default());
    mv_map(&d.get_all_nodes());
    flag(d.get_geom(1, false, &mut c, &mut nb, &mut t));
    end();
}
