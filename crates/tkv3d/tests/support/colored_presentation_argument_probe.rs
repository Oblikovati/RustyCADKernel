//! Named application subclass owns its real ColoredShape parent and forwards original Compute references.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleFieldRef, HandleReference, SharedState, StandardTransient, WeakHandle,
};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*};
use std::{fmt::Write, sync::Arc};
pub struct PresentationArgumentInput {
    pub manager: Arc<SharedState<Handle<PrsMgrPresentationManager>>>,
    pub presentation: Arc<SharedState<Handle<dyn Graphic3dStructureSource>>>,
    pub incoming_manager: WeakHandle<PrsMgrPresentationManager>,
    pub incoming_presentation: WeakHandle<dyn Graphic3dStructureSource>,
    pub rows: Arc<SharedState<String>>,
}
pub struct PresentationArgumentProbe {
    transient: StandardTransient,
    input: Arc<PresentationArgumentInput>,
    base: AisColoredShapeBase,
}
rustycad_tkernel::impl_transient!(PresentationArgumentProbe,"PresentationArgumentProbe",dyn AisColoredShapeSource,transient,
    interfaces:[dyn AisColoredShapeSource,dyn AisShapeSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl PresentationArgumentProbe {
    /// Own the actual native parent once; `PresentationArgumentProbe::new(&shape,&input)`.
    pub fn new(shape: &Shape, input: &Arc<PresentationArgumentInput>) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: Default::default(),
            input: input.clone(),
            base: AisColoredShapeBase::new(shape),
        });
        prs_mgr_presentable_bind(&object.clone().cast());
        object
    }
    fn argument_row(
        &self,
        phase: &str,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
    ) -> String {
        let manager_field = HandleFieldRef::new(&self.input.manager, manager_argument_field);
        let prs_field = HandleFieldRef::new(&self.input.presentation, presentation_argument_field);
        let manager_same =
            i32::from(manager.reference_address() == manager_field.reference_address());
        let prs_same = i32::from(prs.reference_address() == prs_field.reference_address());
        let managers = manager.observe().ref_count();
        let presentations = prs.observe().ref_count();
        format!("compute-slot-{phase} {manager_same} {prs_same} {managers} {presentations}")
    }
    fn record_after(
        &self,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
    ) {
        let row = self.argument_row("after", manager, prs);
        let manager_same =
            i32::from(manager.object_address() == self.input.incoming_manager.object_address());
        let prs_same =
            i32::from(prs.object_address() == self.input.incoming_presentation.object_address());
        writeln!(
            &mut *self.input.rows.write(),
            "{row} {manager_same} {prs_same}"
        )
        .unwrap();
    }
}
impl PrsMgrPresentableObject for PresentationArgumentProbe {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        // The virtual override uses the genuine inherited presentation fields.
        &self.base.shape.interactive.presentable
    }
    fn compute(
        &self,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        let row = self.argument_row("before", manager, prs);
        writeln!(&mut *self.input.rows.write(), "{row}").unwrap();
        *self.input.manager.write() = self.input.incoming_manager.observe().clone();
        ais_colored_shape_compute(self, prs, mode)?;
        self.record_after(manager, prs);
        Ok(())
    }
}
impl SelectMgrSelectableObject for PresentationArgumentProbe {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        // Selection views share the same actual native parent allocation.
        &self.base.shape.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn rustycad_tkernel::standard::HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_colored_shape_compute_selection(self, selection, mode)
    }
}
impl AisInteractiveObject for PresentationArgumentProbe {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        // No separate interactive facade duplicates the native inherited ownership.
        &self.base.shape.interactive
    }
}
impl AisShapeSource for PresentationArgumentProbe {
    fn shape_base(&self) -> &AisShapeBase {
        // Topology and cached bounds belong to the actual superclass fields.
        &self.base.shape
    }
}
impl AisColoredShapeSource for PresentationArgumentProbe {
    fn colored_shape_base(&self) -> &AisColoredShapeBase {
        // Custom aspects remain in the real owned derived map during callbacks.
        &self.base
    }
}
/// Lend the exact manager field; `HandleFieldRef::new(&slot, manager_argument_field)`.
pub fn manager_argument_field(
    value: &Handle<PrsMgrPresentationManager>,
) -> &Handle<PrsMgrPresentationManager> {
    // Const-reference identity belongs to this exact caller field.
    value
}
/// Lend the exact presentation field; `HandleFieldRef::new(&slot, presentation_argument_field)`.
pub fn presentation_argument_field(
    value: &Handle<dyn Graphic3dStructureSource>,
) -> &Handle<dyn Graphic3dStructureSource> {
    // This is the field that the actual graphics callback replaces.
    value
}
