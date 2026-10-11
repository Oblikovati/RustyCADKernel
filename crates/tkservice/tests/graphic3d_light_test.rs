//! Graphic3d_CLight and Graphic3d_LightSet behaviour that the numeric probe does not print:
//! identifiers, dumps and the null arguments.
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::graphic3d::{
    Graphic3dCLight, Graphic3dLightSet, Graphic3dLightSetIterationFilter,
    Graphic3dLightSetIterator, Graphic3dTypeOfLightSource as T,
};

fn dump(light: &Graphic3dCLight) -> std::io::Result<String> {
    let mut stream = DumpStream::default();
    light.dump_json(&mut stream, -1)?;
    Ok(String::from_utf8(stream.bytes().to_vec()).unwrap())
}

#[test]
fn identifiers_name_the_light_type() {
    let id =
        |kind| String::from_utf8(Graphic3dCLight::new(kind).get_id().as_bytes().to_vec()).unwrap();
    assert!(id(T::Ambient).starts_with("Graphic3d_CLight_amb"));
    assert!(id(T::Directional).starts_with("Graphic3d_CLight_dir"));
    assert!(id(T::Positional).starts_with("Graphic3d_CLight_pos"));
    assert!(id(T::Spot).starts_with("Graphic3d_CLight_spot"));
}

#[test]
fn dump_lists_the_type_fields() {
    let text = dump(&Graphic3dCLight::new(T::Positional)).unwrap();
    for key in [
        "\"className\": \"Graphic3d_CLight\"",
        "\"Position\":",
        "\"ConstAttenuation\": 1",
        "\"Range\": 0",
        "\"Type\": 2",
    ] {
        assert!(text.contains(key), "{key} missing from {text}");
    }
    assert!(!text.contains("Angle"), "{text}");
}

#[test]
fn dump_of_a_spot_without_direction_raises() {
    let spot = Graphic3dCLight::new(T::Spot);
    assert!(dump(&spot).is_err());
    spot.set_direction_coords(0.0, 0.0, -1.0).unwrap();
    assert!(dump(&spot).unwrap().contains("\"Angle\": 0.523599"));
}

#[test]
fn copy_from_null_raises() {
    let light = Graphic3dCLight::new(T::Ambient);
    let error = light.copy_from(&Handle::default()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    light
        .copy_from(&Handle::new(Graphic3dCLight::new(T::Ambient)))
        .unwrap();
}

#[test]
fn set_iterator_over_a_null_set_is_empty() {
    let it = Graphic3dLightSetIterator::from_handle(
        &Handle::<Graphic3dLightSet>::default(),
        Graphic3dLightSetIterationFilter::None,
    );
    assert!(!it.more());
    assert_eq!(
        it.value().err().unwrap().exception_type(),
        "Standard_NoSuchObject"
    );
    let set = Graphic3dLightSet::default();
    assert_eq!((set.lower(), set.upper(), set.is_empty()), (1, 0, true));
    assert_eq!(
        set.add(&Handle::default()).unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
}
