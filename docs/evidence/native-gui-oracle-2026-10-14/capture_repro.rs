// SPDX-License-Identifier: GPL-2.0-only
//! Disposable consumer reproducer: GUI replacement plus one model triangle.
use oblikovati_viewport::{Viewport, ViewportRuntime, ViewportScene, ViewportTriangle};
use std::time::{Duration, Instant};
fn verify(window: &mut Viewport, label: &str) {
    let capture = window.redraw().unwrap();
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        let actual = window.native_pixels().unwrap();
        if actual == capture.rgba { break; }
        if Instant::now() >= until {
            let count = actual.chunks_exact(4).zip(capture.rgba.chunks_exact(4)).filter(|(a,b)| a != b).count();
            std::fs::write(format!("/tmp/native-gui-{label}-expected.rgba"), &capture.rgba).unwrap();
            std::fs::write(format!("/tmp/native-gui-{label}-actual.rgba"), &actual).unwrap();
            if std::env::var_os("MARTENSITE_DEBUG_HOLD").is_some() { std::thread::sleep(Duration::from_secs(45)); }
            panic!("{label}: {count} native pixels differ from GPU capture, extent {:?}", capture.extent);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    println!("native equality: {label}");
}
fn main() {
    let empty = std::env::args().any(|arg| arg == "--empty");
    let runtime = ViewportRuntime::new(Default::default()).unwrap();
    let mut window = runtime.open_window("Oblikovati GUI reproduction", [1440,900]).unwrap();
    let triangles = if empty { vec![] } else { vec![ViewportTriangle { positions: [[0.,0.,0.],[5.,0.,0.],[0.,3.,0.]], color: [0.,1.,0.,1.] }] };
    window.submit(&ViewportScene { background: [0.,0.,1.], triangles: triangles.clone() }).unwrap();
    verify(&mut window, "model before GUI");
    window.submit(&ViewportScene { background: [0.,1.,0.], triangles }).unwrap();
    verify(&mut window, "replaced model before GUI");
    let mut gui = oblikovati_gui::GuiFrameEngine::new().unwrap();
    let compiler = oblikovati_viewport::GuiFrameCompiler::new(gui.atlas()).unwrap();
    for index in 0..3 {
        let request = oblikovati_gui::GuiFrameRequest {
            extent: window.drawable_extent().unwrap(), events: vec![],
            commands: Default::default(), status: Some(format!("Frame {index}")),
            delta_seconds: 1./60., framebuffer_scale: [1.;2],
        };
        let packet = gui.frame(&request).unwrap();
        window.submit_gui_frame(&compiler, &packet.draws).unwrap();
    }
    for _ in 0..3 { window.redraw().unwrap(); }
    verify(&mut window, "GUI replacements before drawing");
    window.close();
    assert!(runtime.validation_errors().is_empty());
}
