//! Selection images preserve float normalization, palette order and native failures.
#[path = "support/selection_image_scene.rs"]
mod image_scene;
#[allow(dead_code)]
#[path = "support/pick_scene.rs"]
mod pick_scene;
#[path = "support/traversal_scene.rs"]
mod traversal_scene;
use image_scene::*;
use rustycad_tkernel::{ncollection::Vec3, quantity::Color};
use rustycad_tkv3d::{select_mgr::*, std_select::StdSelectTypeOfSelectionImage as Kind};
#[test]
fn selection_image_modes_keep_native_numeric_values_and_unknowns() {
    let kinds = [
        Kind::NORMALIZED_DEPTH,
        Kind::NORMALIZED_DEPTH_INVERTED,
        Kind::UNNORMALIZED_DEPTH,
        Kind::COLORED_DETECTED_OBJECT,
        Kind::COLORED_ENTITY,
        Kind::COLORED_ENTITY_TYPE,
        Kind::COLORED_OWNER,
        Kind::COLORED_SELECTION_MODE,
        Kind::SURFACE_NORMAL,
    ];
    assert_eq!(kinds.map(|k| k.0), [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    let scene = SelectionImageScene::new(9, -1);
    let image = selection_test_image(14, 1, 1, false, 0);
    assert!(
        select_mgr_create_selection_image_filler(
            &image,
            scene.scene.selector.get().unwrap(),
            Kind(9)
        )
        .unwrap()
        .is_none()
    );
}
#[test]
fn normalized_depth_is_deferred_and_missing_hits_stay_black_when_inverted() {
    let scene = SelectionImageScene::new(9, -1);
    let image = selection_test_image(14, 3, 1, false, 0);
    let before = image_bytes(&image);
    let mut filler = test_filler(&image, &scene, Kind::NORMALIZED_DEPTH_INVERTED);
    for (col, tag, depth) in [(0, 0, 2.), (1, 1, 4.), (2, -1, 0.)] {
        scene.seed_pick(tag, depth, Vec3::default());
        filler.fill(col, 0, 1).unwrap();
    }
    assert_eq!(image_bytes(&image), before);
    filler.flush().unwrap();
    assert_eq!(image_pixel(&image, 0, 0).unwrap(), [1., 1., 1., 1.]);
    assert_eq!(image_pixel(&image, 1, 0).unwrap(), [0., 0., 0., 1.]);
    assert_eq!(image_pixel(&image, 2, 0).unwrap(), [0., 0., 0., 1.]);
}
#[test]
fn raw_depth_outside_rgb_range_preserves_the_previous_pixel() {
    let scene = SelectionImageScene::new(9, -1);
    let image = selection_test_image(14, 1, 1, false, 0);
    let before = image_bytes(&image);
    let mut filler = test_filler(&image, &scene, Kind::UNNORMALIZED_DEPTH);
    scene.seed_pick(0, 2., Vec3::default());
    assert!(filler.fill(0, 0, 1).is_err());
    assert_eq!(image_bytes(&image), before);
}
#[test]
fn normal_filler_uses_float_normalization_and_black_for_zero() {
    let scene = SelectionImageScene::new(9, -1);
    let image = selection_test_image(14, 2, 1, false, 0);
    let mut filler = test_filler(&image, &scene, Kind::SURFACE_NORMAL);
    scene.seed_pick(0, 0., Vec3::new(2., 0., 0.));
    filler.fill(0, 0, 1).unwrap();
    scene.seed_pick(1, 0., Vec3::default());
    filler.fill(1, 0, 1).unwrap();
    assert_eq!(image_pixel(&image, 0, 0).unwrap(), [1., 0.5, 0.5, 1.]);
    assert_eq!(image_pixel(&image, 1, 0).unwrap(), [0., 0., 0., 1.]);
}
#[test]
fn color_generator_is_repeatable_and_unique_calls_do_not_repeat() {
    let scene = SelectionImageScene::new(9, -1);
    let image = selection_test_image(14, 1, 1, false, 0);
    let mut first =
        SelectMgrSelectionImageFillerBase::new(&image, scene.scene.selector.get().unwrap());
    let mut second =
        SelectMgrSelectionImageFillerBase::new(&image, scene.scene.selector.get().unwrap());
    let initial = first.next_random_pastel_color().unwrap();
    assert_eq!(initial, second.next_random_pastel_color().unwrap());
    let mut colors: Vec<Color> = vec![];
    for _ in 0..64 {
        let color = first.random_pastel_color().unwrap();
        assert!(!colors.contains(&color));
        colors.push(color);
    }
}
#[test]
fn empty_image_error_precedes_unknown_kind_and_valid_unknown_does_not_pick() {
    let scene = SelectionImageScene::new(9, -1);
    let (_viewer, view) = pick_scene::pick_view(Default::default());
    let empty = selection_test_image(14, 0, 0, false, 0);
    assert!(
        scene
            .scene
            .selector
            .to_pix_map(&empty, &view, Kind(9), 1)
            .is_err()
    );
    let image = selection_test_image(14, 1, 1, false, 0);
    let before = image_bytes(&image);
    assert!(
        !scene
            .scene
            .selector
            .to_pix_map(&image, &view, Kind(9), 1)
            .unwrap()
    );
    assert_eq!(image_bytes(&image), before);
    assert!(scene.scene.events.read().is_empty());
}

fn palette_pixels(kind: Kind, mode: i32, tags: &[i32]) -> Vec<[f32; 4]> {
    let scene = SelectionImageScene::new(mode, -1);
    let image = selection_test_image(14, tags.len(), 1, false, 0);
    let mut filler = test_filler(&image, &scene, kind);
    for (col, tag) in tags.iter().enumerate() {
        scene.seed_pick(*tag, 0.5, Vec3::default());
        filler.fill(col as i32, 0, 1).unwrap();
    }
    filler.flush().unwrap();
    (0..tags.len())
        .map(|col| image_pixel(&image, col as i32, 0).unwrap())
        .collect()
}
#[test]
fn palettes_distinguish_entity_owner_and_runtime_type_with_duplicate_memberships() {
    let tags = [0, 1, 3, 9, 0];
    let entity = palette_pixels(Kind::COLORED_ENTITY, 9, &tags);
    let owner = palette_pixels(Kind::COLORED_OWNER, 9, &tags);
    let types = palette_pixels(Kind::COLORED_ENTITY_TYPE, 9, &tags);
    assert_eq!(entity[0], entity[4]);
    assert_ne!(entity[0], entity[1]);
    assert_eq!(owner[0], owner[1]);
    assert_eq!(owner[0], owner[3]);
    assert_ne!(owner[0], owner[2]);
    assert_eq!(types[0], types[2]);
    assert_ne!(types[0], types[3]);
}
#[test]
fn last_membership_sets_mode_and_missing_palette_entries_are_black() {
    let black = [0., 0., 0., 1.];
    let mode = palette_pixels(Kind::COLORED_SELECTION_MODE, 3, &[0, 1, 9]);
    assert_eq!(
        mode,
        vec![[1., 0., 0., 1.], [1., 1., 1., 1.], [1., 0., 0., 1.]]
    );
    assert_eq!(
        palette_pixels(Kind::COLORED_SELECTION_MODE, -1, &[0]),
        [black]
    );
    assert_eq!(palette_pixels(Kind::COLORED_ENTITY, 0, &[1]), [black]);
    let custom = palette_pixels(Kind::COLORED_SELECTION_MODE, 9, &[0, 3, 9]);
    assert_eq!(custom[0], custom[1]);
    assert_eq!(custom[0], custom[2]);
    assert_ne!(custom[0], black);
    assert_eq!(
        palette_pixels(Kind::COLORED_DETECTED_OBJECT, 9, &[0, 3, 6]),
        vec![[1., 0., 0., 1.], [0., 1., 0., 1.], [0., 0., 1., 1.]]
    );
}
#[test]
fn unfilled_inverted_depth_flushes_white_but_extreme_hit_sentinels_stay_black() {
    let scene = SelectionImageScene::new(9, -1);
    let image = selection_test_image(14, 3, 1, false, 0);
    let mut filler = test_filler(&image, &scene, Kind::NORMALIZED_DEPTH_INVERTED);
    for (col, depth) in [(0, -f64::from(f32::MAX)), (1, f64::from(f32::MAX))] {
        scene.seed_pick(0, depth, Vec3::default());
        filler.fill(col, 0, 1).unwrap();
    }
    filler.flush().unwrap();
    assert_eq!(image_pixel(&image, 0, 0).unwrap(), [0., 0., 0., 1.]);
    assert_eq!(image_pixel(&image, 1, 0).unwrap(), [0., 0., 0., 1.]);
    assert_eq!(image_pixel(&image, 2, 0).unwrap(), [1., 1., 1., 1.]);
}
#[test]
fn public_pixmap_completes_all_modes_and_preserves_unflushed_depth_on_pick_failure() {
    for kind in 0..9 {
        assert_public_image_result(Kind(kind), -1, true);
    }
    assert_public_image_result(Kind::NORMALIZED_DEPTH, 0, false);
}
fn assert_public_image_result(kind: Kind, failing: i32, success: bool) {
    use rustycad_tkservice::aspect::AspectNeutralWindow;
    let scene = SelectionImageScene::new(9, failing);
    let (_viewer, view) = pick_scene::pick_view(Default::default());
    view.window().cast::<AspectNeutralWindow>().set_size(7, 5);
    scene.scene.selector.set_pixel_tolerance(1);
    let image = selection_test_image(14, 7, 5, false, 0);
    let before = image_bytes(&image);
    let result = scene.scene.selector.to_pix_map(&image, &view, kind, 1);
    // Raw picked depth can exceed RGB range; native ToPixMap propagates it.
    if !success || kind == Kind::UNNORMALIZED_DEPTH {
        assert!(result.is_err());
    } else {
        assert!(result.unwrap());
    }
    if !success {
        assert_eq!(image_bytes(&image), before);
    }
    assert!(!scene.scene.events.read().is_empty());
}
struct DefaultFlushFiller;
impl SelectMgrSelectionImageFiller for DefaultFlushFiller {
    fn fill(&mut self, _: i32, _: i32, _: i32) -> Result<(), rustycad_tkernel::standard::Failure> {
        Ok(())
    }
}
#[test]
fn custom_filler_default_flush_and_protected_state_are_available() {
    let mut filler = DefaultFlushFiller;
    filler.fill(0, 0, 1).unwrap();
    filler.flush().unwrap();
    let scene = SelectionImageScene::new(9, -1);
    let image = selection_test_image(14, 1, 1, false, 0);
    let mut base =
        SelectMgrSelectionImageFillerBase::new(&image, scene.scene.selector.get().unwrap());
    let color = base.next_random_pastel_color().unwrap();
    assert!(base.unique_colors.add(color));
    assert!(!base.unique_colors.add(color));
    let _ = base.generator.next_int();
    assert_eq!(base.image.width(), 1);
    assert_eq!(base.selector.nb_picked(), 0);
}

fn test_filler<'a>(
    image: &'a rustycad_tkservice::image::ImagePixMap,
    scene: &'a SelectionImageScene,
    kind: Kind,
) -> Box<dyn SelectMgrSelectionImageFiller + 'a> {
    select_mgr_create_selection_image_filler(image, scene.scene.selector.get().unwrap(), kind)
        .unwrap()
        .unwrap()
}
