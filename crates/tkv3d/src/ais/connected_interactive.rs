//! `AIS_ConnectedInteractive` (an instance of another object at a location, sharing its
//! presentations and copying its selection) and `AIS_MultipleConnectedInteractive` (an
//! assembly of connected instances under one owner).
use super::*;
use crate::prs_mgr::*;
use crate::select_mgr::*;
use crate::select3d::{Select3DSensitiveEntity, Select3DSensitiveEntityMembers};
use crate::std_select::{StdSelectBRepOwner, StdSelectBRepOwnerMembers, StdSelectBRepOwnerSource};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleReference, SharedState, StandardTransient,
};
use rustycad_tkmath::gp::{Transform3, TransformForm};
use rustycad_tkmath::toploc::{Datum3, Location};
use rustycad_tkservice::graphic3d::*;

/// The protected fields of `AIS_ConnectedInteractive`.
#[derive(Clone, Default)]
pub struct AisConnectedInteractiveFields {
    /// `myReference`: the connected object.
    pub reference: Handle<dyn AisInteractiveObject>,
    /// `myShape`: the located shape of an AIS_Shape reference, for HLR.
    pub shape: Shape,
}

/// `AIS_ConnectedInteractive`: an instance of another object (whose presentations are
/// connected, not recomputed) at its own location.
/// Example: `let instance = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView); instance.connect(&object)?;`.
pub struct AisConnectedInteractive {
    transient: StandardTransient,
    base: AisInteractiveObjectBase,
    /// The protected fields.
    pub fields: SharedState<AisConnectedInteractiveFields>,
}
rustycad_tkernel::impl_transient!(AisConnectedInteractive, "AIS_ConnectedInteractive", dyn AisInteractiveObject, transient,
    interfaces: [dyn AisInteractiveObject, dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);

impl AisConnectedInteractive {
    /// An unconnected instance; `AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView)`.
    pub fn new(kind: PrsMgrTypeOfPresentation3d) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(kind),
            fields: SharedState::default(),
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
    /// `Connect(object)`: at no own location; `instance.connect(&object)?`.
    pub fn connect(&self, other: &Handle<dyn AisInteractiveObject>) -> Result<(), Failure> {
        self.connect_datum(other, &Handle::default())
    }
    /// `Connect(object, trsf)`; `instance.connect_trsf(&object, &trsf)?`.
    pub fn connect_trsf(
        &self,
        other: &Handle<dyn AisInteractiveObject>,
        location: &Transform3,
    ) -> Result<(), Failure> {
        self.connect_datum(other, &Handle::new(Datum3::new(*location)))
    }
    /// `Connect(object, datum)` / `connect` (protected): the reference of a connected object is
    /// shared, an object without own presentations or displayed in a context is refused with
    /// `Standard_ProgramError`; the presentation type follows the reference.
    /// Example: `instance.connect_datum(&object, &datum)?`.
    pub fn connect_datum(
        &self,
        other: &Handle<dyn AisInteractiveObject>,
        location: &Handle<Datum3>,
    ) -> Result<(), Failure> {
        if self.fields.read().reference.object_address() == other.object_address() {
            return self.set_local_transformation_datum(location);
        }
        let connected = other.cast::<AisConnectedInteractive>();
        let reference = if let Some(connected) = connected.get() {
            connected.fields.read().reference.clone()
        } else if other.get().is_some_and(|o| o.has_own_presentations()) {
            other.clone()
        } else {
            return Err(program_error(
                "AIS_ConnectedInteractive::Connect() - object without own presentation can not be connected",
            ));
        };
        self.fields.write().reference = reference.clone();
        if let Some(value) = reference.get() {
            if value.has_interactive_context()
                && value.get_context().display_status(&reference) != PrsMgrDisplayStatus::None
            {
                self.fields.write().reference = Handle::default();
                return Err(program_error(
                    "AIS_ConnectedInteractive::Connect() - connected object should NOT be displayed in context",
                ));
            }
            let kind = value.type_of_presentation3d();
            self.presentable_base()
                .fields
                .write()
                .type_of_presentation3d = kind;
        }
        self.set_local_transformation_datum(location)
    }
    /// `HasConnection`; `instance.has_connection()`.
    pub fn has_connection(&self) -> bool {
        !self.fields.read().reference.is_null()
    }
    /// `ConnectedTo`; `instance.connected_to()`.
    pub fn connected_to(&self) -> Handle<dyn AisInteractiveObject> {
        self.fields.read().reference.clone()
    }
    /// `Disconnect`: every presentation loses its descendants; `instance.disconnect()`.
    pub fn disconnect(&self) {
        for prs in self.presentations().iter().filter_map(Handle::get) {
            prs.disconnect_all(Graphic3dTypeOfConnection::Descendant);
        }
    }
    /// `updateShape` (protected): the shape of an AIS_Shape reference, moved by the instance
    /// transformation when asked. Example: `instance.update_shape(true)?`.
    pub fn update_shape(&self, with_location: bool) -> Result<(), Failure> {
        let reference = self.fields.read().reference.cast::<dyn AisShapeSource>();
        let Some(reference) = reference.get() else {
            return Ok(());
        };
        let shape = reference.shape();
        if shape.is_null() {
            return Ok(());
        }
        let shape = if with_location {
            shape.moved(&Location::new(self.transformation()), true)?
        } else {
            shape
        };
        self.fields.write().shape = shape;
        Ok(())
    }
    /// `computeSubShapeSelection` (protected): one decomposed BRep owner per sub-shape of the
    /// reference selection (in the native map order), with the copies of its entities.
    /// Example: `instance.compute_sub_shape_selection(&selection, 2)?`.
    pub fn compute_sub_shape_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        let reference = self.fields.read().reference.clone();
        let value = reference.get().ok_or_else(no_reference)?;
        if !value.has_selection(mode) {
            value.recompute_primitives_of_mode(mode)?;
        }
        let reference_selection = value.selection(mode);
        let reference_selection = reference_selection.get().ok_or_else(no_reference)?;
        if reference_selection.is_empty()
            || reference_selection.update_status() == SelectMgrTypeOfUpdate::Full
        {
            value.recompute_primitives_of_mode(mode)?;
        }
        let mut shapes: DataMap<Shape, Vec<Handle<dyn Select3DSensitiveEntity>>> =
            DataMap::default();
        for entity in reference_selection.entities() {
            let sensitive = entity.base_sensitive();
            let Some(value) = sensitive.get() else {
                continue;
            };
            let owner = value.owner_id().cast::<dyn StdSelectBRepOwnerSource>();
            let Some(owner) = owner.get() else {
                continue;
            };
            let sub_shape = owner.shape_ref().clone_shape();
            match shapes.change_find(&sub_shape) {
                Ok(list) => list.push(sensitive.clone()),
                Err(_) => {
                    shapes.bind(sub_shape, vec![sensitive.clone()]);
                }
            }
        }
        let this: Handle<dyn SelectMgrSelectableObject> = self.presentable_base().this().cast();
        let location = Location::new(self.transformation());
        for (sub_shape, list) in shapes.items() {
            let priority = list[0].owner_id().priority();
            let owner = StdSelectBRepOwner::with_selectable(sub_shape, &this, priority, true);
            owner.set_location(&location);
            let owner: Handle<dyn SelectMgrEntityOwnerSource> = owner.into_cast();
            for sensitive in list {
                let copy = sensitive.get_connected()?;
                if let Some(copy_value) = copy.get() {
                    copy_value.set(&owner);
                    selection.add(&copy)?;
                }
            }
        }
        Ok(())
    }
}

