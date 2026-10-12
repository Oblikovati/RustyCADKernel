//! Named virtual group fake appends exactly one genuine retained group during aspect assignment.
use rustycad_tkernel::standard::{Handle, StandardTransient, WeakHandle};
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::{graphic3d::*, null_driver::NullGroup};
use rustycad_tkv3d::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentation};
use std::sync::atomic::{AtomicBool, Ordering};
enum TexturedAppendTarget {
    Group,
    Presentation {
        owner: WeakHandle<dyn PrsMgrPresentableObject>,
        next: Handle<PrsMgrPresentation>,
    },
}
pub struct TexturedAppendingGroup {
    transient: StandardTransient,
    inner: NullGroup,
    append_once: AtomicBool,
    append_target: TexturedAppendTarget,
}
rustycad_tkernel::impl_transient!(TexturedAppendingGroup,"TexturedAppendingGroup",StandardTransient,transient,interfaces:[dyn Graphic3dGroup]);
impl TexturedAppendingGroup {
    pub fn new(prs: &Handle<dyn Graphic3dStructureSource>) -> Handle<Self> {
        // The original fixture exercises a real group appended by its own callback.
        Self::with_target(prs, TexturedAppendTarget::Group)
    }
    pub fn with_presentation(
        prs: &Handle<dyn Graphic3dStructureSource>,
        owner: &Handle<dyn PrsMgrPresentableObject>,
        next: &Handle<PrsMgrPresentation>,
    ) -> Handle<Self> {
        let target = TexturedAppendTarget::Presentation {
            owner: owner.downgrade(),
            next: next.clone(),
        };
        Self::with_target(prs, target)
    }
    fn with_target(
        prs: &Handle<dyn Graphic3dStructureSource>,
        target: TexturedAppendTarget,
    ) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            inner: NullGroup::new(prs),
            append_once: AtomicBool::new(true),
            append_target: target,
        })
    }
    fn append_target_once(&self) {
        match &self.append_target {
            TexturedAppendTarget::Group => {
                self.structure().new_group().unwrap();
            }
            TexturedAppendTarget::Presentation { owner, next } => {
                let object = owner.upgrade();
                object
                    .presentable_base()
                    .fields
                    .write()
                    .presentations
                    .push(next.clone());
            }
        }
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
            self.append_target_once();
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
