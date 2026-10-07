//! `Graphic3d_StructureManager`: the displayed and highlighted structures of a viewer, its
//! views and the view affinity of registered objects. No class derives from it natively,
//! so its virtual members are inherent methods (visualization-design.md section 1).
use super::{
    Graphic3dCView, Graphic3dCViewMembers as _, Graphic3dCViewStructures as _,
    Graphic3dDataStructureManager, Graphic3dDisplayPriority, Graphic3dGraphicDriver,
    Graphic3dStructureDisplay as _, Graphic3dStructureNetwork as _, Graphic3dStructureSource,
    Graphic3dTypeOfConnection, Graphic3dViewAffinity, Graphic3dZLayerId,
    graphic3d_structure_network,
};
use crate::aspect::AspectGenId;
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkmath::toploc::Datum3;
use std::collections::HashMap;
use std::io;

/// The structures of a manager, in insertion order (`NCollection_Map` natively, whose
/// iteration order follows the pointer hashes).
pub type Graphic3dStructureSet = IndexedMap<Handle<dyn Graphic3dStructureSource>>;

/// The protected fields of a structure manager.
pub struct Graphic3dStructureManagerFields {
    /// `myViewGenId`: the view identifiers 0..31.
    pub view_gen_id: AspectGenId,
    /// `myDisplayedStructure`.
    pub displayed_structure: Graphic3dStructureSet,
    /// `myHighlightedStructure`.
    pub highlighted_structure: Graphic3dStructureSet,
    /// `myRegisteredObjects`: the view affinity of objects, by object address.
    pub registered_objects: HashMap<usize, Handle<Graphic3dViewAffinity>>,
    /// `myGraphicDriver`.
    pub graphic_driver: Handle<dyn Graphic3dGraphicDriver>,
    /// `myDefinedViews`: the views (raw pointers natively), in index order.
    pub defined_views: Vec<WeakHandle<dyn Graphic3dCView>>,
    /// `myDeviceLostFlag`.
    pub device_lost_flag: bool,
}

/// A structure manager; `Graphic3dStructureManager::new(&driver)?`.
pub struct Graphic3dStructureManager {
    transient: StandardTransient,
    /// The fields, shared by every handle.
    pub fields: SharedState<Graphic3dStructureManagerFields>,
}
rustycad_tkernel::impl_transient!(
    Graphic3dStructureManager,
    "Graphic3d_StructureManager",
    StandardTransient,
    transient
);

impl Drop for Graphic3dStructureManager {
    /// The destructor removes the displayed structures.
    fn drop(&mut self) {
        self.remove();
    }
}

