//! Actual ColoredShape topology and PrsMgr presentations drive the rendering proof.
use super::scene::ViewerScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkbrep::topods::{Builder, Compound, Face, Shape};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::Plane3;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsMembers, Graphic3dStructureSource, Graphic3dTypeOfShadingModel,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dDrawer};

pub const COLORED_CAPTURE_MODES: [&str; 8] = [
    "wireframe",
    "shaded",
    "bbox",
    "custom-color",
    "hidden",
    "custom-width",
    "transparent",
    "updated",
];
pub struct AisColoredShapeScene {
    pub scene: ViewerScene,
    pub object: Handle<AisColoredShape>,
    pub manager: Handle<PrsMgrPresentationManager>,
    face: Face,
}
impl AisColoredShapeScene {
    /// Display genuine custom topology; `AisColoredShapeScene::with_mode(&driver,"hidden")?`.
    pub fn with_mode(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Result<Self, Failure> {
        colored_display_mode(mode)?;
        let scene = ViewerScene::new(driver);
        scene.structure.erase();
        let (shape, face) = colored_capture_topology()?;
        let object = AisColoredShape::new(&shape);
        configure_colored_capture(&object, &face, mode)?;
        let manager = PrsMgrPresentationManager::new(&scene.viewer.structure_manager());
        manager.display(&object.cast(), colored_display_mode(mode)?)?;
        Ok(Self {
            scene,
            object,
            manager,
            face,
        })
    }
    /// Recompute an already displayed native object; `fixture.mutate("updated")?`.
    pub fn mutate(&self, mode: &str) -> Result<(), Failure> {
        if mode != "updated" {
            return Ok(());
        }
        self.object
            .set_custom_color(&self.face, Color::named(NamedColor::GREEN))?;
        self.manager.update(&self.object.cast(), 1)?;
        Ok(())
    }
}
fn colored_capture_topology() -> Result<(Shape, Face), Failure> {
    let left = colored_capture_face((-4., 0., -3., 3.))?;
    let right = colored_capture_face((0., 4., -3., 3.))?;
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    Builder.add(&mut compound, &left)?;
    Builder.add(&mut compound, &right)?;
    BRepMeshIncrementalMesh::with_shape(&compound, 0.1, false, 0.5, false)?;
    Ok((compound.into_shape(), left))
}
fn colored_capture_face(bounds: (f64, f64, f64, f64)) -> Result<Face, Failure> {
    // Native MakeFace builds the same pcurves and boundary topology before actual meshing.
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), bounds)?.face()?;
    Ok(face)
}
fn configure_colored_capture(
    object: &AisColoredShape,
    face: &Shape,
    mode: &str,
) -> Result<(), Failure> {
    let defaults = Handle::new(Prs3dDrawer::new());
    defaults.setup_own_defaults()?;
    let drawer = object.attributes();
    drawer.set_link(&defaults);
    drawer.setup_own_defaults()?;
    drawer.set_auto_triangulation(false);
    drawer.u_iso_aspect().set_number(0);
    drawer.v_iso_aspect().set_number(0);
    object.set_color(Color::named(NamedColor::WHITE))?;
    set_colored_capture_unlit(&drawer);
    configure_colored_custom(object, face, mode)
}
fn set_colored_capture_unlit(drawer: &Prs3dDrawer) {
    // Both native and Rust captures disable lighting, isolating per-subshape appearance.
    let aspect = drawer.shading_aspect().aspect();
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
}
fn configure_colored_custom(
    object: &AisColoredShape,
    face: &Shape,
    mode: &str,
) -> Result<(), Failure> {
    match mode {
        "custom-color" | "updated" => {
            object.set_custom_color(face, Color::named(NamedColor::RED))?
        }
        "hidden" => object.custom_aspects(face)?.set_hidden(true),
        "custom-width" => object.set_custom_width(face, 3.)?,
        "transparent" => object.set_custom_transparency(face, 0.5)?,
        _ => (),
    }
    Ok(())
}
fn colored_display_mode(mode: &str) -> Result<i32, Failure> {
    if !COLORED_CAPTURE_MODES.contains(&mode) {
        return Err(Failure::new(&format!(
            "received mode={mode:?}; expected {COLORED_CAPTURE_MODES:?}"
        )));
    }
    Ok(match mode {
        "wireframe" | "custom-width" => 0,
        "bbox" => 2,
        _ => 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_colored_topology_retains_two_faces_and_native_modes() {
        let (shape, face) = colored_capture_topology().unwrap();
        assert!(!shape.is_null());
        assert!(!face.is_null());
        for mode in COLORED_CAPTURE_MODES {
            let object = AisColoredShape::new(&shape);
            configure_colored_capture(&object, &face, mode).unwrap();
            assert!((0..=2).contains(&colored_display_mode(mode).unwrap()));
        }
        assert!(
            colored_display_mode("invalid")
                .unwrap_err()
                .to_string()
                .contains("invalid")
        );
    }
}
