//! Capture and time actual Martensite light presentations on either Vulkan device.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/light_source_scene.rs"]
mod light_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};

use rustycad_tkv3d::{ais::*, v3d::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let kind = capture_light_kind(&args)?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let (viewer, view) = light_scene::light_view(&driver);
    let context = AisInteractiveContext::new(&viewer)?;
    context.display(&light_scene::rendered_light(kind)?.cast(), false)?;
    view_capture::capture_view(&view, 64, 64, &args[2], true)?;
    time_light_frames(&view);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    view.remove();
    Ok(())
}
fn time_light_frames(view: &V3dView) {
    let render = view.view().cast::<Graphic3dRenderView>();
    for _ in 0..20 {
        render.try_redraw().unwrap();
    }
    let samples: Vec<f64> = (0..100)
        .map(|_| {
            let start = std::time::Instant::now();
            render.try_redraw().unwrap();
            start.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    println!("{{\"warmup\":20,\"samples_ms\":{samples:?}}}");
    assert!(render.capture().is_ok());
}

fn capture_light_kind(args: &[String]) -> Result<usize, Box<dyn std::error::Error>> {
    if args.len() != 3 {
        return Err(format!(
            "received {} arguments; expected kind[0..3] prefix",
            args.len()
        )
        .into());
    }
    let kind = args[1]
        .parse::<usize>()
        .map_err(|_| format!("received kind={:?}; expected 0..3", args[1]))?;
    if kind > 3 {
        return Err(format!("received kind={kind}; expected 0..3").into());
    }
    Ok(kind)
}

#[cfg(test)]
mod tests {
    #[test]
    fn capture_kind_parser_accepts_all_lights_and_reports_invalid_shape() {
        for kind in 0..4 {
            assert_eq!(
                super::capture_light_kind(&["probe".into(), kind.to_string(), "prefix".into()])
                    .unwrap(),
                kind
            );
        }
        assert!(
            super::capture_light_kind(&[])
                .unwrap_err()
                .to_string()
                .contains("expected")
        );
        super::assert_invalid_light_kind_shapes();
    }
}

#[cfg(test)]
fn assert_invalid_light_kind_shapes() {
    for kind in ["4", "bad"] {
        assert!(
            capture_light_kind(&["probe".into(), kind.into(), "prefix".into()])
                .unwrap_err()
                .to_string()
                .contains(kind)
        );
    }
}
