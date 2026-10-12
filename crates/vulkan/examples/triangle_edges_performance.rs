//! Native edge and mapped-line redraw/readback timings include actual retained live updates.
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/triangle_edge_scene.rs"]
mod edge_scene;
#[path = "support/retained_redraw_samples.rs"]
mod redraw_samples;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, RenderMeshEdgePipeline};
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
const STATIC_EDGE_WORKLOADS: &[&str] = &[
    "solid",
    "wide",
    "negative-alpha",
    "phong",
    "gouraud",
    "vertex",
    "bounds",
    "projective",
    "perspective",
    "clipped",
    "skip-first",
    "mask",
    "mask-blend",
    "blend",
    "dashed",
    "bounds-dashed",
    "polygon",
    "polygon-wide",
    "polygon-mask-blend",
    "line",
    "polyline",
    "line-phong",
    "line-mask-blend",
    "line-phong-mat-alpha-blend",
];
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for policy in [
        RenderMeshEdgePipeline::Programmable,
        RenderMeshEdgePipeline::PolygonCompatibility,
    ] {
        let mut options = device_options::validation_options();
        options.validation = false;
        let driver = Graphic3dRenderDriver::with_mesh_edge_pipeline(options, policy)?;
        println!("MARTENSITE_DEVICE={}", driver.device_name());
        static_edge_workloads(&driver, policy);
        if policy == RenderMeshEdgePipeline::Programmable {
            live_edge_workloads(&driver);
            cap_edge_workloads(&driver);
        }
    }
    Ok(())
}
fn static_edge_workloads(driver: &Handle<Graphic3dRenderDriver>, policy: RenderMeshEdgePipeline) {
    for mode in STATIC_EDGE_WORKLOADS {
        if mode.contains("polygon") != (policy == RenderMeshEdgePipeline::PolygonCompatibility) {
            continue;
        }
        let fixture = edge_scene::TriangleEdgeScene::new(driver, mode);
        let samples = redraw_samples::measure_redraws(&fixture.fixture.scene, |_| {});
        redraw_samples::report_samples(&format!("mesh-edge-{mode}"), samples);
        fixture.fixture.scene.view.remove();
    }
}
fn live_edge_workloads(driver: &Handle<Graphic3dRenderDriver>) {
    for update in [
        "width", "color", "enabled", "pattern", "alpha", "skip", "culling",
    ] {
        let fixture = edge_scene::TriangleEdgeScene::new(driver, "solid");
        let samples = redraw_samples::measure_redraws(&fixture.fixture.scene, |frame| {
            refresh_mesh_state(&fixture, update, frame)
        });
        redraw_samples::report_samples(&format!("mesh-edge-live-{update}"), samples);
        fixture.fixture.scene.view.remove();
    }
}
fn refresh_mesh_state(fixture: &edge_scene::TriangleEdgeScene, update: &str, frame: usize) {
    let aspect = &fixture.fixture.aspects;
    let alternate = frame.is_multiple_of(2);
    match update {
        "width" => aspect
            .set_edge_width(if alternate { 1. } else { 3. })
            .unwrap(),
        "enabled" => aspect.set_draw_edges(alternate),
        "skip" => aspect.set_skip_first_edge(alternate),
        "pattern" => aspect.set_line_type(if alternate {
            AspectTypeOfLine::Solid
        } else {
            AspectTypeOfLine::Dash
        }),
        "color" | "alpha" => refresh_edge_rgba(aspect, update, alternate),
        "culling" => aspect.set_face_culling(if alternate {
            Graphic3dTypeOfBackfacingModel::DoubleSided
        } else {
            Graphic3dTypeOfBackfacingModel::FrontCulled
        }),
        _ => panic!("received update={update:?}; expected supported mesh state update"),
    }
    fixture.fixture.scene.group.synchronize_aspects();
}
fn refresh_edge_rgba(aspect: &Graphic3dAspects, update: &str, alternate: bool) {
    let alpha = if update == "alpha" && alternate {
        0.25
    } else {
        1.
    };
    let red = if update == "color" && alternate {
        1.
    } else {
        0.
    };
    aspect.set_edge_color_rgba(ColorRgba::new(red, 1., 1., alpha).unwrap());
}
fn cap_edge_workloads(driver: &Handle<Graphic3dRenderDriver>) {
    for mode in [
        "cap-oblique-source-edges",
        "cap-oblique-source-edges-dashed",
        "cap-oblique-mapped-line-image-updated",
        "cap-oblique-mapped-line-image-updated-line-synchronized",
    ] {
        let fixture = capping_scene::CappingTextureScene::new(driver, mode);
        let samples = redraw_samples::measure_redraws(&fixture.scene, |_| fixture.mutate(mode));
        redraw_samples::report_samples(&format!("mesh-edge-{mode}"), samples);
        fixture.scene.view.remove();
    }
}

#[cfg(test)]
#[path = "../tests/support/recording_device.rs"]
mod recording_device;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mesh_live_updates_change_the_next_snapshot_without_mutating_prior_frames() {
        for update in [
            "width", "color", "enabled", "pattern", "alpha", "skip", "culling",
        ] {
            assert_mesh_update_snapshot(update);
        }
    }
    fn assert_mesh_update_snapshot(update: &str) {
        let recorder = recording_device::RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend(recorder.clone(), 128).unwrap();
        let fixture = edge_scene::TriangleEdgeScene::new(&driver, "solid");
        refresh_mesh_state(&fixture, update, 0);
        fixture.fixture.scene.render_view().try_redraw().unwrap();
        let before = recorder.last_frame();
        refresh_mesh_state(&fixture, update, 1);
        fixture.fixture.scene.render_view().try_redraw().unwrap();
        assert_ne!(before.draws(), recorder.last_frame().draws(), "{update}");
        assert_eq!(recorder.frames.read()[0].draws(), before.draws());
        fixture.fixture.scene.view.remove();
    }
    #[test]
    fn workload_names_are_unique_and_cover_shader_polygon_line_and_resource_updates() {
        let mut names = STATIC_EDGE_WORKLOADS.to_vec();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), STATIC_EDGE_WORKLOADS.len());
        for mode in [
            "solid",
            "polygon",
            "dashed",
            "line-phong",
            "mask-blend",
            "perspective",
        ] {
            assert!(names.contains(&mode));
        }
    }
}
