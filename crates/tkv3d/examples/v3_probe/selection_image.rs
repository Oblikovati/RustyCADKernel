//! Exact pixel bytes and exceptions from native selection image fillers and ToPixMap.
use super::{
    common::{Reader, attempt, end, flag, int},
    selector_traversal::traversal_scene,
};
#[allow(dead_code)]
#[path = "../../tests/support/selection_image_scene.rs"]
pub(super) mod image_scene;
use super::selector_pick::pick_scene;
use image_scene::{SelectionImageScene, image_bytes, selection_test_image};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkservice::{aspect::AspectNeutralWindow, image::ImagePixMap};
use rustycad_tkv3d::{select_mgr::*, std_select::StdSelectTypeOfSelectionImage};
fn image_state(image: &ImagePixMap) {
    let bytes = image_bytes(image);
    int(bytes.len() as i32);
    for byte in bytes {
        int(i32::from(byte));
    }
}
struct ImageRequest {
    kind: StdSelectTypeOfSelectionImage,
    format: i32,
    width: usize,
    height: usize,
    top: bool,
    padding: usize,
    mode: i32,
}
impl ImageRequest {
    fn read(r: &mut Reader<'_>) -> Self {
        Self {
            kind: StdSelectTypeOfSelectionImage(r.integer()),
            format: r.integer(),
            width: r.integer() as usize,
            height: r.integer() as usize,
            top: r.integer() != 0,
            padding: r.integer() as usize,
            mode: r.integer(),
        }
    }
    fn image(&self) -> ImagePixMap {
        selection_test_image(self.format, self.width, self.height, self.top, self.padding)
    }
}
pub fn filler_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let request = ImageRequest::read(&mut r);
    let fixture = SelectionImageScene::new(request.mode, -1);
    let image = request.image();
    let mut filler = probe_filler(&image, &fixture, request.kind);
    flag(filler.is_some());
    fill_requested_pixels(&mut r, &fixture, &mut filler, request.width as i32);
    if let Some(filler) = &mut filler {
        attempt(|| filler.flush());
    }
    image_state(&image);
    end();
}
fn probe_filler<'a>(
    image: &'a ImagePixMap,
    fixture: &'a SelectionImageScene,
    kind: StdSelectTypeOfSelectionImage,
) -> Option<Box<dyn SelectMgrSelectionImageFiller + 'a>> {
    let mut filler = None;
    attempt(|| {
        filler = select_mgr_create_selection_image_filler(
            image,
            fixture.scene.selector.get().unwrap(),
            kind,
        )?;
        Ok(())
    });
    filler
}
fn fill_requested_pixels(
    r: &mut Reader<'_>,
    fixture: &SelectionImageScene,
    filler: &mut Option<Box<dyn SelectMgrSelectionImageFiller + '_>>,
    width: i32,
) {
    let count = r.integer();
    for i in 0..count {
        let tag = r.integer();
        let rank = r.integer();
        let depth = r.real();
        let normal = Vec3::new(r.real() as f32, r.real() as f32, r.real() as f32);
        fixture.seed_pick(tag, depth, normal);
        if let Some(filler) = filler {
            attempt(|| filler.fill(i % width, i / width, rank));
        }
    }
}
pub fn pixmap_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let request = ImageRequest::read(&mut r);
    let picked = r.integer();
    let fixture = SelectionImageScene::new(request.mode, r.integer());
    let image = request.image();
    render_requested_image(&fixture, &image, &request, picked);
    image_state(&image);
    image_pick_state(&fixture);
    end();
}
fn render_requested_image(
    fixture: &SelectionImageScene,
    image: &ImagePixMap,
    request: &ImageRequest,
    picked: i32,
) {
    let (_viewer, view) =
        pick_scene::pick_view(traversal_scene::volume::TraversalConfig::default());
    view.window()
        .cast::<AspectNeutralWindow>()
        .set_size(request.width as i32, request.height as i32);
    fixture.scene.selector.set_pixel_tolerance(1);
    attempt(|| {
        flag(
            fixture
                .scene
                .selector
                .to_pix_map(image, &view, request.kind, picked)?,
        );
        Ok(())
    });
}
fn image_pick_state(fixture: &SelectionImageScene) {
    int(fixture.scene.selector.nb_picked());
    int(fixture
        .scene
        .selector
        .sorts
        .load(std::sync::atomic::Ordering::Acquire) as i32);
    let events = fixture.scene.events.read();
    int(events.len() as i32);
    for event in events.iter() {
        int(*event);
    }
}
