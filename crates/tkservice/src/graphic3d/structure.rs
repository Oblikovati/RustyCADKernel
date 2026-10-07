//! `Graphic3d_Structure`: a presentation structure (visualization-design.md section 1). The
//! class is concrete but derived (PrsMgr_Presentation overrides the virtual members), so it
//! is the [`Graphic3dStructureSource`] trait over the embedded [`Graphic3dStructureBase`],
//! with the non-virtual members in [`Graphic3dStructureMembers`] and the concrete class
//! [`Graphic3dStructure`].
use super::structure_bounds::calculate_bound_box;
use super::structure_display::{
    structure_clear, structure_display, structure_erase, structure_set_visual,
};
use super::{
    Graphic3dCStructure, Graphic3dCamera, Graphic3dStructureManager, Graphic3dTypeOfStructure,
};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{
    Failure, Handle, SharedState, StandardTransient, Transient, WeakHandle,
};
use std::io;

/// The protected fields of a structure.
#[derive(Clone)]
pub struct Graphic3dStructureFields {
    /// `myStructureManager` (a raw pointer natively).
    pub structure_manager: WeakHandle<Graphic3dStructureManager>,
    /// `myCStructure`: null once the structure is removed.
    pub cstructure: Handle<dyn Graphic3dCStructure>,
    /// `myAncestors`: the parent structures (raw pointers natively), in index order.
    pub ancestors: Vec<WeakHandle<dyn Graphic3dStructureSource>>,
    /// `myDescendants`: the child structures (raw pointers natively), in index order.
    pub descendants: Vec<WeakHandle<dyn Graphic3dStructureSource>>,
    /// `myOwner`: the opaque owner (a `void*` natively).
    pub owner: WeakHandle<StandardTransient>,
    /// `myVisual`.
    pub visual: Graphic3dTypeOfStructure,
    /// `myComputeVisual`.
    pub compute_visual: Graphic3dTypeOfStructure,
}

/// The embedded state of a structure.
pub struct Graphic3dStructureBase {
    /// The fields, shared by every handle.
    pub fields: SharedState<Graphic3dStructureFields>,
    /// The structure itself (`this`), set by [`graphic3d_structure_bind`].
    this: SharedState<WeakHandle<dyn Graphic3dStructureSource>>,
}

impl Graphic3dStructureBase {
    /// The constructor: a driver structure from the manager's driver, or the shadow link
    /// of `link` with its owner and visuals (a computed visual is not inherited).
    /// A null manager, driver or link structure is refused (natively a null dereference).
    /// Example: `Graphic3dStructureBase::new(&manager, &Handle::default())?`.
    pub fn new(
        manager: &Handle<Graphic3dStructureManager>,
        link: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Self, Failure> {
        let Some(manager_ref) = manager.get() else {
            return Err(program_error(
                "Graphic3d_Structure: null structure manager; expected a manager",
            ));
        };
        let mut fields = Graphic3dStructureFields {
            structure_manager: manager.downgrade(),
            cstructure: Handle::default(),
            ancestors: Vec::new(),
            descendants: Vec::new(),
            owner: WeakHandle::default(),
            visual: Graphic3dTypeOfStructure::All,
            compute_visual: Graphic3dTypeOfStructure::All,
        };
        fields.cstructure = match link.get() {
            Some(link) => inherit_link(&mut fields, link, manager)?,
            None => create_cstructure(manager_ref, manager)?,
        };
        Ok(Self {
            fields: SharedState::new(fields),
            this: SharedState::new(WeakHandle::default()),
        })
    }
    /// The structure itself, null before [`graphic3d_structure_bind`]; `base.this()`.
    pub fn this(&self) -> Handle<dyn Graphic3dStructureSource> {
        self.this.read().upgrade()
    }
}

/// The driver structure created by the manager's driver.
fn create_cstructure(
    manager_ref: &Graphic3dStructureManager,
    manager: &Handle<Graphic3dStructureManager>,
) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
    let driver = manager_ref.graphic_driver();
    let Some(driver) = driver.get() else {
        return Err(program_error(
            "Graphic3d_Structure: null graphic driver; expected a driver",
        ));
    };
    driver.create_structure(manager)
}

/// Copies the owner and visuals of `link` and returns the shadow link of its driver structure.
fn inherit_link(
    fields: &mut Graphic3dStructureFields,
    link: &dyn Graphic3dStructureSource,
    manager: &Handle<Graphic3dStructureManager>,
) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
    let link_fields = link.structure_base().fields.read().clone();
    fields.owner = link_fields.owner;
    if link_fields.visual != Graphic3dTypeOfStructure::Computed {
        fields.visual = link_fields.visual;
    }
    fields.compute_visual = link_fields.compute_visual;
    let Some(cstructure) = link_fields.cstructure.get() else {
        return Err(program_error(
            "Graphic3d_Structure: the link structure is removed; expected a live structure",
        ));
    };
    cstructure.shadow_link(manager)
}