fn no_reference() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "AIS_ConnectedInteractive: no connected object",
    )
}

impl AisInteractiveObject for AisConnectedInteractive {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    /// `Type`: `AIS_KindOfInteractive_Object`.
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Object
    }
    /// `Signature`: 0.
    fn signature(&self) -> i32 {
        0
    }
}

impl PrsMgrPresentableObject for AisConnectedInteractive {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    /// `AcceptDisplayMode`: every mode without reference, else the reference's.
    fn accept_display_mode(&self, mode: i32) -> bool {
        let reference = self.fields.read().reference.clone();
        reference.get().is_none_or(|r| r.accept_display_mode(mode))
    }
    /// `Compute`: the presentation is cleared and connected to the reference presentation
    /// (updated when stale), the reference taking the instance context.
    fn compute(
        &self,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        let prs = prs.observe();
        if self.has_connection() {
            prs.clear(false);
            prs.disconnect_all(Graphic3dTypeOfConnection::Descendant);
            let reference = self.fields.read().reference.clone();
            if !reference.has_interactive_context() {
                reference.set_context(&self.get_context());
            }
            let manager = manager.observe();
            let this: Handle<dyn PrsMgrPresentableObject> = self.presentable_base().this();
            manager.connect(&this, &reference.cast(), mode, mode)?;
            let presentation =
                manager.presentation(&reference.cast(), mode, true, &Handle::default())?;
            if presentation.must_be_updated() {
                manager.update(&reference.cast(), mode)?;
            }
        }
        if !prs.is_null() {
            prs.re_compute();
        }
        Ok(())
    }
    /// `computeHLR`: the HLR presentation of the shape of an AIS_Shape reference, at the
    /// instance location or moved by a non-identity transformation.
    fn compute_hlr(
        &self,
        projector: &Handle<Graphic3dCamera>,
        trsf: &Handle<Datum3>,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let moved = trsf
            .get()
            .filter(|datum| datum.trsf().form() != TransformForm::Identity);
        self.update_shape(moved.is_none())?;
        let shape = self.fields.read().shape.clone();
        if shape.is_null() {
            return Ok(());
        }
        let drawer = self.attributes();
        match moved {
            Some(datum) => {
                let location = Location::new(*datum.trsf()).multiplied(shape.location())?;
                let located = shape.located(&location, true)?;
                ais_shape_compute_hlr_presentation(projector, prs, &located, &drawer)
            }
            None => ais_shape_compute_hlr_presentation(projector, prs, &shape, &drawer),
        }
    }
    crate::ais_presentable_overrides!();
}

