//! Named virtual group fake appends exactly one genuine retained group during aspect assignment.
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::{graphic3d::*, null_driver::NullGroup};
use std::sync::atomic::{AtomicBool, Ordering};
pub struct TexturedAppendingGroup {
    transient: StandardTransient,
    inner: NullGroup,
    append_once: AtomicBool,
}
rustycad_tkernel::impl_transient!(TexturedAppendingGroup,"TexturedAppendingGroup",StandardTransient,transient,interfaces:[dyn Graphic3dGroup]);
impl TexturedAppendingGroup {
    pub fn new(prs: &Handle<dyn Graphic3dStructureSource>) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            inner: NullGroup::new(prs),
            append_once: AtomicBool::new(true),
        })
    }
}
impl Graphic3dGroup for TexturedAppendingGroup {
    fn group_base(&self) -> &Graphic3dGroupBase {
        self.inner.group_base()
    }
    fn aspects(&self) -> Handle<dyn Graphic3dAspectsSource> {
        self.inner.aspects()
    }
    fn set_group_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.inner.set_group_primitives_aspect(aspect);
        if self.append_once.swap(false, Ordering::Relaxed) {
            self.structure().new_group().unwrap();
        }
    }
    fn set_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.inner.set_primitives_aspect(aspect);
    }
    fn synchronize_aspects(&self) {
        self.inner.synchronize_aspects();
    }
    fn replace_aspects(&self, replacements: &Graphic3dAspectsReplaceMap) {
        self.inner.replace_aspects(replacements);
    }
    fn set_stencil_test_options(&self, enabled: bool) {
        self.inner.set_stencil_test_options(enabled);
    }
    fn set_flipping_options(&self, enabled: bool, plane: &Axis2) {
        self.inner.set_flipping_options(enabled, plane);
    }
}
