//! Named application subclass isolates source dispatch from unported parent selection/HLR.
#[path = "textured_diagnostic_printer.rs"]
mod diagnostics;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{message::Messenger, standard::*, tcollection::AsciiString};
use rustycad_tkservice::{graphic3d::*, image::ImagePixMap};
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub struct TexturedStateFixture {
    transient: StandardTransient,
    pub base: AisTexturedShapeBase,
    pub file_calls: AtomicUsize,
    pub pixmap_calls: AtomicUsize,
    pub diagnostics: Arc<diagnostics::TexturedDiagnosticPrinter>,
}
rustycad_tkernel::impl_transient!(TexturedStateFixture,"TexturedStateFixture",dyn AisTexturedShapeSource,transient,interfaces:[dyn AisTexturedShapeSource,dyn AisShapeSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl TexturedStateFixture {
    pub fn new() -> Handle<Self> {
        Self::with_factory(Arc::new(AisNativeTexturedTextureFactory))
    }
    pub fn with_factory(factory: Arc<dyn AisTexturedTextureFactory>) -> Handle<Self> {
        let diagnostics = Arc::new(diagnostics::TexturedDiagnosticPrinter::default());
        Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisTexturedShapeBase::with_services(
                &Shape::default(),
                Arc::new(Messenger::with_printer(diagnostics.clone())),
                factory,
            ),
            file_calls: AtomicUsize::new(0),
            pixmap_calls: AtomicUsize::new(0),
            diagnostics,
        })
    }
}
impl AisTexturedShapeSource for TexturedStateFixture {
    fn textured_shape_base(&self) -> &AisTexturedShapeBase {
        &self.base
    }
    fn set_texture_file_name(&self, file: &AsciiString) -> Result<(), Failure> {
        self.file_calls.fetch_add(1, Ordering::Relaxed);
        ais_textured_shape_set_texture_file_name(self, file)
    }
    fn set_texture_pix_map(&self, pixmap: &dyn HandleReference<ImagePixMap>) {
        self.pixmap_calls.fetch_add(1, Ordering::Relaxed);
        ais_textured_shape_set_texture_pix_map(self, pixmap);
    }
}
impl AisShapeSource for TexturedStateFixture {
    fn shape_base(&self) -> &AisShapeBase {
        &self.base.shape
    }
}
impl AisInteractiveObject for TexturedStateFixture {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.shape.interactive
    }
}
impl PrsMgrPresentableObject for TexturedStateFixture {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.shape.interactive.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        _: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        Err(Failure::new(&format!(
            "fixture Compute mode {mode}; expected source-state operations"
        )))
    }
}
impl SelectMgrSelectableObject for TexturedStateFixture {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.shape.interactive.selectable
    }
    fn compute_selection(&self, _: &Handle<SelectMgrSelection>, mode: i32) -> Result<(), Failure> {
        Err(Failure::new(&format!(
            "fixture selection mode {mode}; expected source-state operations"
        )))
    }
}
