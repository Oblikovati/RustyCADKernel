//! `Graphic3d_BvhCStructureSet` and `Graphic3d_BvhCStructureSetTrsfPers`: the BVH sets of
//! the cullable structures of a layer, without and with transform persistence.
use super::{
    Graphic3dBndBox3d, Graphic3dCStructure, Graphic3dCStructureMembers as _, Graphic3dCamera,
    Graphic3dTransformPersApply as _, Graphic3dWorldViewProjState,
};
use rustycad_tkernel::ncollection::{IndexedMap, Mat4};
use rustycad_tkernel::standard::{Failure, Handle, WeakHandle};
use rustycad_tkmath::bvh::constants::{LEAF_NODE_SIZE_SINGLE, MAX_TREE_DEPTH, NB_BINS_BEST};
use rustycad_tkmath::bvh::{
    BvhBinnedBuilder, BvhBox, BvhBuilder, BvhObject, BvhObjectTransient, BvhPrimitiveSet,
    BvhPrimitiveSetAccess, BvhSet, BvhTree, BvhVec3d,
};
use std::sync::Arc;

/// The structures of a set (raw pointers natively), with one-based dense indices.
pub type Graphic3dCStructureIndexedMap = IndexedMap<WeakHandle<dyn Graphic3dCStructure>>;

/// The builder handle of the BVH sets (`occ::handle<BVH_Builder3d>`, null for none).
pub type Graphic3dBvhBuilder3d = Option<Arc<dyn BvhBuilder<BvhVec3d>>>;

/// The bounding box of the structure at a one-based index (void when it is gone).
fn structure_box(structs: &Graphic3dCStructureIndexedMap, index: usize) -> Graphic3dBndBox3d {
    let structure = structs
        .find_key(index)
        .map(WeakHandle::upgrade)
        .unwrap_or_default();
    structure
        .get()
        .map(|s| s.bounding_box())
        .unwrap_or_default()
}

/// The component of a corner along an axis.
fn axis_value(corner: &BvhVec3d, axis: i32) -> f64 {
    match axis {
        0 => corner.x,
        1 => corner.y,
        _ => corner.z,
    }
}

/// `Add`: a new structure; true when it was not in the set.
fn add_structure(
    structs: &mut Graphic3dCStructureIndexedMap,
    structure: &Handle<dyn Graphic3dCStructure>,
) -> bool {
    let size = structs.extent();
    structs.add(structure.downgrade()) > size
}

/// `Remove`: swaps the structure with the last one and removes it; true when it was present.
fn remove_structure(
    structs: &mut Graphic3dCStructureIndexedMap,
    structure: &Handle<dyn Graphic3dCStructure>,
) -> bool {
    let index = structs.find_index(&structure.downgrade());
    if index == 0 {
        return false;
    }
    let last = structs.extent();
    // Both indices are in 1..=extent, so the swap and the removal cannot fail.
    let _ = structs.swap(last, index);
    let _ = structs.remove_last();
    true
}

/// `GetStructureById`: the structure at a zero-based index (null when out of range).
fn structure_by_id(
    structs: &Graphic3dCStructureIndexedMap,
    id: i32,
) -> Handle<dyn Graphic3dCStructure> {
    let index = usize::try_from(id + 1).unwrap_or(0);
    structs
        .find_key(index)
        .map(WeakHandle::upgrade)
        .unwrap_or_default()
}

/// The set of the cullable structures without transform persistence; a binned builder with
/// one structure per leaf. Example: `Graphic3dBvhCStructureSet::default().size()`.
pub struct Graphic3dBvhCStructureSet {
    primitive_set: BvhPrimitiveSet<BvhVec3d>,
    structs: Graphic3dCStructureIndexedMap,
}

impl Default for Graphic3dBvhCStructureSet {
    fn default() -> Self {
        let builder = BvhBinnedBuilder::<BvhVec3d, NB_BINS_BEST>::new(
            LEAF_NODE_SIZE_SINGLE,
            MAX_TREE_DEPTH,
            false,
            1,
        );
        Self {
            primitive_set: BvhPrimitiveSet::new(Some(Arc::new(builder))),
            structs: Graphic3dCStructureIndexedMap::default(),
        }
    }
}

