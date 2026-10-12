//! Martensite per-subshape ownership flags over the genuine presentation drawer.
use crate::prs3d::{Prs3dDrawer, Prs3dDrawerFields};
use rustycad_tkernel::{
    quantity::Color,
    standard::{Handle, HandleFieldRef, SharedState},
};
use rustycad_tkservice::graphic3d::{
    Graphic3dPresentationAttributesBase, Graphic3dPresentationAttributesSource,
};

/// Public native flags sharing the same storage as every member call.
/// Example: `drawer.flags.write().is_hidden = true`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AisColoredDrawerFlags {
    /// Native `myIsHidden`; `drawer.flags.write().is_hidden = true`.
    pub is_hidden: bool,
    /// Native `myHasOwnMaterial`; `drawer.flags.write().has_own_material = true`.
    pub has_own_material: bool,
    /// Native `myHasOwnColor`; `drawer.flags.write().has_own_color = true`.
    pub has_own_color: bool,
    /// Native `myHasOwnTransp`; `drawer.flags.write().has_own_transparency = true`.
    pub has_own_transparency: bool,
    /// Native `myHasOwnWidth`; `drawer.flags.write().has_own_width = true`.
    pub has_own_width: bool,
}
/// The actual inherited drawer type; `let parent: Handle<AisColoredDrawerParent> = colored.cast()`.
pub type AisColoredDrawerParent = Prs3dDrawer;

/// A linked presentation drawer with independent per-subshape override flags.
/// Example: `AisColoredDrawer::new(&Handle::new(Prs3dDrawer::new()))`.
pub struct AisColoredDrawer {
    drawer: Prs3dDrawer,
    /// Writable native public fields; guards borrow actual storage, never copied flag proxies.
    pub flags: SharedState<AisColoredDrawerFlags>,
}
rustycad_tkernel::impl_transient!(AisColoredDrawer, "AIS_ColoredDrawer", Prs3dDrawer, drawer,
    interfaces: [dyn Graphic3dPresentationAttributesSource]);
impl AisColoredDrawer {
    /// Links the actual parent handle while leaving every ownership flag false.
    /// Example: `AisColoredDrawer::new(&defaults)`.
    pub fn new(link: &Handle<Prs3dDrawer>) -> Self {
        let drawer = Prs3dDrawer::new();
        drawer.link_to(link);
        Self {
            drawer,
            flags: SharedState::default(),
        }
    }
    /// Borrows the actual inherited link slot without retaining its current target.
    /// Example: `let alias=colored.link_ref(); colored.set_link(&replacement); alias.clone_handle()`.
    pub fn link_ref(&self) -> HandleFieldRef<'_, Prs3dDrawerFields, Prs3dDrawer> {
        let fields = &self.drawer.fields;
        HandleFieldRef::new(fields, select_colored_drawer_link)
    }
    /// Reads visibility independently of inherited draw flags; `colored.is_hidden()`.
    pub fn is_hidden(&self) -> bool {
        let flags = self.flags.read();
        flags.is_hidden
    }
    /// Changes visibility without changing any aspect; `colored.set_hidden(true)`.
    pub fn set_hidden(&self, hidden: bool) {
        let mut flags = self.flags.write();
        flags.is_hidden = hidden;
    }
    /// Reads material ownership; `colored.has_own_material()`.
    pub fn has_own_material(&self) -> bool {
        let flags = self.flags.read();
        flags.has_own_material
    }
    /// Clears only material ownership; `colored.unset_own_material()`.
    pub fn unset_own_material(&self) {
        let mut flags = self.flags.write();
        flags.has_own_material = false;
    }
    /// Marks material ownership without creating an aspect; `colored.set_own_material()`.
    pub fn set_own_material(&self) {
        let mut flags = self.flags.write();
        flags.has_own_material = true;
    }
    /// Reads color ownership; `colored.has_own_color()`.
    pub fn has_own_color(&self) -> bool {
        let flags = self.flags.read();
        flags.has_own_color
    }
    /// Clears only color ownership; `colored.unset_own_color()`.
    pub fn unset_own_color(&self) {
        let mut flags = self.flags.write();
        flags.has_own_color = false;
    }
    /// Marks color ownership; native OCCT 3d097a0 ignores the color argument.
    /// Example: `colored.set_own_color(Color::default())`.
    pub fn set_own_color(&self, _color: Color) {
        // Appearance belongs to AIS_ColoredShape::setColor, not this flag-only member.
        let mut flags = self.flags.write();
        flags.has_own_color = true;
    }
    /// Reads transparency ownership; `colored.has_own_transparency()`.
    pub fn has_own_transparency(&self) -> bool {
        let flags = self.flags.read();
        flags.has_own_transparency
    }
    /// Clears only transparency ownership; `colored.unset_own_transparency()`.
    pub fn unset_own_transparency(&self) {
        let mut flags = self.flags.write();
        flags.has_own_transparency = false;
    }
    /// Marks ownership without validating or storing the ignored native value.
    /// Example: `colored.set_own_transparency(f64::NAN)`.
    pub fn set_own_transparency(&self, _transparency: f64) {
        // The pinned inline setter accepts NaN and out-of-range arguments unchanged.
        let mut flags = self.flags.write();
        flags.has_own_transparency = true;
    }
    /// Reads width ownership; `colored.has_own_width()`.
    pub fn has_own_width(&self) -> bool {
        let flags = self.flags.read();
        flags.has_own_width
    }
    /// Clears only width ownership; `colored.unset_own_width()`.
    pub fn unset_own_width(&self) {
        let mut flags = self.flags.write();
        flags.has_own_width = false;
    }
    /// Marks ownership without validating or storing the ignored native value.
    /// Example: `colored.set_own_width(-1.0)`.
    pub fn set_own_width(&self, _width: f64) {
        // Actual line-width validation happens in the separate presentation-aspect setter.
        let mut flags = self.flags.write();
        flags.has_own_width = true;
    }
}
impl std::ops::Deref for AisColoredDrawer {
    type Target = Prs3dDrawer;
    /// Borrows the genuine owned parent; `colored.link()`.
    fn deref(&self) -> &Prs3dDrawer {
        // Base calls read the owned parent, rather than copying its presentation fields.
        &self.drawer
    }
}
impl Graphic3dPresentationAttributesSource for AisColoredDrawer {
    fn presentation_attributes_base(&self) -> &Graphic3dPresentationAttributesBase {
        let drawer = &self.drawer;
        drawer.presentation_attributes_base()
    }
    fn dump_json(
        &self,
        stream: &mut rustycad_tkernel::standard::dump::DumpStream,
        depth: i32,
    ) -> std::io::Result<()> {
        // AIS_ColoredDrawer does not override the inherited virtual DumpJson method.
        self.drawer.dump_json(stream, depth)
    }
}

fn select_colored_drawer_link(fields: &Prs3dDrawerFields) -> &Handle<Prs3dDrawer> {
    // A retained native Link reference follows field reassignment instead of pinning the old target.
    &fields.link
}