impl SelectMgrSelectableObject for AisConnectedInteractive {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    /// `AcceptShapeDecomposition`: the reference's.
    fn accept_shape_decomposition(&self) -> bool {
        let reference = self.fields.read().reference.clone();
        reference
            .get()
            .is_some_and(|r| r.accept_shape_decomposition())
    }
    /// `ComputeSelection`: the sub-shape selection for a decomposable reference in a mode
    /// other than 0, else the copies of the reference entities under one owner at the
    /// instance location.
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        if !self.has_connection() {
            return Ok(());
        }
        let selection = selection.observe();
        let reference = self.fields.read().reference.clone();
        let value = reference.get().ok_or_else(no_reference)?;
        if mode != 0 && value.accept_shape_decomposition() {
            return self.compute_sub_shape_selection(&selection, mode);
        }
        if !value.has_selection(mode) {
            value.recompute_primitives_of_mode(mode)?;
        }
        let reference_selection = value.selection(mode);
        let this: Handle<dyn SelectMgrSelectableObject> = self.presentable_base().this().cast();
        let owner = SelectMgrEntityOwner::with_selectable(&this, 0);
        owner.set_location(&Location::new(self.transformation()));
        let owner: Handle<dyn SelectMgrEntityOwnerSource> = owner.into_cast();
        if reference_selection.get().is_some_and(|s| s.is_empty()) {
            value.recompute_primitives_of_mode(mode)?;
        }
        let Some(reference_selection) = reference_selection.get() else {
            return Ok(());
        };
        for entity in reference_selection.entities() {
            let sensitive = entity.base_sensitive();
            let Some(sensitive) = sensitive.get() else {
                continue;
            };
            let copy = sensitive.get_connected()?;
            if let Some(copy_value) = copy.get() {
                copy_value.set(&owner);
                selection.add(&copy)?;
            }
        }
        Ok(())
    }
}

/// The protected field of `AIS_MultipleConnectedInteractive`.
#[derive(Clone, Default)]
pub struct AisMultipleConnectedInteractiveFields {
    /// `myAssemblyOwner`: the owner of the whole assembly.
    pub assembly_owner: Handle<dyn SelectMgrEntityOwnerSource>,
}