impl Graphic3dBvhCStructureSet {
    /// `Add`; marks the set dirty for a new structure; `set.add(&cstructure)`.
    pub fn add(&mut self, structure: &Handle<dyn Graphic3dCStructure>) -> bool {
        let added = add_structure(&mut self.structs, structure);
        if added {
            self.mark_dirty();
        }
        added
    }
    /// `Remove`; marks the set dirty when found; `set.remove(&cstructure)`.
    pub fn remove(&mut self, structure: &Handle<dyn Graphic3dCStructure>) -> bool {
        let removed = remove_structure(&mut self.structs, structure);
        if removed {
            self.mark_dirty();
        }
        removed
    }
    /// `Clear`; `set.clear()`.
    pub fn clear(&mut self) {
        self.structs.clear(true);
        self.mark_dirty();
    }
    /// `GetStructureById`; `set.get_structure_by_id(0)`.
    pub fn get_structure_by_id(&self, id: i32) -> Handle<dyn Graphic3dCStructure> {
        structure_by_id(&self.structs, id)
    }
    /// `Structures`; `set.structures().extent()`.
    pub fn structures(&self) -> &Graphic3dCStructureIndexedMap {
        &self.structs
    }
}

impl BvhSet<BvhVec3d> for Graphic3dBvhCStructureSet {
    fn size(&self) -> i32 {
        self.structs.extent() as i32
    }
    fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        Ok(structure_box(&self.structs, (index + 1) as usize))
    }
    fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        let bounds = structure_box(&self.structs, (index + 1) as usize);
        Ok((axis_value(bounds.corner_min(), axis) + axis_value(bounds.corner_max(), axis)) * 0.5)
    }
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.structs
            .swap((first + 1) as usize, (second + 1) as usize)
    }
    fn bounding_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.cached_box()
    }
}

impl BvhPrimitiveSetAccess<BvhVec3d> for Graphic3dBvhCStructureSet {
    fn primitive_set(&self) -> &BvhPrimitiveSet<BvhVec3d> {
        &self.primitive_set
    }
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet<BvhVec3d> {
        &mut self.primitive_set
    }
}

impl BvhObject<BvhVec3d> for Graphic3dBvhCStructureSet {
    fn object_transient(&self) -> &BvhObjectTransient {
        self.primitive_set.object_transient()
    }
    fn object_transient_mut(&mut self) -> &mut BvhObjectTransient {
        self.primitive_set.object_transient_mut()
    }
    fn object_box(&self) -> Result<BvhBox<BvhVec3d>, Failure> {
        self.cached_box()
    }
}

/// The view parameters the transform persistence boxes depend on.
pub struct Graphic3dTrsfPersView<'a> {
    /// The camera.
    pub camera: &'a Handle<Graphic3dCamera>,
    /// The projection matrix.
    pub projection: &'a Mat4<f64>,
    /// The world-view matrix.
    pub world_view: &'a Mat4<f64>,
    /// The viewport `[width, height]`.
    pub viewport: [i32; 2],
    /// The world-view-projection state of the camera.
    pub state: &'a Graphic3dWorldViewProjState,
}

/// The set of the cullable structures with transform persistence: their boxes depend on the
/// view, so the tree is rebuilt when the view state changes.
/// Example: `Graphic3dBvhCStructureSetTrsfPers::new(None).size()`.
pub struct Graphic3dBvhCStructureSetTrsfPers {
    /// `myIsDirty`.
    is_dirty: bool,
    /// `myBVH`.
    bvh: BvhTree<BvhVec3d>,
    /// `myBuilder`.
    builder: Graphic3dBvhBuilder3d,
    /// `myStructs`.
    structs: Graphic3dCStructureIndexedMap,
    /// `myStructBoxes`: the boxes moved by the persistence, during a build.
    struct_boxes: Vec<Graphic3dBndBox3d>,
    /// `myStructBoxesState`.
    struct_boxes_state: Graphic3dWorldViewProjState,
}

