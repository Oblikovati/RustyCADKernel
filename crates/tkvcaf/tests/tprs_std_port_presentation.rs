//! Port tests of TPrsStd_AISViewer and TPrsStd_AISPresentation: the interactive object built
//! by the drivers, its display state in the viewer's context and the undo callbacks.
mod support;

use rustycad_tkcaf::tdata_xtd::{TDataXtdPoint, TDataXtdPresentation};
use rustycad_tkernel::quantity::NamedColor;
use rustycad_tkernel::standard::Handle;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::Graphic3dNameOfMaterial;
use rustycad_tkv3d::ais::{AisInteractiveContext, AisInteractiveObjectMembers, AisPoint};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkvcaf::tprs_std::{TPrsStdAISPresentation, TPrsStdAISViewer};

/// A point label with its presentation (driver `TDataXtd_Point`).
fn point_presentation(label: &TdfLabel) -> Handle<TPrsStdAISPresentation> {
    TDataXtdPoint::set_point(label, Point3::new(1.0, 2.0, 3.0)).unwrap();
    TPrsStdAISPresentation::set(label, &TDataXtdPoint::get_id()).unwrap()
}

/// The viewer context of a document's root.
fn context_of(label: &TdfLabel) -> Handle<AisInteractiveContext> {
    let mut context = Handle::default();
    assert!(TPrsStdAISViewer::find_context(label, &mut context).unwrap());
    context
}

#[test]
fn display_without_viewer_builds_the_owned_point_only() {
    let document = support::new_document();
    let label = support::main_child(&document, 1);
    let presentation = point_presentation(&label);
    presentation.display(true).unwrap();
    let object = presentation.get_ais();
    assert!(!object.cast::<AisPoint>().is_null());
    let owner = object.get_owner().clone();
    assert_eq!(owner.object_address(), presentation.object_address());
    assert!(!presentation.is_displayed().unwrap());
}

#[test]
fn viewer_new_sets_presentations_and_refuses_a_second_viewer() {
    let document = support::new_document();
    let label = support::main_child(&document, 1);
    TDataXtdPresentation::set(&label, &TDataXtdPoint::get_id()).unwrap();
    assert!(!TPrsStdAISViewer::has(&label).unwrap());
    let viewer = TPrsStdAISViewer::new_with_viewer(&label, &support::null_viewer()).unwrap();
    assert!(TPrsStdAISViewer::has(&label).unwrap());
    let mut presentation: Handle<TPrsStdAISPresentation> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TPrsStdAISPresentation::get_id(), &mut presentation)
            .unwrap()
    );
    let mut found = Handle::default();
    assert!(TPrsStdAISViewer::find(&label, &mut found).unwrap());
    assert_eq!(found.object_address(), viewer.object_address());
    let mut v3d = Handle::default();
    assert!(TPrsStdAISViewer::find_viewer(&label, &mut v3d).unwrap());
    assert!(!v3d.is_null());
    let again = TPrsStdAISViewer::new_with_context(&label, &viewer.get_interactive_context());
    assert!(again.is_err_and(|failure| failure.is_kind("Standard_DomainError")));
}

#[test]
fn display_and_erase_follow_the_viewer_context() {
    let document = support::new_document();
    let label = support::main_child(&document, 1);
    TPrsStdAISViewer::new_with_viewer(&label, &support::null_viewer()).unwrap();
    let presentation = point_presentation(&label);
    presentation.display(false).unwrap();
    let context = context_of(&label);
    let object = presentation.get_ais();
    assert!(context.is_displayed(&object));
    assert!(presentation.is_displayed().unwrap());
    presentation.erase(false).unwrap();
    assert!(!context.is_displayed(&object));
    assert!(!presentation.is_displayed().unwrap());
    assert!(!presentation.get_ais().is_null());
    presentation.erase(true).unwrap();
    assert!(presentation.get_ais().is_null());
    TPrsStdAISViewer::update_label(&label).unwrap();
}

