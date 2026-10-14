// SPDX-License-Identifier: GPL-2.0-only
//! Actual getters after independent source/output mutations prove value-copy behavior.
use martensite_tkmeshvs::{MeshVSDrawer,MeshVSDrawerSource};
use rustycad_tkernel::{quantity::{Color,NamedColor},standard::{Handle,RuntimeTyped,Transient},tcollection::AsciiString};
use rustycad_tkservice::graphic3d::{Graphic3dMaterialAspect,Graphic3dNameOfMaterial};
#[test]
fn overwrite_all_typed_channels_keeps_exact_values_and_absent_outputs() {
    let drawer=MeshVSDrawer::default();
    drawer.set_double(7,f64::NAN);drawer.set_double(7,f64::INFINITY);
    drawer.set_boolean(7,true);drawer.set_boolean(7,false);
    let mut real=0.;let mut boolean=true;
    assert!(drawer.get_double(7,&mut real));assert_eq!(real,f64::INFINITY);
    assert!(drawer.get_boolean(7,&mut boolean));assert!(!boolean);
    assert!(drawer.remove_double(7));assert!(!drawer.remove_double(7));
    assert!(!drawer.get_double(7,&mut real));assert_eq!(real,f64::INFINITY);
    assert!(drawer.remove_boolean(7));assert!(!drawer.remove_boolean(7));
    assert!(!drawer.get_boolean(7,&mut boolean));assert!(!boolean);
}
#[test]
fn color_material_and_string_overwrites_outputs_and_copies_are_independent() {
    let source=Handle::new(MeshVSDrawer::default());
    fill_values(&source,0.25,"original");
    let target=MeshVSDrawer::default();target.assign(&source);
    fill_values(&source,0.75,"changed");
    assert_values(&target,0.25,"original");assert_values(&source,0.75,"changed");
    let copy=target.clone();fill_values(&target,0.5,"other");
    assert_values(&copy,0.25,"original");assert_eq!(copy.transient_base().ref_count(),0);
    assert!(copy.remove_color(7));assert!(!copy.remove_color(7));
    assert!(copy.remove_material(7));assert!(!copy.remove_material(7));
    assert!(copy.remove_ascii_string(7));assert!(!copy.remove_ascii_string(7));
}
fn fill_values(drawer:&MeshVSDrawer,alpha:f32,text:&str) {
    let color=if alpha<0.5 {NamedColor::RED} else {NamedColor::BLUE};
    let mut material=Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold);
    material.set_transparency(alpha).unwrap();
    drawer.set_color(7,&Color::named(color));drawer.set_material(7,&material);
    drawer.set_ascii_string(7,&AsciiString::new(text));
}
fn assert_values(drawer:&MeshVSDrawer,alpha:f32,text:&str) {
    let mut color=Color::default();let mut material=Graphic3dMaterialAspect::default();
    let mut string=AsciiString::default();
    assert!(drawer.get_color(7,&mut color));assert!(drawer.get_material(7,&mut material));
    assert!(drawer.get_ascii_string(7,&mut string));
    assert_eq!(color.red(),if alpha<0.5 {1.} else {0.});
    assert_eq!(material.transparency(),alpha);assert_eq!(string.as_bytes(),text.as_bytes());
    material.set_transparency(0.).unwrap();string.set_value(1,b"X").unwrap();
    drawer.get_material(7,&mut material);drawer.get_ascii_string(7,&mut string);
    assert_eq!(material.transparency(),alpha);assert_eq!(string.as_bytes(),text.as_bytes());
}
#[test]
fn actual_source_projection_shares_descriptor_base_and_does_not_increment_count() {
    let owner=Handle::new(MeshVSDrawer::default());
    let argument:&dyn rustycad_tkernel::standard::HandleReference<dyn MeshVSDrawerSource>=&owner;
    let observed=argument.observe();
    assert_eq!(owner.ref_count(),1);
    assert_eq!(observed.object_address(),owner.object_address());
    assert!(std::ptr::eq(MeshVSDrawer::static_type(),<dyn MeshVSDrawerSource>::static_type()));
    assert_eq!(MeshVSDrawer::get_type_name(),"MeshVS_Drawer");
    assert!(std::ptr::eq(observed.get().unwrap().transient_base(),owner.transient_base()));
    assert_eq!(argument.reference_address(),std::ptr::from_ref(&owner).cast());
}
