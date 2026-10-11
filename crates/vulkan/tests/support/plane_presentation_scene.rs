#![allow(dead_code)]
//! Exercise real plane presentation builders through the production retained renderer.
use crate::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{geom::GeomPlane, geom_adaptor::GeomAdaptorSurface};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkservice::{aspect::AspectNeutralWindow, graphic3d::*};
use rustycad_tkv3d::{dsg_prs::*, prs3d::*, std_prs::StdPrsPlane, v3d::*};
pub const MODES: [&str; 6] = [
    "prsplane-edges",
    "prsplane-isos",
    "prsplane-center",
    "prsplane-corners",
    "prsplane-datum",
    "prsplane-shaded",
];
pub struct PlanePresentationScene {
    // A V3dView borrows its viewer; retain it until captures and rebuilds finish.
    pub viewer: Handle<V3dViewer>,
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub structure: Handle<Graphic3dStructure>,
    pub drawer: Handle<Prs3dDrawer>,
    pub plane: GeomAdaptorSurface,
}
impl PlanePresentationScene {
    /// Retain a live plane scene; `PlanePresentationScene::new(&driver, "prsplane-isos", 0.)`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str, offset: f64) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let structure =
            Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
        let drawer = plane_drawer(mode);
        let plane = capture_plane(offset);
        let scene = Self {
            viewer,
            view,
            window,
            structure,
            drawer,
            plane,
        };
        scene.rebuild(mode);
        scene.structure.display();
        scene
    }
    /// Access redraw/readback; `scene.render_view().try_redraw()?`.
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
    /// Rebuild from live aspects; `scene.rebuild("prsplane-corners")`.
    pub fn rebuild(&self, mode: &str) {
        self.structure.clear(false);
        let points = [
            Point3::new(-6., -4., 0.),
            Point3::new(6., -4., 0.),
            Point3::new(0., 6., 0.),
        ];
        match mode {
            "prsplane-datum" => DsgPrsXYZPlanePresentation::add(
                &self.structure.cast(),
                &self.drawer,
                &points[0],
                &points[1],
                &points[2],
            )
            .unwrap(),
            "prsplane-shaded" => DsgPrsShadedPlanePresentation::add(
                &self.structure.cast(),
                &self.drawer,
                &points[0],
                &points[1],
                &points[2],
            )
            .unwrap(),
            _ => StdPrsPlane::add(&self.structure.cast(), &self.plane, &self.drawer).unwrap(),
        }
    }
}
fn plane_drawer(mode: &str) -> Handle<Prs3dDrawer> {
    let drawer = Handle::new(Prs3dDrawer::new());
    let aspect = Handle::new(Prs3dPlaneAspect::new().unwrap());
    aspect.set_plane_length(12., 10.);
    aspect.set_iso_distance(2.);
    aspect.set_display_iso(mode == "prsplane-isos");
    aspect.set_display_center_arrow(mode == "prsplane-center");
    aspect.set_display_edges_arrows(mode == "prsplane-corners");
    aspect.set_arrows_size(3.);
    aspect.set_arrows_length(1.);
    drawer.set_plane_aspect(aspect);
    let shading = Handle::new(Prs3dShadingAspect::new().unwrap());
    shading
        .aspect()
        .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    shading.aspect().set_polygon_offsets(
        rustycad_tkservice::aspect::AspectPolygonOffsetMode::OFF.0,
        0.,
        0.,
    );
    drawer.set_shading_aspect(shading);
    drawer
}

fn capture_plane(offset: f64) -> GeomAdaptorSurface {
    let frame = Axis3::new(
        Point3::new(offset, 0., 0.),
        Direction3::new(0., 0.6, 0.8).unwrap(),
        Direction3::X,
    )
    .unwrap();
    GeomAdaptorSurface::with_surface(&Handle::new(GeomPlane::new(frame)).cast()).unwrap()
}
