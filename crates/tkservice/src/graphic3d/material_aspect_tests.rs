//! Unit tests of `Graphic3d_MaterialAspect`; the native values are compared by the
//! sv_material probes.
use super::*;
use Graphic3dNameOfMaterial as N;

fn rgb(r: f64, g: f64, b: f64) -> Color {
    Color::new(r, g, b, rustycad_tkernel::quantity::ColorSpace::Rgb).unwrap()
}

#[test]
fn every_predefined_material_builds() {
    for (name, text) in super::MATERIAL_NAMES {
        let material = Graphic3dMaterialAspect::new(name);
        assert_eq!(material.material_name(), text.as_bytes());
        assert_eq!(material.requested_name(), name);
    }
}

#[test]
fn predefined_materials_keep_their_names_and_types() {
    let gold = Graphic3dMaterialAspect::new(N::Gold);
    assert_eq!(gold.string_name().as_bytes(), b"Gold");
    assert_eq!(gold.material_type(), Graphic3dTypeOfMaterial::Physic);
    assert_eq!(
        Graphic3dMaterialAspect::default().string_name().as_bytes(),
        b"Default"
    );
    assert_eq!(
        Graphic3dMaterialAspect::new(N::Aluminum)
            .string_name()
            .as_bytes(),
        b"Aluminium"
    );
    assert!(Graphic3dMaterialAspect::new(N::Water).transparency() == 0.8);
}

#[test]
fn ranks_outside_the_catalogue_raise_out_of_range() {
    assert_eq!(Graphic3dMaterialAspect::number_of_materials(), 24);
    assert_eq!(
        Graphic3dMaterialAspect::material_name_of(1).unwrap(),
        "Brass"
    );
    assert_eq!(
        Graphic3dMaterialAspect::material_name_of(24).unwrap(),
        "Transparent"
    );
    let error = Graphic3dMaterialAspect::material_type_of(25).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(error.message().contains("25"));
}

#[test]
fn names_match_case_insensitively_and_through_aliases() {
    assert_eq!(
        Graphic3dMaterialAspect::material_from_name("gOLD"),
        Some(N::Gold)
    );
    assert_eq!(
        Graphic3dMaterialAspect::material_from_name("default"),
        Some(N::Default)
    );
    assert_eq!(
        Graphic3dMaterialAspect::material_from_name("plastic"),
        Some(N::Plastified)
    );
    assert_eq!(
        Graphic3dMaterialAspect::material_from_name("NEON_PHC"),
        Some(N::Neon)
    );
    assert_eq!(
        Graphic3dMaterialAspect::material_from_name("UserDefined"),
        None
    );
    assert_eq!(
        Graphic3dMaterialAspect::material_from_name_or_default("x"),
        N::Default
    );
}

#[test]
fn set_color_scales_by_the_requested_coefficients() {
    let mut stone = Graphic3dMaterialAspect::new(N::Stone);
    stone.set_color(rgb(1.0, 1.0, 1.0)).unwrap();
    assert_eq!(stone.name(), N::UserDefined);
    assert_eq!(stone.diffuse_color().rgb_vec3().x, 0.75);
    assert_eq!(stone.ambient_color().rgb_vec3().x, 0.19f32 * 0.25);
    let mut plastic = Graphic3dMaterialAspect::new(N::Plastified);
    plastic.set_color(rgb(1.0, 0.0, 0.0)).unwrap();
    assert_eq!(plastic.name(), N::Plastified);
}

#[test]
fn physical_colors_and_shininess_make_the_material_user_defined() {
    let mut gold = Graphic3dMaterialAspect::new(N::Gold);
    gold.set_ambient_color(gold.ambient_color());
    assert_eq!(gold.name(), N::Gold);
    gold.set_specular_color(rgb(0.0, 0.0, 0.0));
    assert_eq!(gold.string_name().as_bytes(), b"UserDefined");
    let mut plastic = Graphic3dMaterialAspect::new(N::Plastified);
    plastic.set_shininess(0.5).unwrap();
    assert_eq!(plastic.name(), N::UserDefined);
    plastic.reset();
    assert_eq!(plastic.name(), N::Plastified);
}

#[test]
fn out_of_range_coefficients_raise_with_the_value() {
    let mut m = Graphic3dMaterialAspect::default();
    let error = m.set_transparency(1.5).unwrap_err();
    assert_eq!(error.exception_type(), "Graphic3d_MaterialDefinitionError");
    assert!(error.message().contains("1.5"));
    assert!(m.set_shininess(-0.1).is_err());
    assert!(m.set_refraction_index(0.5).is_err());
    m.set_alpha(0.25).unwrap();
    assert_eq!(m.transparency(), 0.75);
    assert_eq!(m.pbr_material().alpha(), 0.25);
}

#[test]
fn increase_shine_reverts_out_of_range_results() {
    let mut m = Graphic3dMaterialAspect::new(N::Satin);
    m.increase_shine(100.0);
    assert_eq!(m.shininess(), 0.09375 * 2.0);
    m.increase_shine(-300.0);
    assert_eq!(m.shininess(), 0.09375 * 2.0);
}

#[test]
fn set_material_type_never_marks_user_defined() {
    let mut m = Graphic3dMaterialAspect::new(N::Gold);
    m.set_material_type(Graphic3dTypeOfMaterial::Aspect);
    assert!(m.is_material_type(Graphic3dTypeOfMaterial::Aspect));
    assert_eq!(m.name(), N::Gold);
}

#[test]
#[allow(deprecated)]
fn reflection_mode_off_blackens_the_component() {
    let mut m = Graphic3dMaterialAspect::default();
    assert!(m.reflection_mode(Graphic3dTypeOfReflection::Ambient));
    assert!(!m.reflection_mode(Graphic3dTypeOfReflection::Specular));
    m.set_reflection_mode_off(Graphic3dTypeOfReflection::Ambient);
    assert!(!m.reflection_mode(Graphic3dTypeOfReflection::Ambient));
    assert!(m != Graphic3dMaterialAspect::default());
}

#[test]
fn dump_lists_the_members() {
    let mut stream = DumpStream::default();
    Graphic3dMaterialAspect::default()
        .dump_json(&mut stream, 1)
        .unwrap();
    let text = stream.text();
    assert!(text.starts_with("\"className\": \"Graphic3d_MaterialAspect\""));
    assert!(text.contains("\"StringName\": \"Default\""));
    assert!(text.contains("\"Colors[Graphic3d_TOR_AMBIENT]\""));
    assert!(text.ends_with("\"RequestedMaterialName\": 24"));
}
