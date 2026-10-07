//! `Graphic3d_Layer`: the structures of a Z layer by display priority, with the BVH sets
//! used for frustum culling and the cached bounding boxes.
use super::bvh_cstructure_set::{Graphic3dBvhBuilder3d, Graphic3dCStructureIndexedMap};
use super::{
    GRAPHIC3D_DISPLAY_PRIORITY_NB, Graphic3dBvhCStructureSet, Graphic3dBvhCStructureSetTrsfPers,
    Graphic3dCStructure, Graphic3dCStructureMembers as _, Graphic3dDisplayPriority,
    Graphic3dZLayerId, Graphic3dZLayerSettings,
};
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Xyz;
use std::sync::{Mutex, MutexGuard};

/// `Graphic3d_ArrayOfIndexedMapOfStructure`: the structures of each display priority.
pub type Graphic3dArrayOfIndexedMapOfStructure = Vec<Graphic3dCStructureIndexedMap>;

/// The mutable state of a layer.
pub(crate) struct LayerState {
    /// `myArray`: the structures of each priority.
    pub(crate) array: Graphic3dArrayOfIndexedMapOfStructure,
    /// `myNbStructures`.
    pub(crate) nb_structures: i32,
    /// `myNbStructuresNotCulled`.
    pub(crate) nb_structures_not_culled: i32,
    /// `myLayerSettings`.
    pub(crate) layer_settings: Graphic3dZLayerSettings,
    /// `myAlwaysRenderedMap`: the structures that are never culled.
    pub(crate) always_rendered: Graphic3dCStructureIndexedMap,
    /// `myBVHIsLeftChildQueuedFirst`: alternates the traversal order.
    pub(crate) bvh_is_left_child_queued_first: bool,
    /// `myIsBVHPrimitivesNeedsReset`.
    pub(crate) is_bvh_primitives_needs_reset: bool,
    /// `myIsBoundingBoxNeedsReset`: without and with the auxiliary structures.
    pub(crate) is_bounding_box_needs_reset: [bool; 2],
    /// `myBoundingBox`: the cached boxes.
    pub(crate) bounding_box: [BoundingBox3; 2],
}

/// The BVH sets of a layer (`mutable` members natively).
pub(crate) struct LayerBvh {
    /// `myBVHPrimitives`.
    pub(crate) primitives: Graphic3dBvhCStructureSet,
    /// `myBVHPrimitivesTrsfPers`.
    pub(crate) trsf_pers: Graphic3dBvhCStructureSetTrsfPers,
}

/// A Z layer; `Graphic3dLayer::new(GRAPHIC3D_ZLAYER_ID_DEFAULT, None)`.
pub struct Graphic3dLayer {
    transient: StandardTransient,
    /// `myLayerId`.
    layer_id: Graphic3dZLayerId,
    pub(crate) state: SharedState<LayerState>,
    pub(crate) bvh: Mutex<LayerBvh>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dLayer,
    "Graphic3d_Layer",
    StandardTransient,
    transient
);

/// The priority index of the native `std::min(std::max(priority, Bottom), Topmost)`.
fn priority_index(priority: Graphic3dDisplayPriority) -> usize {
    (priority as i32).clamp(0, GRAPHIC3D_DISPLAY_PRIORITY_NB - 1) as usize
}

