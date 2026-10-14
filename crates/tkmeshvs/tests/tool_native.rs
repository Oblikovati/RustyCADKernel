//! Full seven-operation native replay without compiling or invoking OCCT.
mod corpus;
mod normals;
mod snapshot;
use martensite_tkmeshvs::{MeshVSDrawer, MeshVSTool as T};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsSource, Graphic3dMaterialAspect, Graphic3dNameOfMaterial,
};
use std::fmt::Write;
type AspectOwner = Handle<dyn Graphic3dAspectsSource>;
#[test]
fn full_native_meshvs_tool_factories_raw_values_failures_and_normals_replay() {
    let mut rows = String::new();
    corpus::defaults(&mut rows);
    corpus::missing(&mut rows);
    corpus::raw_enums(&mut rows);
    corpus::failures(&mut rows);
    corpus::font_bytes(&mut rows);
    corpus::early_exit(&mut rows);
    normals::replay(&mut rows);
    assert_eq!(rows, include_str!("fixtures/tool-native.txt"));
}
fn build(kind: i32, drawer: &Handle<MeshVSDrawer>, defaults: bool) -> Result<AspectOwner, Failure> {
    let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold);
    material.set_transparency(0.375)?;
    Ok(match kind {
        0 => T::create_aspect_fill_area3d(drawer, defaults)?.into_cast(),
        1 => T::create_aspect_fill_area3d_with_material(drawer, &material, defaults)?.into_cast(),
        2 => T::create_aspect_line3d(drawer, defaults)?.into_cast(),
        3 => T::create_aspect_marker3d(drawer, defaults)?.into_cast(),
        _ => T::create_aspect_text3d(drawer, defaults)?.into_cast(),
    })
}
fn row(
    rows: &mut String,
    label: &str,
    kind: i32,
    index: i32,
    drawer: &Handle<MeshVSDrawer>,
    defaults: bool,
) {
    write!(rows, "{label} {kind} {index}").unwrap();
    match build(kind, drawer, defaults) {
        Ok(value) => snapshot::aspect(rows, kind, &value),
        Err(cause) => write!(rows, " failure {}", cause.exception_type()).unwrap(),
    }
    writeln!(rows, " {}", drawer.ref_count()).unwrap();
}