/// Records `structure` as its own `this`, which the manager and the views receive; every
/// constructor of a structure class calls it once the handle exists.
/// Example: `graphic3d_structure_bind(&handle)`.
pub fn graphic3d_structure_bind(structure: &Handle<dyn Graphic3dStructureSource>) {
    if let Some(value) = structure.get() {
        *value.structure_base().this.write() = structure.downgrade();
    }
}

impl Drop for Graphic3dStructureBase {
    /// The destructor removes the structure, as `~Graphic3d_Structure` calls `Remove()`.
    fn drop(&mut self) {
        let address = self.this.read().object_address();
        super::structure_network::remove_fields(&mut self.fields.write(), address);
    }
}

/// A presentation structure; the display and computation members may be overridden.
pub trait Graphic3dStructureSource: Transient {
    /// The embedded base; `structure.structure_base()`.
    fn structure_base(&self) -> &Graphic3dStructureBase;
    /// `Clear`: clears the groups (destroying them when asked) and the views' computed
    /// structures; `structure.clear(true)`.
    fn clear(&self, with_destruction: bool) {
        structure_clear(self, with_destruction);
    }
    /// `Display`: sticks the structure in the scene graph and makes it visible; `structure.display()`.
    fn display(&self) {
        structure_display(self);
    }
    /// `Erase`: removes the structure from the scene graph; `structure.erase()`.
    fn erase(&self) {
        structure_erase(self);
    }
    /// `CalculateBoundBox`: the driver structure box from the groups and the descendants;
    /// `structure.calculate_bound_box()`.
    fn calculate_bound_box(&self) {
        calculate_bound_box(self);
    }
    /// `SetVisual`: changes the visual (erasing and displaying again a displayed structure);
    /// `structure.set_visual(Graphic3dTypeOfStructure::Shading)`.
    fn set_visual(&self, visual: Graphic3dTypeOfStructure) {
        structure_set_visual(self, visual);
    }
    /// `Compute`: nothing by default; `structure.compute()`.
    fn compute(&self) {}
    /// `computeHLR`: nothing by default; `structure.compute_hlr(&camera, &mut computed)`.
    fn compute_hlr(
        &self,
        projector: &Handle<Graphic3dCamera>,
        structure: &mut Handle<dyn Graphic3dStructureSource>,
    ) {
        let _ = (projector, structure);
    }
    /// `RecomputeTransformation`: nothing by default; `structure.recompute_transformation(&camera)`.
    fn recompute_transformation(&self, projector: &Handle<Graphic3dCamera>) {
        let _ = projector;
    }
    /// `IsDisplayed`: not removed and stuck in the scene graph; `structure.is_displayed()`.
    fn is_displayed(&self) -> bool {
        let cstructure = self.structure_base().fields.read().cstructure.clone();
        cstructure
            .get()
            .is_some_and(|c| c.cstructure_base().fields.read().stick)
    }
    /// `DumpJson`: the manager address, the driver structure, the network addresses, the
    /// owner and the visuals; `structure.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        super::driver_dump::dump_structure(self, stream, depth)
    }
    /// `IsHighlighted`: not removed and highlighted; `structure.is_highlighted()`.
    fn is_highlighted(&self) -> bool {
        let cstructure = self.structure_base().fields.read().cstructure.clone();
        cstructure
            .get()
            .is_some_and(|c| c.cstructure_base().fields.read().highlight)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dStructureSource,
    "Graphic3d_Structure",
    StandardTransient
);

/// The concrete `Graphic3d_Structure`; `Graphic3dStructure::new(&manager, &Handle::default())?`.
pub struct Graphic3dStructure {
    transient: StandardTransient,
    base: Graphic3dStructureBase,
}
rustycad_tkernel::impl_transient!(Graphic3dStructure, "Graphic3d_Structure", StandardTransient, transient,
    interfaces: [dyn Graphic3dStructureSource]);
impl Graphic3dStructureSource for Graphic3dStructure {
    fn structure_base(&self) -> &Graphic3dStructureBase {
        &self.base
    }
}

impl Graphic3dStructure {
    /// A structure of the manager, or a shadow of `link` when not null;
    /// `Graphic3dStructure::new(&manager, &Handle::default())?`.
    pub fn new(
        manager: &Handle<Graphic3dStructureManager>,
        link: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<Self>, Failure> {
        let base = Graphic3dStructureBase::new(manager, link)?;
        let handle = Handle::new(Self {
            transient: StandardTransient::default(),
            base,
        });
        graphic3d_structure_bind(&handle.cast());
        Ok(handle)
    }
}