#[test]
fn owned_settings_reach_the_object() {
    let document = support::new_document();
    let label = support::main_child(&document, 1);
    TPrsStdAISViewer::new_with_viewer(&label, &support::null_viewer()).unwrap();
    let presentation = point_presentation(&label);
    presentation.display(false).unwrap();
    presentation.set_color(NamedColor::RED).unwrap();
    presentation.set_width(3.0).unwrap();
    presentation.set_transparency(0.5).unwrap();
    presentation
        .set_material(Graphic3dNameOfMaterial::Jade)
        .unwrap();
    let object = presentation.get_ais();
    assert_eq!(object.color().name(), NamedColor::RED);
    assert!(object.has_width() && object.width() == 3.0);
    assert_eq!(object.transparency(), 0.5);
    assert_eq!(object.material(), Graphic3dNameOfMaterial::Jade);
    assert_eq!(
        presentation.material().unwrap(),
        Graphic3dNameOfMaterial::Jade
    );
    presentation.unset_color().unwrap();
    presentation.unset_width().unwrap();
    presentation.unset_transparency().unwrap();
    presentation.unset_material().unwrap();
    assert!(!object.has_color() && !object.has_width() && !object.has_material());
    assert!(!presentation.has_own_color().unwrap());
}

#[test]
fn selection_modes_activate_in_the_context() {
    let document = support::new_document();
    let label = support::main_child(&document, 1);
    TPrsStdAISViewer::new_with_viewer(&label, &support::null_viewer()).unwrap();
    let presentation = point_presentation(&label);
    presentation.display(false).unwrap();
    presentation.set_selection_mode(1, true).unwrap();
    presentation.add_selection_mode(0, true).unwrap();
    assert_eq!(presentation.get_nb_selection_modes().unwrap(), 2);
    let mut modes = rustycad_tkernel::ncollection::List::default();
    context_of(&label).activated_modes(&presentation.get_ais(), &mut modes);
    assert!(modes.iter().any(|mode| *mode == 1) && modes.iter().any(|mode| *mode == 0));
    presentation.unset_selection_mode().unwrap();
    assert!(!presentation.has_own_selection_mode().unwrap());
}

#[test]
fn unset_forgets_the_presentation_and_its_data() {
    let document = support::new_document();
    let label = support::main_child(&document, 1);
    let presentation = point_presentation(&label);
    TDataXtdPresentation::unset(&label).unwrap();
    let failure = presentation.get_data().err().unwrap();
    assert!(failure.is_kind("Standard_NoMoreObject"));
    TPrsStdAISPresentation::unset(&label).unwrap();
    assert!(
        !label
            .is_attribute(&TPrsStdAISPresentation::get_id())
            .unwrap()
    );
    let failure = presentation.get_data().err().unwrap();
    assert!(failure.is_kind("Standard_NullObject"));
}

#[test]
fn undo_of_the_addition_removes_the_object_from_the_context() {
    let document = support::new_document();
    document.set_undo_limit(10).unwrap();
    let label = support::main_child(&document, 1);
    TPrsStdAISViewer::new_with_viewer(&label, &support::null_viewer()).unwrap();
    document.open_command().unwrap();
    let presentation = point_presentation(&label);
    presentation.display(false).unwrap();
    document.commit_command().unwrap();
    let object = presentation.get_ais();
    let context = context_of(&label);
    assert!(context.is_displayed(&object));
    assert!(document.undo().unwrap());
    assert!(!context.is_displayed(&object));
    assert!(
        !label
            .is_attribute(&TPrsStdAISPresentation::get_id())
            .unwrap()
    );
}

#[test]
fn set_from_master_uses_the_master_id_as_driver() {
    let document = support::new_document();
    let label = support::main_child(&document, 1);
    let point = TDataXtdPoint::set_point(&label, Point3::new(0.0, 0.0, 0.0)).unwrap();
    let presentation = TPrsStdAISPresentation::set_from_master(&point.cast()).unwrap();
    assert_eq!(
        presentation.get_driver_guid().unwrap(),
        TDataXtdPoint::get_id()
    );
    presentation.update().unwrap();
    assert!(!presentation.get_ais().cast::<AisPoint>().is_null());
}
