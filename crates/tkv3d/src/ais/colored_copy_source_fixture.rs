//! A real application subclass replaces the original constructor argument during virtual getters.
use super::*;
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkservice::graphic3d::Graphic3dNameOfMaterial;
use std::{fmt::Write, sync::Arc};
pub(super) struct CopySourceInput {
    pub slot: SharedState<Handle<dyn AisShapeSource>>,
    pub incoming: WeakHandle<dyn AisShapeSource>,
    pub rows: SharedState<String>,
    pub stage: i32,
    pub failure: i32,
}
pub(super) struct CopySourceProbe {
    transient: StandardTransient,
    base: AisColoredShapeBase,
    input: Arc<CopySourceInput>,
}
rustycad_tkernel::impl_transient!(CopySourceProbe,"ColoredCopySourceProbe",dyn AisShapeSource,transient,
    interfaces:[dyn AisColoredShapeSource,dyn AisShapeSource,dyn AisInteractiveObject,
    dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl CopySourceProbe {
    /// Own genuine inherited state for a live-slot probe; `CopySourceProbe::new(&shape, &input)`.
    pub fn new(shape: &Shape, input: &Arc<CopySourceInput>) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisColoredShapeBase::new(shape),
            input: input.clone(),
        });
        prs_mgr_presentable_bind(&object.clone().cast());
        object
    }
    fn replace_source(&self, stage: i32) {
        if self.input.stage != stage {
            return;
        }
        self.record_source_counts("before", stage);
        *self.input.slot.write() = self.input.incoming.observe().clone();
        self.record_source_counts("after", stage);
        source_getter_failure(self.input.failure);
    }
    fn record_source_counts(&self, phase: &str, stage: i32) {
        let own = self.transient.ref_count();
        let incoming = self.input.incoming.observe().ref_count();
        let topology = self.base.shape.fields.read().shape.tshape().ref_count();
        writeln!(
            &mut *self.input.rows.write(),
            "getter-{phase} {stage} {own} {incoming} {topology}"
        )
        .unwrap();
    }
}
fn source_getter_failure(failure: i32) {
    if failure == 2 {
        std::panic::panic_any("unrelated copy-source panic");
    }
    if failure == 1 {
        rustycad_tkernel::standard::raise_native_failure::<()>(Err(
            rustycad_tkernel::standard::Failure::new(
                "source failure=1; expected successful getter",
            ),
        ));
    }
}
impl AisColoredShapeSource for CopySourceProbe {
    fn colored_shape_base(&self) -> &AisColoredShapeBase {
        // Real derived map precedes the actual inherited parent during destruction.
        &self.base
    }
}
impl AisShapeSource for CopySourceProbe {
    fn shape_base(&self) -> &AisShapeBase {
        // All runtime views retain the same parent, not a copied appearance object.
        &self.base.shape
    }
}
impl AisInteractiveObject for CopySourceProbe {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        // Native interactive state is the genuine inherited owned field.
        &self.base.shape.interactive
    }
}
impl SelectMgrSelectableObject for CopySourceProbe {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        // This probe does not invent selection geometry or fake parent ownership.
        &self.base.shape.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn rustycad_tkernel::standard::HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        // The application subclass inherits genuine ColoredShape selection dispatch.
        ais_colored_shape_compute_selection(self, selection, mode)
    }
}
impl PrsMgrPresentableObject for CopySourceProbe {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        // Getter callbacks borrow the actual source object without retaining it.
        &self.base.shape.interactive.presentable
    }
    fn compute(
        &self,
        _: &dyn rustycad_tkernel::standard::HandleReference<PrsMgrPresentationManager>,
        prs: &dyn rustycad_tkernel::standard::HandleReference<
            dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource,
        >,
        mode: i32,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        // Presentation overrides remain genuine inherited dispatch, unrelated to the getter fault.
        ais_colored_shape_compute(self, prs, mode)
    }
    fn material(&self) -> Graphic3dNameOfMaterial {
        let value = ais_shape_material(self);
        self.replace_source(3);
        value
    }
    fn color(&self) -> Color {
        let value = super::shape_appearance_queries::shape_color_value(self);
        self.replace_source(4);
        value
    }
    fn transparency(&self) -> f64 {
        let value = ais_shape_transparency(self);
        self.replace_source(5);
        value
    }
}
pub(super) fn copy_source_slot(slot: &Handle<dyn AisShapeSource>) -> &Handle<dyn AisShapeSource> {
    // Return the actual caller slot, so callbacks replace what the constructor reads.
    slot
}