/// `AIS_MultipleConnectedInteractive`: an assembly of connected instances (its children),
/// selected as a whole through one assembly owner.
/// Example: `let assembly = AisMultipleConnectedInteractive::new(); assembly.connect(&object)?;`.
pub struct AisMultipleConnectedInteractive {
    transient: StandardTransient,
    base: AisInteractiveObjectBase,
    /// The protected fields.
    pub fields: SharedState<AisMultipleConnectedInteractiveFields>,
}
rustycad_tkernel::impl_transient!(AisMultipleConnectedInteractive, "AIS_MultipleConnectedInteractive", dyn AisInteractiveObject, transient,
    interfaces: [dyn AisInteractiveObject, dyn SelectMgrSelectableObject, dyn PrsMgrPresentableObject]);

impl AisMultipleConnectedInteractive {
    /// An empty assembly without own presentations; `AisMultipleConnectedInteractive::new()`.
    pub fn new() -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            fields: SharedState::default(),
        });
        object
            .presentable_base()
            .fields
            .write()
            .has_own_presentations = false;
        prs_mgr_presentable_bind(&object.cast());
        object
    }
    /// `Connect(object)`: at the object local transformation and persistence.
    /// Example: `assembly.connect(&object)?`.
    pub fn connect(
        &self,
        other: &Handle<dyn AisInteractiveObject>,
    ) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        let value = other.get().ok_or_else(no_reference)?;
        self.connect_full(
            other,
            &value.local_transformation_geom(),
            &value.transform_persistence(),
        )
    }
    /// `Connect(object, trsf)`: at a location, keeping the object persistence.
    /// Example: `assembly.connect_trsf(&object, &trsf)?`.
    pub fn connect_trsf(
        &self,
        other: &Handle<dyn AisInteractiveObject>,
        location: &Transform3,
    ) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        let value = other.get().ok_or_else(no_reference)?;
        self.connect_full(
            other,
            &Handle::new(Datum3::new(*location)),
            &value.transform_persistence(),
        )
    }
    /// `Connect(object, trsf, persistence)`; `assembly.connect_trsf_pers(&object, &trsf, &pers)?`.
    pub fn connect_trsf_pers(
        &self,
        other: &Handle<dyn AisInteractiveObject>,
        location: &Transform3,
        persistence: &Handle<dyn Graphic3dTransformPersSource>,
    ) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        self.connect_full(other, &Handle::new(Datum3::new(*location)), persistence)
    }
    /// `Connect(object, datum, persistence)` / `connect` (protected): an assembly is copied as
    /// a new assembly sharing the owner, another object as a connected instance; the new child
    /// takes the location and the persistence. Example: `assembly.connect_full(&object, &datum, &pers)?`.
    pub fn connect_full(
        &self,
        other: &Handle<dyn AisInteractiveObject>,
        location: &Handle<Datum3>,
        persistence: &Handle<dyn Graphic3dTransformPersSource>,
    ) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        if self.fields.read().assembly_owner.is_null() {
            let this: Handle<dyn SelectMgrSelectableObject> = self.presentable_base().this().cast();
            self.fields.write().assembly_owner =
                SelectMgrEntityOwner::with_selectable(&this, 0).into_cast();
        }
        let multi = other.cast::<AisMultipleConnectedInteractive>();
        let added: Handle<dyn AisInteractiveObject> = if let Some(multi) = multi.get() {
            let copy = AisMultipleConnectedInteractive::new();
            copy.fields.write().assembly_owner = self.fields.read().assembly_owner.clone();
            copy.set_local_transformation_datum(&multi.local_transformation_geom())?;
            for child in multi.children() {
                let child = child.cast::<dyn AisInteractiveObject>();
                if !child.is_null() {
                    copy.connect(&child)?;
                }
            }
            copy.into_cast()
        } else {
            let value = other.get().ok_or_else(no_reference)?;
            let instance = AisConnectedInteractive::new(PrsMgrTypeOfPresentation3d::AllView);
            instance.connect_datum(other, &value.local_transformation_geom())?;
            instance.into_cast()
        };
        let value = added.get().ok_or_else(no_reference)?;
        value.set_local_transformation_datum(location)?;
        if !persistence.is_null() {
            value.set_transform_persistence(persistence);
        }
        self.add_child(&added.cast())?;
        Ok(added)
    }
    /// `HasConnection`: whether there are children; `assembly.has_connection()`.
    pub fn has_connection(&self) -> bool {
        !self.children().is_empty()
    }
    /// `Disconnect(object)`: removes a child; `assembly.disconnect(&child)?`.
    pub fn disconnect(&self, other: &Handle<dyn AisInteractiveObject>) -> Result<(), Failure> {
        self.remove_child(&other.cast())
    }
    /// `DisconnectAll`: removes every child, first first; `assembly.disconnect_all()?`.
    pub fn disconnect_all(&self) -> Result<(), Failure> {
        let count = self.children().len();
        for _ in 0..count {
            if let Some(first) = self.children().first().cloned() {
                self.remove_child(&first)?;
            }
        }
        Ok(())
    }
}