impl Graphic3dBvhCStructureSetTrsfPers {
    /// An empty set with a builder (may be null); `Graphic3dBvhCStructureSetTrsfPers::new(None)`.
    pub fn new(builder: Graphic3dBvhBuilder3d) -> Self {
        Self {
            is_dirty: false,
            bvh: BvhTree::default(),
            builder,
            structs: Graphic3dCStructureIndexedMap::default(),
            struct_boxes: Vec::new(),
            struct_boxes_state: Graphic3dWorldViewProjState::default(),
        }
    }
    /// `Add`; `set.add(&cstructure)`.
    pub fn add(&mut self, structure: &Handle<dyn Graphic3dCStructure>) -> bool {
        let added = add_structure(&mut self.structs, structure);
        self.is_dirty |= added;
        added
    }
    /// `Remove`; `set.remove(&cstructure)`.
    pub fn remove(&mut self, structure: &Handle<dyn Graphic3dCStructure>) -> bool {
        let removed = remove_structure(&mut self.structs, structure);
        self.is_dirty |= removed;
        removed
    }
    /// `Clear`; `set.clear()`.
    pub fn clear(&mut self) {
        self.structs.clear(true);
        self.is_dirty = true;
    }
    /// `GetStructureById`; `set.get_structure_by_id(0)`.
    pub fn get_structure_by_id(&self, id: i32) -> Handle<dyn Graphic3dCStructure> {
        structure_by_id(&self.structs, id)
    }
    /// `Structures`; `set.structures().extent()`.
    pub fn structures(&self) -> &Graphic3dCStructureIndexedMap {
        &self.structs
    }
    /// `MarkDirty`; `set.mark_dirty()`.
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }
    /// `Builder`; `set.builder()`.
    pub fn builder(&self) -> &Graphic3dBvhBuilder3d {
        &self.builder
    }
    /// `SetBuilder`; `set.set_builder(None)`.
    pub fn set_builder(&mut self, builder: Graphic3dBvhBuilder3d) {
        self.builder = builder;
    }
    /// `BVH`: the tree of the persistent boxes, rebuilt when dirty or when the view state
    /// changed; a null builder fails as `Standard_NullObject` (natively a null dereference).
    /// Example: `set.bvh(&view)?`.
    pub fn bvh(&mut self, view: &Graphic3dTrsfPersView<'_>) -> Result<&BvhTree<BvhVec3d>, Failure> {
        if !self.is_dirty
            && self.struct_boxes_state.is_valid()
            && !self.struct_boxes_state.is_changed(view.state)
        {
            return Ok(&self.bvh);
        }
        self.struct_boxes = self.persistent_boxes(view)?;
        let builder = self.builder.clone().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "Graphic3d_BvhCStructureSetTrsfPers: null BVH builder; expected a builder",
                "",
            )
        })?;
        let bounding = rustycad_tkmath::bvh::combined_element_boxes(self)?;
        let mut tree = std::mem::take(&mut self.bvh);
        let built = builder.build(self, &mut tree, &bounding);
        self.bvh = tree;
        built?;
        self.struct_boxes_state = *view.state;
        self.struct_boxes.clear();
        self.is_dirty = false;
        Ok(&self.bvh)
    }
    /// The structure boxes moved by their transform persistence.
    fn persistent_boxes(
        &self,
        view: &Graphic3dTrsfPersView<'_>,
    ) -> Result<Vec<Graphic3dBndBox3d>, Failure> {
        let mut boxes = Vec::with_capacity(self.structs.extent());
        for index in 1..=self.structs.extent() {
            let structure = self
                .structs
                .find_key(index)
                .map(WeakHandle::upgrade)
                .unwrap_or_default();
            let mut bounds = structure
                .get()
                .map(|s| s.bounding_box())
                .unwrap_or_default();
            let pers = structure
                .get()
                .map(|s| s.transform_persistence())
                .unwrap_or_default();
            if let Some(pers) = pers.get() {
                pers.apply_bvh(
                    view.camera,
                    view.projection,
                    view.world_view,
                    view.viewport,
                    &mut bounds,
                )?;
            }
            boxes.push(bounds);
        }
        Ok(boxes)
    }
    /// The persistent box at a zero-based index (void outside a build).
    fn struct_box(&self, index: i32) -> Graphic3dBndBox3d {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.struct_boxes.get(i))
            .copied()
            .unwrap_or_default()
    }
}

impl BvhSet<BvhVec3d> for Graphic3dBvhCStructureSetTrsfPers {
    fn size(&self) -> i32 {
        self.structs.extent() as i32
    }
    fn element_box(&self, index: i32) -> Result<BvhBox<BvhVec3d>, Failure> {
        Ok(self.struct_box(index))
    }
    fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        let bounds = self.struct_box(index);
        Ok((axis_value(bounds.corner_min(), axis) + axis_value(bounds.corner_max(), axis)) * 0.5)
    }
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.structs
            .swap((first + 1) as usize, (second + 1) as usize)?;
        let (a, b) = (first as usize, second as usize);
        if a < self.struct_boxes.len() && b < self.struct_boxes.len() {
            self.struct_boxes.swap(a, b);
        }
        Ok(())
    }
}
