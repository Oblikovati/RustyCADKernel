//! Real StdSelect_Shape owns geometry and the inherited presentable state used by BRep highlight owners.
use super::shape_reference::{StdSelectShapeFields, StdSelectShapeRef};
use crate::{prs_mgr::*, prs3d::Prs3dDrawer, std_prs::*};
use rustycad_tkbrep::topods::{Shape, TopoDSShapeReadSource};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
/// Genuine derived state; `let base=StdSelectShapeBase::new(&shape,&drawer);`.
pub struct StdSelectShapeBase {
    /// Derived geometry is released before its native parent.
    pub shape: SharedState<StdSelectShapeFields>,
    /// The actual inherited presentation parent.
    pub presentable: PrsMgrPresentableObjectBase,
}
impl StdSelectShapeBase {
    /// Initialize the parent before copying geometry and linking the supplied drawer; `Self::new(&shape,&drawer)`.
    pub fn new(shape: &Shape, drawer: &Handle<Prs3dDrawer>) -> Self {
        let presentable = PrsMgrPresentableObjectBase::new(PrsMgrTypeOfPresentation3d::AllView);
        let shape = SharedState::new(StdSelectShapeFields {
            shape: shape.clone(),
        });
        if !drawer.is_null() {
            presentable.fields.read().drawer.set_link(drawer);
        }
        Self { shape, presentable }
    }
}
/// Source dispatch shares the actual derived fields; `source.std_select_shape_base()`.
pub trait StdSelectShapeSource: PrsMgrPresentableObject {
    /// Borrow the genuine subclass state; `source.std_select_shape_base().shape.read()`.
    fn std_select_shape_base(&self) -> &StdSelectShapeBase;
}
impl rustycad_tkernel::standard::RuntimeTyped for dyn StdSelectShapeSource {
    const TYPE_NAME: &'static str =
        <StdSelectShape as rustycad_tkernel::standard::RuntimeTyped>::TYPE_NAME;
    fn static_type() -> &'static rustycad_tkernel::standard::RuntimeType {
        // The application dispatch protocol shares the one genuine native class descriptor.
        <StdSelectShape as rustycad_tkernel::standard::RuntimeTyped>::static_type()
    }
    fn project_from(object: &dyn rustycad_tkernel::standard::Transient) -> Option<&Self> {
        // Registered casters project the actual subclass base without allocating a facade.
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}
/// Nonvirtual native shape-field operations; `source.set_shape(&replacement)`.
pub trait StdSelectShapeMembers: StdSelectShapeSource {
    /// Borrow the field across later setters without retaining geometry; `let reference=source.shape_ref();`.
    fn shape_ref(&self) -> StdSelectShapeRef<'_> {
        let fields = &self.std_select_shape_base().shape;
        StdSelectShapeRef { fields }
    }
    /// Replace actual geometry without presentation invalidation; `source.set_shape(&replacement)`.
    fn set_shape(&self, shape: &Shape) {
        let replacement = shape.clone();
        let old = std::mem::replace(
            &mut self.std_select_shape_base().shape.write().shape,
            replacement,
        );
        drop(old);
    }
}
impl<S: StdSelectShapeSource + ?Sized> StdSelectShapeMembers for S {}
/// Native highlight presentation object; `let object=StdSelectShape::new(&shape,&drawer);`.
pub struct StdSelectShape {
    transient: StandardTransient,
    base: StdSelectShapeBase,
}
rustycad_tkernel::impl_transient!(StdSelectShape,"StdSelect_Shape",dyn PrsMgrPresentableObject,transient,interfaces:[dyn StdSelectShapeSource,dyn PrsMgrPresentableObject]);
impl StdSelectShape {
    /// Allocate real geometry and register its inherited presentation identity; `Self::new(&shape,&drawer)`.
    pub fn new(shape: &Shape, drawer: &Handle<Prs3dDrawer>) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: StdSelectShapeBase::new(shape, drawer),
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
}
impl StdSelectShapeSource for StdSelectShape {
    fn std_select_shape_base(&self) -> &StdSelectShapeBase {
        // The runtime projection borrows one actual subobject.
        &self.base
    }
}
impl PrsMgrPresentableObject for StdSelectShape {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        // Parent ownership is shared with the native derived fields.
        &self.base.presentable
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        std_select_shape_compute(self, prs, mode)
    }
    fn dump_json(
        &self,
        stream: &mut rustycad_tkernel::standard::dump::DumpStream,
        depth: i32,
    ) -> std::io::Result<()> {
        std_select_shape_dump_json(self, stream, depth)
    }
}
/// The native `StdSelect_Shape::DumpJson` body: the presentable parent, then the shape.
/// Example: `std_select_shape_dump_json(&*object, &mut stream, -1)?`.
pub fn std_select_shape_dump_json<S: StdSelectShapeSource + ?Sized>(
    source: &S,
    stream: &mut rustycad_tkernel::standard::dump::DumpStream,
    depth: i32,
) -> std::io::Result<()> {
    use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped};
    class_begin(stream, "StdSelect_Shape")?;
    base_class(stream, depth, &|s, d| {
        prs_mgr_presentable_dump_json(source, s, d)
    })?;
    let shape = source.shape_ref().clone_shape();
    field_dumped(stream, depth, "&mysh", &|s, d| shape.dump_json(s, d))
}
/// Execute native highlight shape mode policy using current geometry; `std_select_shape_compute(source,&prs,0)?`.
pub fn std_select_shape_compute<S: StdSelectShapeSource + ?Sized>(
    source: &S,
    prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    let shape = source.shape_ref();
    if shape.is_null() {
        return Ok(());
    }
    let kind = shape.read_current_shape(|current| current.shape_type())?;
    let drawer = source.attributes();
    if mode == 1 && ((kind as usize) < 5 || kind as usize == 8) {
        return compute_std_select_shading(prs, &shape, &drawer);
    }
    if mode == 0 || mode == 1 {
        return StdPrsWfShape::add_reference(prs, &shape, &drawer);
    }
    Ok(())
}

fn compute_std_select_shading(
    prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
    shape: &StdSelectShapeRef<'_>,
    drawer: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    let group = Handle::default();
    StdPrsShadedShape::add_reference(prs, shape, drawer, StdPrsVolume::Autodetection, &group)
}