impl AisInteractiveObject for AisMultipleConnectedInteractive {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    /// `Type`: `AIS_KindOfInteractive_Object`.
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::Object
    }
    /// `Signature`: 1.
    fn signature(&self) -> i32 {
        1
    }
    /// `SetContext`: the assembly and every interactive child.
    fn set_context(&self, context: &Handle<AisInteractiveContext>) {
        ais_interactive_object_set_context(self, context);
        for child in self.children() {
            let child = child.cast::<dyn AisInteractiveObject>();
            if let Some(child) = child.get() {
                child.set_context(context);
            }
        }
    }
}

impl PrsMgrPresentableObject for AisMultipleConnectedInteractive {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    /// `Compute`: the children take the assembly context (the children present themselves).
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        _: &dyn HandleReference<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let context = self.get_context();
        for child in self.children() {
            let child = child.cast::<dyn AisInteractiveObject>();
            if let Some(child) = child.get() {
                child.set_context(&context);
            }
        }
        Ok(())
    }
    crate::ais_presentable_overrides!();
}

impl SelectMgrSelectableObject for AisMultipleConnectedInteractive {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    /// `AcceptShapeDecomposition`: one interactive child accepts it.
    fn accept_shape_decomposition(&self) -> bool {
        self.children().iter().any(|child| {
            child
                .cast::<dyn AisInteractiveObject>()
                .get()
                .is_some_and(|child| child.accept_shape_decomposition())
        })
    }
    /// `GetAssemblyOwner`: the assembly owner.
    fn get_assembly_owner(&self) -> Handle<dyn SelectMgrEntityOwnerSource> {
        self.fields.read().assembly_owner.clone()
    }
    /// `GlobalSelOwner`: the assembly owner.
    fn global_sel_owner(&self) -> Handle<dyn SelectMgrEntityOwnerSource> {
        self.fields.read().assembly_owner.clone()
    }
    /// `ComputeSelection`: nothing in mode 0, else every child computes its own selection
    /// (into a throwaway selection, as natively).
    fn compute_selection(
        &self,
        _: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode == 0 {
            return Ok(());
        }
        for child in self.children() {
            let child = child.cast::<dyn AisInteractiveObject>();
            let Some(value) = child.get() else {
                continue;
            };
            if !value.has_selection(mode) {
                value.recompute_primitives_of_mode(mode)?;
            }
            let selection = Handle::new(SelectMgrSelection::new(mode));
            value.compute_selection(&selection, mode)?;
        }
        Ok(())
    }
}

impl AisInteractiveContext {
    /// `Disconnect(assembly, object)`: removes an object from an assembly (erasing it when it
    /// is not registered in the context) or disconnects a connected instance (with a null
    /// object), and forgets the selection structures of the removed object; a null assembly
    /// is a `Standard_NullObject` failure (a null dereference natively).
    /// Example: `context.disconnect(&assembly, &child)?`.
    pub fn disconnect(
        &self,
        assembly: &Handle<dyn AisInteractiveObject>,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        if assembly.is_null() {
            return Err(no_reference());
        }
        let multi = assembly.cast::<AisMultipleConnectedInteractive>();
        let connected = assembly.cast::<AisConnectedInteractive>();
        if let Some(multi) = multi.get() {
            multi.disconnect(object)?;
            if !self.fields.read().objects.is_bound(object) {
                self.main_prs_mgr().erase(&object.cast(), -1);
                if let Some(child) = object.get() {
                    child.erase_presentations(true);
                }
            }
            self.selection_manager().remove(&object.cast())?;
        } else if let Some(connected) = connected.get()
            && object.is_null()
        {
            connected.disconnect();
            self.selection_manager().remove(&assembly.cast())?;
        }
        Ok(())
    }
}
