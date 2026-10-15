// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_DeformedDataSource` at OCCT 3d097a0: another data source with every node moved
//! by its magnified vector. A node without a vector makes its geometry (and the geometry of
//! its elements) unknown, as natively.
//!
//! ```
//! use martensite_tkmeshvs::{MeshVSDataSource, MeshVSDeformedDataSource};
//! use rustycad_tkernel::standard::Handle;
//! let deformed = MeshVSDeformedDataSource::new(&Handle::default(), -2.0);
//! assert_eq!(deformed.get_magnify(), 1.0);
//! assert!(deformed.get_all_nodes().is_empty());
//! ```
use crate::MeshVSEntityType;
use crate::data_source::{MeshVSDataSource, MeshVSPackedMap, MeshVSTopologyHandle};
use crate::flags::MESHVS_ET_NODE;
use rustycad_tkernel::ncollection::{Array1, DataMap};
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::Vector3;

/// The private fields of `MeshVS_DeformedDataSource`.
#[derive(Default)]
pub struct MeshVSDeformedDataSourceFields {
    /// `myNonDeformedDataSource`.
    pub non_deformed: Handle<dyn MeshVSDataSource>,
    /// `myVectors`.
    pub vectors: DataMap<i32, Vector3>,
    /// `myMagnify`.
    pub magnify: f64,
}

/// A data source deformed by nodal vectors; `MeshVSDeformedDataSource::new(&source, 2.0)`.
pub struct MeshVSDeformedDataSource {
    transient: StandardTransient,
    /// The private fields; `deformed.fields.read().magnify`.
    pub fields: SharedState<MeshVSDeformedDataSourceFields>,
}
rustycad_tkernel::impl_transient!(MeshVSDeformedDataSource, "MeshVS_DeformedDataSource", dyn MeshVSDataSource, transient,
    interfaces: [dyn MeshVSDataSource]);

impl MeshVSDeformedDataSource {
    /// `MeshVS_DeformedDataSource(theNonDeformDS, theMagnify)`; a magnify <= 0 becomes 1.
    /// Example: `MeshVSDeformedDataSource::new(&source, 3.0)`.
    pub fn new(non_deformed: &Handle<dyn MeshVSDataSource>, magnify: f64) -> Self {
        let source = Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(MeshVSDeformedDataSourceFields {
                non_deformed: non_deformed.clone(),
                ..Default::default()
            }),
        };
        source.set_magnify(magnify);
        source
    }
    /// `GetVectors`: a copy of the nodal vectors; `deformed.get_vectors().extent()`.
    pub fn get_vectors(&self) -> DataMap<i32, Vector3> {
        let mut copy = DataMap::default();
        copy.assign(&self.fields.read().vectors);
        copy
    }
    /// `SetVectors`: replaces every nodal vector; `deformed.set_vectors(&map)`.
    pub fn set_vectors(&self, map: &DataMap<i32, Vector3>) {
        self.fields.write().vectors.assign(map);
    }
    /// `GetVector`: the vector of a node, the output kept without; `deformed.get_vector(3, &mut v)`.
    pub fn get_vector(&self, id: i32, vect: &mut Vector3) -> bool {
        match self.fields.read().vectors.seek(&id) {
            Some(found) => {
                *vect = *found;
                true
            }
            None => false,
        }
    }
    /// `SetVector`: binds or replaces the vector of a node; `deformed.set_vector(3, &v)`.
    pub fn set_vector(&self, id: i32, vect: &Vector3) {
        let mut fields = self.fields.write();
        match fields.vectors.change_find(&id) {
            Ok(slot) => *slot = *vect,
            Err(_) => {
                fields.vectors.bind(id, *vect);
            }
        }
    }
    /// `SetNonDeformedDataSource`; `deformed.set_non_deformed_data_source(&source)`.
    pub fn set_non_deformed_data_source(&self, source: &Handle<dyn MeshVSDataSource>) {
        self.fields.write().non_deformed = source.clone();
    }
    /// `GetNonDeformedDataSource`; `deformed.get_non_deformed_data_source().is_null()`.
    pub fn get_non_deformed_data_source(&self) -> Handle<dyn MeshVSDataSource> {
        self.fields.read().non_deformed.clone()
    }
    /// `SetMagnify`: a value <= 0 becomes 1; `deformed.set_magnify(0.5)`.
    pub fn set_magnify(&self, magnify: f64) {
        self.fields.write().magnify = if magnify <= 0.0 { 1.0 } else { magnify };
    }
    /// `GetMagnify`; `deformed.get_magnify()`.
    pub fn get_magnify(&self) -> f64 {
        self.fields.read().magnify
    }
    /// Shifts the coordinates of node `i` (one-based) by the magnified vector of `node_id`.
    fn shift(&self, coords: &mut Array1<'_, f64>, i: i32, node_id: i32) -> bool {
        let mut vect = Vector3::new(0.0, 0.0, 0.0);
        if !self.get_vector(node_id, &mut vect) {
            return false;
        }
        let magnify = self.get_magnify();
        let shift = [vect.x(), vect.y(), vect.z()].map(|c| magnify * c);
        for (axis, value) in shift.into_iter().enumerate() {
            match coords.change_value(3 * i - 2 + axis as i32) {
                Ok(slot) => *slot += value,
                Err(_) => return false,
            }
        }
        true
    }
    /// Shifts every node of an element; false at the first node without vector.
    fn shift_element(
        &self,
        source: &dyn MeshVSDataSource,
        id: i32,
        coords: &mut Array1<'_, f64>,
        nb_nodes: &mut i32,
    ) -> bool {
        let Ok(mut nodes) = Array1::<i32>::with_bounds(1, *nb_nodes) else {
            return false;
        };
        if !source.get_nodes_by_element(id, &mut nodes, nb_nodes) {
            return false;
        }
        (1..=*nb_nodes).all(|i| {
            nodes
                .value(i)
                .is_ok_and(|&node| self.shift(coords, i, node))
        })
    }
}

