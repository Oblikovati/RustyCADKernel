//! Actual clipping planes and perspective foreshortening exercise surface texture interpolation.
use super::super::scene::ViewerScene;
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
pub fn configure_surface_view(
    scene: &ViewerScene,
    vertices: &Graphic3dArrayOfPrimitives,
    mode: &str,
) {
    if mode == "perspective" {
        scene
            .view
            .camera()
            .set_projection_type(Graphic3dCameraProjection::Perspective);
        for (index, z) in [-4., -4., 4., 4.].into_iter().enumerate() {
            let point = vertices.vertice(index as i32 + 1).unwrap();
            vertices
                .set_vertice(index as i32 + 1, point.x() as f32, point.y() as f32, z)
                .unwrap();
        }
    }
    if mode == "clipped" {
        let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        planes.append(Handle::new(
            Graphic3dClipPlane::from_equation(Vec4::new(1., 0., 0., 0.)).unwrap(),
        ));
        scene.cview().set_clip_planes(&planes);
    }
}
