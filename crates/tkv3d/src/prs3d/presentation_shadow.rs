//! `Prs3d_PresentationShadow`: a structure that shares the groups of a parent structure (its
//! shadow link) and remembers the parent's identifier and view affinity.
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::graphic3d::{
    Graphic3dStructureBase, Graphic3dStructureManager, Graphic3dStructureMembers as _,
    Graphic3dStructureSource, Graphic3dViewAffinity, graphic3d_structure_bind,
    graphic3d_structure_dump_json,
};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;

/// A shadow of a presentation, for highlighting it in another structure manager.
///
/// Example: `let shadow = Prs3dPresentationShadow::new(&manager, &prs)?; shadow.parent_id();`.
pub struct Prs3dPresentationShadow {
    transient: StandardTransient,
    base: Graphic3dStructureBase,
    parent_affinity: Handle<Graphic3dViewAffinity>,
    parent_struct_id: i32,
}
rustycad_tkernel::impl_transient!(Prs3dPresentationShadow, "Prs3d_PresentationShadow", dyn Graphic3dStructureSource, transient,
    interfaces: [dyn Graphic3dStructureSource]);

impl Prs3dPresentationShadow {
    /// The shadow of `prs` in a structure manager; a null or removed parent is refused
    /// (natively a null dereference). Example: `Prs3dPresentationShadow::new(&manager, &prs)?`.
    pub fn new(
        viewer: &Handle<Graphic3dStructureManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<Self>, Failure> {
        let Some(parent) = prs.get() else {
            return Err(null_object(
                "Prs3d_PresentationShadow: null parent presentation; expected a structure",
            ));
        };
        let base = Graphic3dStructureBase::new(viewer, prs)?;
        let affinity = parent
            .cstructure()
            .get()
            .map(|c| c.cstructure_base().fields.read().view_affinity.clone())
            .unwrap_or_default();
        let shadow = Handle::new(Self {
            transient: StandardTransient::default(),
            base,
            parent_affinity: affinity,
            parent_struct_id: parent.identification(),
        });
        graphic3d_structure_bind(&shadow.clone().cast());
        Ok(shadow)
    }
    /// The identifier of the parent structure; `shadow.parent_id()`.
    pub fn parent_id(&self) -> i32 {
        self.parent_struct_id
    }
    /// The view affinity of the parent's driver structure; `shadow.parent_affinity()`.
    pub fn parent_affinity(&self) -> Handle<Graphic3dViewAffinity> {
        self.parent_affinity.clone()
    }
}

impl Graphic3dStructureSource for Prs3dPresentationShadow {
    fn structure_base(&self) -> &Graphic3dStructureBase {
        &self.base
    }
    /// `CalculateBoundBox`: nothing; the shadow keeps the bounds of the shared groups.
    fn calculate_bound_box(&self) {}
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Prs3d_PresentationShadow")?;
        base_class(stream, depth, &|s, d| {
            graphic3d_structure_dump_json(self, s, d)
        })?;
        if let Some(affinity) = self.parent_affinity.get() {
            field_dumped(stream, depth, "myParentAffinity.get()", &|s, d| {
                affinity.dump_json(s, d)
            })?;
        }
        field_integer(stream, "myParentStructId", self.parent_struct_id)
    }
}