impl Graphic3dLayer {
    /// A layer with the frustum culling builder of the persistent structures (may be null);
    /// `Graphic3dLayer::new(GRAPHIC3D_ZLAYER_ID_DEFAULT, None)`.
    pub fn new(layer_id: Graphic3dZLayerId, builder: Graphic3dBvhBuilder3d) -> Self {
        let state = LayerState {
            array: (0..GRAPHIC3D_DISPLAY_PRIORITY_NB)
                .map(|_| Graphic3dCStructureIndexedMap::default())
                .collect(),
            nb_structures: 0,
            nb_structures_not_culled: 0,
            layer_settings: Graphic3dZLayerSettings::default(),
            always_rendered: Graphic3dCStructureIndexedMap::default(),
            bvh_is_left_child_queued_first: true,
            is_bvh_primitives_needs_reset: false,
            is_bounding_box_needs_reset: [true, true],
            bounding_box: [BoundingBox3::default(), BoundingBox3::default()],
        };
        let bvh = LayerBvh {
            primitives: Graphic3dBvhCStructureSet::default(),
            trsf_pers: Graphic3dBvhCStructureSetTrsfPers::new(builder),
        };
        Self {
            transient: StandardTransient::default(),
            layer_id,
            state: SharedState::new(state),
            bvh: Mutex::new(bvh),
        }
    }
    /// The BVH sets, recovered from a poisoned lock (the sets stay consistent per call).
    pub(crate) fn bvh_sets(&self) -> MutexGuard<'_, LayerBvh> {
        self.bvh
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// `LayerId`; `layer.layer_id()`.
    pub fn layer_id(&self) -> Graphic3dZLayerId {
        self.layer_id
    }
    /// `FrustumCullingBVHBuilder`; `layer.frustum_culling_bvh_builder()`.
    pub fn frustum_culling_bvh_builder(&self) -> Graphic3dBvhBuilder3d {
        self.bvh_sets().trsf_pers.builder().clone()
    }
    /// `SetFrustumCullingBVHBuilder`; `layer.set_frustum_culling_bvh_builder(None)`.
    pub fn set_frustum_culling_bvh_builder(&self, builder: Graphic3dBvhBuilder3d) {
        self.bvh_sets().trsf_pers.set_builder(builder);
    }
    /// `IsImmediate`; `layer.is_immediate()`.
    pub fn is_immediate(&self) -> bool {
        self.state.read().layer_settings.is_immediate()
    }
    /// `LayerSettings`; `layer.layer_settings().name()`.
    pub fn layer_settings(&self) -> Graphic3dZLayerSettings {
        self.state.read().layer_settings.clone()
    }
    /// `SetLayerSettings`: a moved origin updates the layer transformation of every structure;
    /// `layer.set_layer_settings(&settings)`.
    pub fn set_layer_settings(&self, settings: &Graphic3dZLayerSettings) {
        let structures = {
            let mut state = self.state.write();
            let moved = !xyz_equal(&state.layer_settings.origin(), &settings.origin());
            state.layer_settings = settings.clone();
            if !moved {
                return;
            }
            state
                .array
                .iter()
                .flat_map(|map| map.iter().cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>()
        };
        for structure in structures.iter().map(WeakHandle::upgrade) {
            if let Some(structure) = structure.get() {
                structure.update_layer_transformation();
            }
        }
    }
    /// `Add`: adds a structure at a priority (clamped to Bottom..Topmost); outside a
    /// priority change, files it as always rendered or in a BVH set.
    /// Example: `layer.add(&cstructure, Graphic3dDisplayPriority::Normal, false)`.
    pub fn add(
        &self,
        structure: &Handle<dyn Graphic3dCStructure>,
        priority: Graphic3dDisplayPriority,
        is_for_change_priority: bool,
    ) {
        let Some(value) = structure.get() else { return };
        let always_rendered = value.is_always_rendered();
        if always_rendered {
            value.mark_as_not_culled();
        }
        let without_pers = value.transform_persistence().is_null();
        let mut state = self.state.write();
        state.array[priority_index(priority)].add(structure.downgrade());
        if always_rendered && !is_for_change_priority {
            state.always_rendered.add(structure.downgrade());
        } else if !always_rendered && !is_for_change_priority {
            let mut sets = self.bvh_sets();
            if without_pers {
                sets.primitives.add(structure);
            } else {
                sets.trsf_pers.add(structure);
            }
        }
        state.nb_structures += 1;
    }
    /// `Remove`: removes a structure and returns its priority (`Invalid` when absent).
    /// Example: `layer.remove(&cstructure, false)`.
    pub fn remove(
        &self,
        structure: &Handle<dyn Graphic3dCStructure>,
        is_for_change_priority: bool,
    ) -> Graphic3dDisplayPriority {
        let Some(value) = structure.get() else {
            return Graphic3dDisplayPriority::Invalid;
        };
        let always_rendered = value.is_always_rendered();
        let key = structure.downgrade();
        let mut state = self.state.write();
        let Some(priority) = (0..state.array.len()).find(|&p| state.array[p].find_index(&key) != 0)
        else {
            return Graphic3dDisplayPriority::Invalid;
        };
        swap_remove_key(&mut state.array[priority], &key);
        if !is_for_change_priority {
            let mut in_always = always_rendered;
            if !always_rendered {
                let mut sets = self.bvh_sets();
                in_always = !sets.primitives.remove(structure) && !sets.trsf_pers.remove(structure);
            }
            if in_always {
                swap_remove_key(&mut state.always_rendered, &key);
            }
        }
        state.nb_structures -= 1;
        priority_of(priority)
    }
    /// `NbStructures`; `layer.nb_structures()`.
    pub fn nb_structures(&self) -> i32 {
        self.state.read().nb_structures
    }
    /// `NbStructuresNotCulled`; `layer.nb_structures_not_culled()`.
    pub fn nb_structures_not_culled(&self) -> i32 {
        self.state.read().nb_structures_not_culled
    }
    /// `NbPriorities`; `layer.nb_priorities()`.
    pub fn nb_priorities(&self) -> i32 {
        GRAPHIC3D_DISPLAY_PRIORITY_NB
    }
    /// `Append`: adds every structure of `other` at its priority; `layer.append(&other)`.
    pub fn append(&self, other: &Graphic3dLayer) -> bool {
        let array = other.array_of_structures();
        for (priority, structures) in array.iter().enumerate() {
            for structure in structures.iter().map(WeakHandle::upgrade) {
                self.add(&structure, priority_of(priority), false);
            }
        }
        true
    }
    /// `ArrayOfStructures` (a copy); `layer.array_of_structures()[5].extent()`.
    pub fn array_of_structures(&self) -> Graphic3dArrayOfIndexedMapOfStructure {
        self.state.read().array.clone()
    }
    /// `Structures(priority)` (a copy); `layer.structures(Graphic3dDisplayPriority::Normal)`.
    pub fn structures(&self, priority: Graphic3dDisplayPriority) -> Graphic3dCStructureIndexedMap {
        self.state.read().array[priority_index(priority)].clone()
    }
    /// `InvalidateBVHData`; `layer.invalidate_bvh_data()`.
    pub fn invalidate_bvh_data(&self) {
        self.state.write().is_bvh_primitives_needs_reset = true;
    }
    /// `InvalidateBoundingBox`; `layer.invalidate_bounding_box()`.
    pub fn invalidate_bounding_box(&self) {
        self.state.write().is_bounding_box_needs_reset = [true, true];
    }
    /// `IsCulled`: no structure left after the culling; `layer.is_culled()`.
    pub fn is_culled(&self) -> bool {
        self.state.read().nb_structures_not_culled == 0
    }
    /// `NbOfTransformPersistenceObjects`; `layer.nb_of_transform_persistence_objects()`.
    pub fn nb_of_transform_persistence_objects(&self) -> i32 {
        self.bvh_sets().trsf_pers.structures().extent() as i32
    }
    /// `CullableStructuresBVH` (the structures of the set); `layer.cullable_structures()`.
    pub fn cullable_structures(&self) -> Graphic3dCStructureIndexedMap {
        self.bvh_sets().primitives.structures().clone()
    }
    /// `CullableTrsfPersStructuresBVH` (the structures of the set); `layer.cullable_trsf_pers_structures()`.
    pub fn cullable_trsf_pers_structures(&self) -> Graphic3dCStructureIndexedMap {
        self.bvh_sets().trsf_pers.structures().clone()
    }
    /// `NonCullableStructures`; `layer.non_cullable_structures()`.
    pub fn non_cullable_structures(&self) -> Graphic3dCStructureIndexedMap {
        self.state.read().always_rendered.clone()
    }
}

/// The priority of an index of `myArray`.
pub(crate) fn priority_of(index: usize) -> Graphic3dDisplayPriority {
    use Graphic3dDisplayPriority as P;
    const ALL: [P; 11] = [
        P::Bottom,
        P::AlmostBottom,
        P::Below2,
        P::Below1,
        P::Below,
        P::Normal,
        P::Above,
        P::Above1,
        P::Above2,
        P::Highlight,
        P::Topmost,
    ];
    ALL.get(index).copied().unwrap_or(P::Invalid)
}

/// `FindIndex`, `Swap` with the last, `RemoveLast`.
pub(crate) fn swap_remove_key(
    map: &mut Graphic3dCStructureIndexedMap,
    key: &WeakHandle<dyn Graphic3dCStructure>,
) {
    let index = map.find_index(key);
    if index == 0 {
        return;
    }
    let last = map.extent();
    // Both indices are in 1..=extent, so the swap and the removal cannot fail.
    let _ = map.swap(index, last);
    let _ = map.remove_last();
}

/// `gp_XYZ::IsEqual(other, gp::Resolution())`.
fn xyz_equal(first: &Xyz, second: &Xyz) -> bool {
    first.is_equal(second, rustycad_tkmath::gp::resolution())
}
