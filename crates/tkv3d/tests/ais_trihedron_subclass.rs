//! Application subclasses retain upstream trihedron RTTI and mutable protected fields.
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkg3d::geom::GeomAxis2Placement;
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, select3d::*};
struct ApplicationTrihedron {
    transient: StandardTransient,
    base: AisTrihedronBase,
}
rustycad_tkernel::impl_transient!(ApplicationTrihedron,"ApplicationTrihedron",dyn AisTrihedronSource,transient,interfaces:[dyn AisTrihedronSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisTrihedronSource for ApplicationTrihedron {
    fn trihedron_base(&self) -> &AisTrihedronBase {
        &self.base
    }
}
impl AisInteractiveObject for ApplicationTrihedron {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.interactive
    }
}
impl PrsMgrPresentableObject for ApplicationTrihedron {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.interactive.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_trihedron_compute(self, prs, mode)
    }
}
impl SelectMgrSelectableObject for ApplicationTrihedron {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_trihedron_compute_selection(self, selection, mode)
    }
}
struct ApplicationTrihedronOwner {
    transient: StandardTransient,
    base: AisTrihedronOwnerBase,
}
rustycad_tkernel::impl_transient!(ApplicationTrihedronOwner,"ApplicationTrihedronOwner",dyn AisTrihedronOwnerSource,transient,interfaces:[dyn AisTrihedronOwnerSource,dyn SelectMgrEntityOwnerSource]);
impl AisTrihedronOwnerSource for ApplicationTrihedronOwner {
    fn trihedron_owner_base(&self) -> &AisTrihedronOwnerBase {
        &self.base
    }
}
impl SelectMgrEntityOwnerSource for ApplicationTrihedronOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base.owner
    }
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        _: i32,
    ) -> Result<(), Failure> {
        ais_trihedron_owner_hilight(self, manager, style)
    }
}
#[test]
fn derived_protected_cache_and_ownership_flags_reach_generic_callbacks() {
    let object = Handle::new(ApplicationTrihedron {
        transient: StandardTransient::default(),
        base: AisTrihedronBase::new(&Handle::new(GeomAxis2Placement::new(Axis2::default())))
            .unwrap(),
    });
    prs_mgr_presentable_bind(&object.cast());
    assert!(!object.cast::<dyn AisTrihedronSource>().is_null());
    object.set_own_datum_aspect().unwrap();
    object
        .set_text_color(Color::named(NamedColor::RED))
        .unwrap();
    object.base.fields.write().has_own_text_color = true;
    ais_trihedron_unset_color(&*object).unwrap();
    assert!(!object.has_text_color());
    assert_eq!(
        object.text_color(),
        Color::named(NamedColor::LIGHTSTEELBLUE4)
    );
    let array = Handle::new(Graphic3dArrayOfSegments::new(2, 0, 0).unwrap());
    array.add_vertex_pnt(&Point3::new(1., 2., 3.)).unwrap();
    array.add_vertex_pnt(&Point3::new(4., 5., 6.)).unwrap();
    object.base.fields.write().primitives[Prs3dDatumParts::XAxis as usize] = array.cast();
    let owner = AisTrihedronOwner::new(&object.cast(), Prs3dDatumParts::XAxis, 7);
    let sensitive = ais_trihedron_sensitive(&*object, Prs3dDatumParts::XAxis, &owner.cast())
        .unwrap()
        .cast::<Select3DSensitiveSegment>();
    assert_eq!(sensitive.start_point(), Point3::new(1., 2., 3.));
    assert_eq!(sensitive.end_point(), Point3::new(4., 5., 6.));
}
#[test]
fn derived_owner_keeps_native_interface_and_replaceable_part() {
    let owner = Handle::new(ApplicationTrihedronOwner {
        transient: StandardTransient::default(),
        base: AisTrihedronOwnerBase::new(&Handle::default(), Prs3dDatumParts::XAxis, 7),
    });
    select_mgr_entity_owner_bind(&owner.cast());
    *owner.base.part.write() = Prs3dDatumParts::XOYAxis;
    let native = owner.cast::<dyn AisTrihedronOwnerSource>();
    assert_eq!(native.datum_part(), Prs3dDatumParts::XOYAxis);
    assert_eq!(native.priority(), 7);
    assert!(
        native
            .hilight_with_color(&Handle::default(), &Handle::default(), 0)
            .is_err()
    );
}