impl MeshVSDataSource for MeshVSDeformedDataSource {
    fn get_geom(
        &self,
        id: i32,
        is_element: bool,
        coords: &mut Array1<'_, f64>,
        nb_nodes: &mut i32,
        kind: &mut MeshVSEntityType,
    ) -> bool {
        let source = self.get_non_deformed_data_source();
        let Some(source) = source.get() else {
            return false;
        };
        if !source.get_geom(id, is_element, coords, nb_nodes, kind) {
            return false;
        }
        if *kind == MESHVS_ET_NODE {
            return self.shift(coords, 1, id);
        }
        self.shift_element(source, id, coords, nb_nodes)
    }
    fn get_geom_type(&self, id: i32, is_element: bool, kind: &mut MeshVSEntityType) -> bool {
        let source = self.get_non_deformed_data_source();
        source
            .get()
            .is_some_and(|s| s.get_geom_type(id, is_element, kind))
    }
    fn get_3d_geom(&self, id: i32, nb_nodes: &mut i32, data: &mut MeshVSTopologyHandle) -> bool {
        let source = self.get_non_deformed_data_source();
        source
            .get()
            .is_some_and(|s| s.get_3d_geom(id, nb_nodes, data))
    }
    fn get_addr(&self, id: i32, is_element: bool) -> usize {
        let source = self.get_non_deformed_data_source();
        source.get().map_or(0, |s| s.get_addr(id, is_element))
    }
    fn get_nodes_by_element(
        &self,
        id: i32,
        node_ids: &mut Array1<'_, i32>,
        nb_nodes: &mut i32,
    ) -> bool {
        let source = self.get_non_deformed_data_source();
        source
            .get()
            .is_some_and(|s| s.get_nodes_by_element(id, node_ids, nb_nodes))
    }
    /// The nodes of the non-deformed source, else the empty `myEmptyMap`.
    fn get_all_nodes(&self) -> MeshVSPackedMap {
        let source = self.get_non_deformed_data_source();
        source.get().map(|s| s.get_all_nodes()).unwrap_or_default()
    }
    /// The elements of the non-deformed source, else the empty `myEmptyMap`.
    fn get_all_elements(&self) -> MeshVSPackedMap {
        let source = self.get_non_deformed_data_source();
        source
            .get()
            .map(|s| s.get_all_elements())
            .unwrap_or_default()
    }
}
