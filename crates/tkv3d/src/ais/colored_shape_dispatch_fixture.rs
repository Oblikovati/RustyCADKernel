//! Named application subclass owns native parent state and overrides real virtual lifecycle calls.
use super::*;
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use std::{
    fmt::Write,
    sync::{
        Mutex,
        atomic::{AtomicI32, Ordering},
    },
};
pub(super) struct ColoredDispatchFake {
    transient: StandardTransient,
    base: AisColoredShapeBase,
    pub prs: SharedState<Handle<PrsMgrPresentation>>,
    pub rows: Mutex<String>,
    pub failure: AtomicI32,
}
rustycad_tkernel::impl_transient!(ColoredDispatchFake,"ColoredDispatchProbe",dyn AisColoredShapeSource,transient,
    interfaces:[dyn AisColoredShapeSource,dyn AisShapeSource,dyn AisInteractiveObject,
    dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl ColoredDispatchFake {
    pub fn new(shape: &Shape) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisColoredShapeBase::new(shape),
            prs: SharedState::default(),
            rows: Mutex::new(String::new()),
            failure: AtomicI32::new(0),
        });
        prs_mgr_presentable_bind(&object.clone().cast());
        object
    }
    pub fn event(&self, label: &str) {
        let extent = self.custom_aspects_map().len();
        let dirty = i32::from(self.prs.read().must_be_updated());
        writeln!(self.rows.lock().unwrap(), "{label} {extent} {dirty}").unwrap();
    }
    fn fail_stage(&self, stage: i32) -> Result<(), Failure> {
        if self.failure.load(Ordering::Relaxed) == stage {
            return Err(Failure::new(&format!(
                "callback stage={stage}; expected successful callback"
            )));
        }
        Ok(())
    }
}
impl AisColoredShapeSource for ColoredDispatchFake {
    fn colored_shape_base(&self) -> &AisColoredShapeBase {
        // Source overrides retain this actual single native subclass map.
        &self.base
    }
    fn custom_aspects(&self, shape: &Shape) -> Result<Handle<AisColoredDrawer>, Failure> {
        self.event("custom-before");
        self.fail_stage(1)?;
        let result = ais_colored_shape_custom_aspects(self, shape)?;
        self.event("custom-after");
        self.fail_stage(2)?;
        Ok(result)
    }
    fn clear_custom_aspects(&self) -> Result<(), Failure> {
        self.event("clear-before");
        self.fail_stage(1)?;
        ais_colored_shape_clear_custom_aspects(self);
        self.event("clear-after");
        self.fail_stage(2)?;
        Ok(())
    }
}
impl AisShapeSource for ColoredDispatchFake {
    fn shape_base(&self) -> &AisShapeBase {
        // Parent topology is shared with the concrete native subclass owner.
        &self.base.shape
    }
}
impl AisInteractiveObject for ColoredDispatchFake {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        // Inherited runtime views expose the same owned context and parent fields.
        &self.base.shape.interactive
    }
}
impl PrsMgrPresentableObject for ColoredDispatchFake {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        // Every callback observes actual inherited presentation state.
        &self.base.shape.interactive.presentable
    }
    fn color(&self) -> rustycad_tkernel::quantity::Color {
        copy_getter_failure(self, 4);
        super::shape_appearance_queries::shape_color_value(self)
    }
    fn material(&self) -> rustycad_tkservice::graphic3d::Graphic3dNameOfMaterial {
        copy_getter_failure(self, 3);
        super::ais_shape_material(self)
    }
    fn transparency(&self) -> f64 {
        copy_getter_failure(self, 5);
        super::ais_shape_transparency(self)
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        _: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        Err(Failure::new(&format!(
            "probe Compute mode={mode}; expected lifecycle-only call"
        )))
    }
}
impl SelectMgrSelectableObject for ColoredDispatchFake {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        // Genuine parent fields remain owned even though this probe never selects geometry.
        &self.base.shape.interactive.selectable
    }
    fn compute_selection(&self, _: &Handle<SelectMgrSelection>, mode: i32) -> Result<(), Failure> {
        Err(Failure::new(&format!(
            "probe selection mode={mode}; expected lifecycle-only call"
        )))
    }
}

fn copy_getter_failure(object: &ColoredDispatchFake, stage: i32) {
    if object.failure.load(Ordering::Relaxed) == 6 {
        std::panic::panic_any("unrelated copy-source panic");
    }
    if object.failure.load(Ordering::Relaxed) == stage {
        rustycad_tkernel::standard::raise_native_failure::<()>(Err(Failure::new(&format!(
            "copy getter stage={stage}; expected successful getter"
        ))));
    }
}