impl Graphic3dStructureManager {
    /// A manager of the driver, with no view; `Graphic3dStructureManager::new(&driver)?`.
    pub fn new(driver: &Handle<dyn Graphic3dGraphicDriver>) -> Result<Self, Failure> {
        let fields = Graphic3dStructureManagerFields {
            view_gen_id: AspectGenId::new(0, 31)?,
            displayed_structure: Graphic3dStructureSet::default(),
            highlighted_structure: Graphic3dStructureSet::default(),
            registered_objects: HashMap::new(),
            graphic_driver: driver.clone(),
            defined_views: Vec::new(),
            device_lost_flag: false,
        };
        Ok(Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        })
    }
    /// The live defined views, in index order.
    pub fn views(&self) -> Vec<Handle<dyn Graphic3dCView>> {
        self.fields
            .read()
            .defined_views
            .iter()
            .map(WeakHandle::upgrade)
            .filter(|v| !v.is_null())
            .collect()
    }
    /// Runs `body` on every defined view.
    fn for_each_view(&self, mut body: impl FnMut(&dyn Graphic3dCView)) {
        for view in self.views() {
            if let Some(view) = view.get() {
                body(view);
            }
        }
    }
    /// The displayed structures, in index order.
    fn displayed(&self) -> Vec<Handle<dyn Graphic3dStructureSource>> {
        self.fields
            .read()
            .displayed_structure
            .iter()
            .cloned()
            .collect()
    }
    /// `Update`: updates the layer (or every layer for `UNKNOWN`) in every view;
    /// `manager.update(GRAPHIC3D_ZLAYER_ID_UNKNOWN)`.
    pub fn update(&self, layer_id: Graphic3dZLayerId) {
        self.for_each_view(|view| view.update(layer_id));
    }
    /// `Remove`: removes the displayed structures and forgets every view; `manager.remove()`.
    pub fn remove(&self) {
        for structure in self.displayed() {
            if let Some(structure) = structure.get() {
                structure.remove();
            }
        }
        let mut fields = self.fields.write();
        fields.displayed_structure.clear(true);
        fields.highlighted_structure.clear(true);
        fields.defined_views.clear();
    }
    /// `Erase()`: erases every displayed structure; `manager.erase()`.
    pub fn erase(&self) {
        for structure in self.displayed() {
            if let Some(structure) = structure.get() {
                structure.erase();
            }
        }
    }
    /// `DisplayedStructures`; `manager.displayed_structures()`.
    pub fn displayed_structures(&self) -> Vec<Handle<dyn Graphic3dStructureSource>> {
        self.displayed()
    }
    /// `HighlightedStructures`; `manager.highlighted_structures()`.
    pub fn highlighted_structures(&self) -> Vec<Handle<dyn Graphic3dStructureSource>> {
        self.fields
            .read()
            .highlighted_structure
            .iter()
            .cloned()
            .collect()
    }
    /// `NumberOfDisplayedStructures`; `manager.number_of_displayed_structures()`.
    pub fn number_of_displayed_structures(&self) -> i32 {
        self.fields.read().displayed_structure.extent() as i32
    }
    /// `Identification(int)`: the displayed structure of an identification, or null;
    /// `manager.structure_of(3)`.
    pub fn structure_of(&self, id: i32) -> Handle<dyn Graphic3dStructureSource> {
        use super::Graphic3dStructureMembers as _;
        let found = self
            .displayed()
            .into_iter()
            .find(|s| s.get().is_some_and(|s| s.identification() == id));
        found.unwrap_or_default()
    }
    /// `GraphicDriver`; `manager.graphic_driver()`.
    pub fn graphic_driver(&self) -> Handle<dyn Graphic3dGraphicDriver> {
        self.fields.read().graphic_driver.clone()
    }
    /// `RecomputeStructures()`: clears the device-lost flag and recomputes the network of
    /// every displayed structure; `manager.recompute_structures()`.
    pub fn recompute_structures(&self) {
        self.fields.write().device_lost_flag = false;
        let mut network = Vec::new();
        for structure in self.displayed() {
            graphic3d_structure_network(
                &structure,
                Graphic3dTypeOfConnection::Descendant,
                &mut network,
            );
        }
        self.recompute_structure_set(&network);
    }
    /// `RecomputeStructures(set)`: clears and computes each structure; `manager.recompute_structure_set(&set)`.
    pub fn recompute_structure_set(&self, structures: &[Handle<dyn Graphic3dStructureSource>]) {
        for structure in structures.iter().filter_map(Handle::get) {
            structure.clear(true);
            structure.compute();
        }
    }
    /// `RegisterObject`: binds the affinity of an object; `manager.register_object(address, &affinity)`.
    pub fn register_object(
        &self,
        object: &Handle<StandardTransient>,
        affinity: &Handle<Graphic3dViewAffinity>,
    ) {
        let key = object.object_address() as usize;
        let mut fields = self.fields.write();
        if fields.registered_objects.get(&key) == Some(affinity) {
            return;
        }
        fields.registered_objects.insert(key, affinity.clone());
    }
    /// `UnregisterObject`; `manager.unregister_object(&object)`.
    pub fn unregister_object(&self, object: &Handle<StandardTransient>) {
        self.fields
            .write()
            .registered_objects
            .remove(&(object.object_address() as usize));
    }
    /// `ObjectAffinity`: the affinity of an object, or null; `manager.object_affinity(&object)`.
    pub fn object_affinity(
        &self,
        object: &Handle<StandardTransient>,
    ) -> Handle<Graphic3dViewAffinity> {
        let fields = self.fields.read();
        fields
            .registered_objects
            .get(&(object.object_address() as usize))
            .cloned()
            .unwrap_or_default()
    }
    /// `IsDeviceLost`; `manager.is_device_lost()`.
    pub fn is_device_lost(&self) -> bool {
        self.fields.read().device_lost_flag
    }
    /// `SetDeviceLost`; `manager.set_device_lost()`.
    pub fn set_device_lost(&self) {
        self.fields.write().device_lost_flag = true;
    }
    /// `Identification(Graphic3d_CView*)`: the view identifier, a new one for a new view;
    /// `manager.identification(&view)?`.
    pub fn identification(&self, view: &Handle<dyn Graphic3dCView>) -> Result<i32, Failure> {
        let known = self
            .fields
            .read()
            .defined_views
            .iter()
            .any(|v| v.object_address() == view.object_address());
        if known {
            return Ok(view.get().map_or(0, |v| v.identification()));
        }
        let mut fields = self.fields.write();
        fields.defined_views.push(view.downgrade());
        fields.view_gen_id.next_id()
    }
    /// `UnIdentification`: forgets a view (at `address`) and frees its identifier;
    /// `manager.un_identification(view.object_address(), id)`.
    pub fn un_identification(&self, address: *const (), id: i32) {
        let mut fields = self.fields.write();
        let Some(index) = fields
            .defined_views
            .iter()
            .position(|v| v.object_address() == address)
        else {
            return;
        };
        fields.defined_views.swap_remove(index);
        fields.view_gen_id.free(id);
    }
    /// `DefinedViews`; `manager.defined_views().len()`.
    pub fn defined_views(&self) -> Vec<Handle<dyn Graphic3dCView>> {
        self.views()
    }
    /// `MaxNumOfViews`: the size of the identifier range; `manager.max_num_of_views()`.
    pub fn max_num_of_views(&self) -> i32 {
        let fields = self.fields.read();
        fields.view_gen_id.upper() - fields.view_gen_id.lower() + 1
    }
    /// `ReCompute(structure)`: recomputes a displayed structure in every view; `manager.re_compute(&s)`.
    pub fn re_compute(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        if !self.fields.read().displayed_structure.contains(structure) {
            return;
        }
        self.for_each_view(|view| view.re_compute(structure));
    }
    /// `ReCompute(structure, projector)`: recomputes a displayed structure in one defined view;
    /// `manager.re_compute_in(&s, &view)`.
    pub fn re_compute_in(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        projector: &Handle<dyn Graphic3dDataStructureManager>,
    ) {
        let view = projector.cast::<dyn Graphic3dCView>();
        let defined = self
            .fields
            .read()
            .defined_views
            .iter()
            .any(|v| v.object_address() == view.object_address());
        let displayed = self.fields.read().displayed_structure.contains(structure);
        if let (true, true, Some(view)) = (defined, displayed, view.get()) {
            view.re_compute(structure);
        }
    }
    /// `Clear`: clears the computed structures of `structure` in every view; `manager.clear(&s, true)`.
    pub fn clear(&self, structure: &Handle<dyn Graphic3dStructureSource>, with_destruction: bool) {
        self.for_each_view(|view| view.clear_computed(structure, with_destruction));
    }
    /// `Connect`: connects the computed structures in every view; `manager.connect(&mother, &daughter)`.
    pub fn connect(
        &self,
        mother: &Handle<dyn Graphic3dStructureSource>,
        daughter: &Handle<dyn Graphic3dStructureSource>,
    ) {
        self.for_each_view(|view| view.connect_computed(mother, daughter));
    }
    /// `Disconnect`: disconnects the computed structures in every view; `manager.disconnect(&mother, &daughter)`.
    pub fn disconnect(
        &self,
        mother: &Handle<dyn Graphic3dStructureSource>,
        daughter: &Handle<dyn Graphic3dStructureSource>,
    ) {
        self.for_each_view(|view| view.disconnect_computed(mother, daughter));
    }
    /// `Display`: records and displays a structure in every view; `manager.display(&s)`.
    pub fn display(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        self.fields
            .write()
            .displayed_structure
            .add(structure.clone());
        self.for_each_view(|view| view.display(structure));
    }
    /// `Erase(structure)`: forgets and erases a structure in every view; `manager.erase_structure(&s)`.
    pub fn erase_structure(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        {
            let mut fields = self.fields.write();
            fields.displayed_structure.remove_key(structure);
            fields.highlighted_structure.remove_key(structure);
        }
        self.for_each_view(|view| view.erase(structure));
    }
    /// `Highlight`: records and highlights the computed structures; `manager.highlight(&s)`.
    pub fn highlight(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        self.fields
            .write()
            .highlighted_structure
            .add(structure.clone());
        self.for_each_view(|view| view.highlight(structure));
    }
    /// `UnHighlight(structure)`: forgets and unhighlights the computed structures;
    /// `manager.unhighlight_structure(&s)`.
    pub fn unhighlight_structure(&self, structure: &Handle<dyn Graphic3dStructureSource>) {
        self.fields
            .write()
            .highlighted_structure
            .remove_key(structure);
        self.for_each_view(|view| view.unhighlight(structure));
    }
    /// `UnHighlight()`: unhighlights every highlighted structure; `manager.unhighlight()`.
    pub fn unhighlight(&self) {
        for structure in self.highlighted_structures() {
            if let Some(structure) = structure.get() {
                structure.unhighlight();
            }
        }
    }
    /// `SetTransform`: forwards a transformation change to every view; `manager.set_transform(&s, &datum)`.
    pub fn set_transform(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        trsf: &Handle<Datum3>,
    ) {
        self.for_each_view(|view| view.set_transform(structure, trsf));
    }
    /// `ChangeDisplayPriority`: moves the structure in every view; `manager.change_display_priority(&s, old, new)`.
    pub fn change_display_priority(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        old: Graphic3dDisplayPriority,
        new: Graphic3dDisplayPriority,
    ) {
        self.for_each_view(|view| view.change_priority(structure, old, new));
    }
    /// `DumpJson`: the displayed, highlighted and registered objects, the driver, the views
    /// and the device-lost flag; `manager.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        super::driver_dump::dump_structure_manager(self, stream, depth)
    }
    /// `ChangeZLayer`: moves a displayed structure to a layer in every view; `manager.change_z_layer(&s, 1)`.
    pub fn change_z_layer(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        layer_id: Graphic3dZLayerId,
    ) {
        if !self.fields.read().displayed_structure.contains(structure) {
            return;
        }
        self.for_each_view(|view| view.change_z_layer(structure, layer_id));
    }
}
