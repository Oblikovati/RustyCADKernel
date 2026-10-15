// SPDX-License-Identifier: GPL-2.0-only
//! The body of the `MeshVS_Mesh` constructor: the object appearance and the current,
//! selection and highlight drawers.
use super::MeshVSMeshSource;
use crate::flags::MESHVS_DMF_WIRE_FRAME;
use crate::{MeshVSDrawer, MeshVSDrawerAttribute as Key, MeshVSDrawerSource};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::aspect::{AspectInteriorStyle, AspectTypeOfLine, AspectTypeOfMarker};
use rustycad_tkservice::graphic3d::{
    Graphic3dMaterialAspect, Graphic3dNameOfMaterial, Graphic3dPresentationAttributesSource as _,
};
use rustycad_tkv3d::prs_mgr::{PrsMgrPresentableObjectMembers, prs_mgr_set_material};

/// The native constructor body, run once the object is bound: no automatic highlight,
/// wireframe display and highlight modes, a white plastic appearance and the three drawers.
/// Example: `meshvs_mesh_initialize(&*mesh, false)?`.
pub fn meshvs_mesh_initialize<O: MeshVSMeshSource + ?Sized>(
    object: &O,
    allow_overlapped: bool,
) -> Result<(), Failure> {
    object.set_auto_hilight(false);
    // SetDisplayMode asks AcceptDisplayMode, which accepts every positive mode without builder.
    object.attributes().set_display_mode(MESHVS_DMF_WIRE_FRAME);
    object.set_hilight_mode(MESHVS_DMF_WIRE_FRAME);
    // The constructor calls the PrsMgr_PresentableObject bodies of SetColor and SetMaterial.
    object
        .attributes()
        .set_color(Color::named(NamedColor::WHITE));
    object.presentable_base().fields.write().has_own_color = true;
    let plastic = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified);
    prs_mgr_set_material(object, &plastic)?;
    let current = current_drawer(allow_overlapped);
    let selection = selection_drawer(&current);
    let hilight = hilight_drawer(&current);
    let mut fields = object.mesh_base().fields.write();
    fields.current_drawer = current;
    fields.selection_drawer = selection;
    fields.hilight_drawer = hilight;
    Ok(())
}

fn new_drawer() -> Handle<dyn MeshVSDrawerSource> {
    Handle::new(MeshVSDrawer::default()).cast()
}

fn current_drawer(allow_overlapped: bool) -> Handle<dyn MeshVSDrawerSource> {
    let handle = new_drawer();
    let drawer = &**handle.get().expect("new drawer");
    drawer.set_color(Key::InteriorColor as i32, &Color::named(NamedColor::BLUE4));
    drawer.set_integer(Key::InteriorStyle as i32, AspectInteriorStyle::Solid as i32);
    drawer.set_integer(Key::MaxFaceNodes as i32, 10);
    drawer.set_boolean(Key::IsAllowOverlapped as i32, allow_overlapped);
    drawer.set_boolean(Key::Reflection as i32, true);
    drawer.set_double(Key::ShrinkCoeff as i32, 0.8);
    drawer.set_boolean(Key::ComputeTime as i32, false);
    drawer.set_boolean(Key::ComputeSelectionTime as i32, false);
    drawer.set_boolean(Key::DisplayNodes as i32, true);
    drawer.set_double(Key::EdgeWidth as i32, 1.0);
    drawer.set_integer(Key::EdgeType as i32, AspectTypeOfLine::Solid as i32);
    drawer.set_integer(Key::MarkerType as i32, AspectTypeOfMarker::O as i32);
    drawer.set_color(Key::MarkerColor as i32, &Color::named(NamedColor::WHITE));
    drawer.set_double(Key::MarkerScale as i32, 1.0);
    drawer.set_integer(Key::BeamType as i32, AspectTypeOfLine::Solid as i32);
    drawer.set_double(Key::BeamWidth as i32, 1.0);
    drawer.set_boolean(Key::SmoothShading as i32, false);
    drawer.set_boolean(Key::SupressBackFaces as i32, false);
    handle
}

fn copied_drawer(current: &Handle<dyn MeshVSDrawerSource>) -> Handle<dyn MeshVSDrawerSource> {
    let handle = new_drawer();
    handle.get().expect("new drawer").assign(current);
    handle
}

fn selection_drawer(current: &Handle<dyn MeshVSDrawerSource>) -> Handle<dyn MeshVSDrawerSource> {
    let handle = copied_drawer(current);
    let drawer = &**handle.get().expect("new drawer");
    drawer.set_integer(Key::MarkerType as i32, AspectTypeOfMarker::Star as i32);
    drawer.set_color(Key::MarkerColor as i32, &Color::named(NamedColor::GRAY80));
    drawer.set_double(Key::MarkerScale as i32, 2.0);
    drawer.set_color(Key::BeamColor as i32, &Color::named(NamedColor::GRAY80));
    drawer.set_integer(Key::BeamType as i32, AspectTypeOfLine::Solid as i32);
    drawer.set_double(Key::BeamWidth as i32, 3.0);
    handle
}

fn hilight_drawer(current: &Handle<dyn MeshVSDrawerSource>) -> Handle<dyn MeshVSDrawerSource> {
    let handle = copied_drawer(current);
    let drawer = &**handle.get().expect("new drawer");
    let plastic = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified);
    drawer.set_double(Key::ShrinkCoeff as i32, 0.7);
    drawer.set_integer(Key::InteriorStyle as i32, AspectInteriorStyle::Solid as i32);
    drawer.set_color(Key::InteriorColor as i32, &Color::named(NamedColor::YELLOW));
    drawer.set_color(
        Key::BackInteriorColor as i32,
        &Color::named(NamedColor::YELLOW),
    );
    drawer.set_color(Key::EdgeColor as i32, &Color::named(NamedColor::GREEN));
    drawer.set_integer(Key::EdgeType as i32, AspectTypeOfLine::Solid as i32);
    drawer.set_double(Key::EdgeWidth as i32, 1.0);
    drawer.set_material(Key::FrontMaterial as i32, &plastic);
    drawer.set_material(Key::BackMaterial as i32, &plastic);
    drawer.set_color(Key::BeamColor as i32, &Color::named(NamedColor::GRAY80));
    drawer.set_integer(Key::BeamType as i32, AspectTypeOfLine::Solid as i32);
    drawer.set_double(Key::BeamWidth as i32, 3.0);
    drawer.set_integer(Key::MarkerType as i32, AspectTypeOfMarker::Star as i32);
    drawer.set_color(Key::MarkerColor as i32, &Color::named(NamedColor::GRAY80));
    drawer.set_double(Key::MarkerScale as i32, 2.0);
    handle
}
