//! Bounded face-adaptor seam preserves actual application virtual calls and the native nonvirtual Face core.
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
/// Native BRepAdaptor_Surface subclasses embed their real core and override Adaptor3d calls.
/// Example: `impl StdPrsIsolineSurfaceSource for Wrapper { fn isoline_surface_base(&self) -> &BRepAdaptorSurface { &self.core } }`.
pub trait StdPrsIsolineSurfaceSource: Adaptor3dSurface {
    /// Project actual owned BRep adaptor state, not a copied face; `surface.isoline_surface_base().face()`.
    fn isoline_surface_base(&self) -> &BRepAdaptorSurface;
}
impl StdPrsIsolineSurfaceSource for BRepAdaptorSurface {
    fn isoline_surface_base(&self) -> &BRepAdaptorSurface {
        // Native Face is nonvirtual even when algorithms dispatch other adaptor methods virtually.
        self
    }
}

impl rustycad_tkernel::standard::RuntimeTyped for dyn StdPrsIsolineSurfaceSource {
    const TYPE_NAME: &'static str = "BRepAdaptor_Surface";
    fn static_type() -> &'static rustycad_tkernel::standard::RuntimeType {
        // The dispatch seam shares the actual BRep descriptor, without inventing another native ancestor.
        <BRepAdaptorSurface as rustycad_tkernel::standard::RuntimeTyped>::static_type()
    }
    fn project_from(object: &dyn rustycad_tkernel::standard::Transient) -> Option<&Self> {
        // Registered application interface views preserve virtual dispatch and real embedded parent state.
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}
