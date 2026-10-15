//! Translation of OCCT's GTests/StdPrs_BRepFont_Test.cxx at 3d097a0. Font-dependent
//! cases skip, as natively, when FreeType and usable fonts are unavailable.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkservice::font::{FontFontAspect, FontStrictLevel};
use rustycad_tkservice::graphic3d::{
    Graphic3dHorizontalTextAlignment as Hta, Graphic3dVerticalTextAlignment as Vta,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tkv3d::std_prs::{
    StdPrsBRepFont, StdPrsBRepFontCache, StdPrsBRepFontGlyphOptions, StdPrsBRepFontTextOptions,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const FALLBACK_REQUEST: &str = "OCCT-Embedded-Fallback-Request-4A72C3";

fn text(value: &str) -> String8 {
    String8::from_unicode_units(value.as_bytes(), -1)
}

fn create_font() -> Handle<StdPrsBRepFont> {
    StdPrsBRepFont::find_and_create(
        &FALLBACK_REQUEST.into(),
        FontFontAspect::Regular,
        12.0,
        FontStrictLevel::Any,
    )
}

fn count_sub_shapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut found = 0;
    while explorer.more() {
        found += 1;
        explorer.next().unwrap();
    }
    found
}

fn is_valid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

fn skip() {
    eprintln!("SKIPPED: FreeType and usable fonts are unavailable");
}

const A: u32 = 'A' as u32;
const B: u32 = 'B' as u32;
const I: u32 = 'I' as u32;

#[test]
fn invalid_and_released_font_return_empty_results() {
    let font = StdPrsBRepFont::new();
    assert!(!font.is_valid());
    assert_eq!(0.0, font.size());
    assert_eq!(0.0, font.ascender());
    assert!(font.render_glyph(A).is_null());
    assert!(font.render_text(&text("A")).is_null());
    let initialized = create_font();
    let Some(initialized) = initialized.get() else {
        return skip();
    };
    initialized.release();
    assert!(!initialized.is_valid());
    assert!(initialized.render_glyph(A).is_null());
}

#[test]
fn glyph_options_concatenate_contours_per_call() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let default_shape = font.render_glyph(B);
    assert!(!default_shape.is_null());
    let options = StdPrsBRepFontGlyphOptions {
        to_concatenate_contours: true,
    };
    let composite = font.render_glyph_with(B, &options);
    assert!(!composite.is_null());
    assert!(is_valid(&composite));
    assert!(
        count_sub_shapes(&composite, ShapeEnum::Edge)
            < count_sub_shapes(&default_shape, ShapeEnum::Edge)
    );
    assert_eq!(
        count_sub_shapes(&default_shape, ShapeEnum::Face),
        count_sub_shapes(&composite, ShapeEnum::Face)
    );
}

#[test]
fn render_glyph_returns_independent_topology() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let default_shape = font.render_glyph(B);
    assert!(!default_shape.is_null());
    let second_default = font.render_glyph(B);
    assert!(!second_default.is_null());
    assert!(!default_shape.is_same(&second_default));
    let options = StdPrsBRepFontGlyphOptions {
        to_concatenate_contours: true,
    };
    let composite = font.render_glyph_with(B, &options);
    assert!(!composite.is_null());
    let second_composite = font.render_glyph_with(B, &options);
    assert!(!second_composite.is_null());
    assert!(!composite.is_same(&second_composite));
    assert!(!default_shape.is_same(&composite));
}

#[test]
fn render_text_builds_placed_compound() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let default_text = font.render_text(&text("ABBA"));
    assert!(!default_text.is_null());
    assert!(is_valid(&default_text));
    let options = StdPrsBRepFontTextOptions {
        horizontal_alignment: Hta::Center,
        vertical_alignment: Vta::Center,
        to_concatenate_contours: true,
        ..StdPrsBRepFontTextOptions::default()
    };
    let configured = font.render_text_with(&text("ABBA"), &options);
    assert!(!configured.is_null());
    assert!(is_valid(&configured));
}

#[test]
fn render_text_shares_only_per_call_glyph_topology() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let glyph = font.render_glyph(A);
    assert!(!glyph.is_null());
    let rendered = font.render_text(&text("AA"));
    assert!(!rendered.is_null());
    let mut first_occurrence = Shape::default();
    let mut occurrences = 0;
    let mut iterator = ShapeIterator::new(&rendered, true, true);
    while iterator.more() {
        let value = iterator.value().unwrap().clone();
        assert!(!glyph.is_partner(&value));
        if first_occurrence.is_null() {
            first_occurrence = value;
        } else {
            assert!(first_occurrence.is_partner(&value));
        }
        occurrences += 1;
        iterator.next();
    }
    assert_eq!(2, occurrences);
}

#[test]
fn text_plan_stores_bounds_and_regions() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let plan = font
        .plan_text(&text("ABBA"), Hta::Center, Vta::Center)
        .unwrap();
    assert_eq!(2, plan.nb_regions());
    assert_eq!(4, plan.items().len());
    assert!(plan.width() > 0.0);
    assert!(plan.height() > 0.0);
    assert!(plan.lower_left().x() < 0.0);
    assert!(plan.upper_right().x() > 0.0);
    let rendered = font.render_plan(&plan);
    assert!(!rendered.is_null());
    assert_eq!(4, rendered.nb_children());
}

#[test]
fn right_aligned_multiline_text_builds_shape() {
    let font = StdPrsBRepFont::find_and_create(
        &"Times-Roman".into(),
        FontFontAspect::Regular,
        50.0,
        FontStrictLevel::Any,
    );
    let Some(font) = font.get() else {
        return skip();
    };
    let plan = font
        .plan_text(
            &text("Top-Right\nFirst line\nLion The Second\n3rd"),
            Hta::Right,
            Vta::Top,
        )
        .unwrap();
    assert!(plan.nb_regions() > 0);
    assert!(!plan.items().is_empty());
    assert!(plan.lower_left().x() < plan.upper_right().x());
    assert!(plan.lower_left().y() < plan.upper_right().y());
    assert!(!font.render_plan(&plan).is_null());
}

#[test]
fn text_plan_remains_valid_after_font_changes() {
    let (font, other_font) = (create_font(), create_font());
    let (Some(font), Some(other_font)) = (font.get(), other_font.get()) else {
        return skip();
    };
    let plan = font.plan_text(&text("B"), Hta::Left, Vta::Bottom).unwrap();
    let original = font.render_plan(&plan);
    assert!(!original.is_null());
    assert!(count_sub_shapes(&original, ShapeEnum::Face) > 0);
    assert!(!other_font.render_plan(&plan).is_null());
    assert!(font.set_width_scaling(1.25));
    let from_old_plan = font.render_plan(&plan);
    assert!(!from_old_plan.is_null());
    assert!(count_sub_shapes(&from_old_plan, ShapeEnum::Face) > 0);
    let scaled_plan = font.plan_text(&text("B"), Hta::Left, Vta::Bottom).unwrap();
    assert!(!font.render_plan(&scaled_plan).is_null());
    font.set_single_stroke_font(true);
    let from_filled_plan = font.render_plan(&scaled_plan);
    assert!(!from_filled_plan.is_null());
    assert!(count_sub_shapes(&from_filled_plan, ShapeEnum::Face) > 0);
    let stroke_plan = font.plan_text(&text("B"), Hta::Left, Vta::Bottom).unwrap();
    let stroke_text = font.render_plan(&stroke_plan);
    assert!(!stroke_text.is_null());
    assert_eq!(0, count_sub_shapes(&stroke_text, ShapeEnum::Face));
    assert!(count_sub_shapes(&stroke_text, ShapeEnum::Wire) > 0);
}

#[test]
fn invisible_text_retains_layout_without_building_topology() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let plan = font.plan_text(&text("  "), Hta::Left, Vta::Bottom).unwrap();
    assert!(plan.items().is_empty());
    assert_eq!(0, plan.nb_regions());
    assert!(plan.width() > 0.0);
    assert!(font.render_plan(&plan).is_null());
}

#[test]
fn width_scaling_validation_preserves_usable_font() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let before = font.render_glyph(I);
    assert!(!before.is_null());
    assert!(!font.set_width_scaling(0.0));
    assert!(!font.set_width_scaling(-1.0));
    assert!(!font.set_width_scaling(f32::INFINITY));
    assert!(!font.render_glyph(I).is_null());
    assert!(font.set_width_scaling(1.25));
    let after = font.render_glyph(I);
    assert!(!after.is_null());
    assert!(!before.is_same(&after));
}

#[test]
fn cache_distinguishes_font_parameters() {
    let cache = StdPrsBRepFontCache::new(4);
    let name = FALLBACK_REQUEST.into();
    let font = cache.find_font(&name, FontFontAspect::Regular, 12.0, FontStrictLevel::Any);
    if font.is_null() {
        return skip();
    }
    let same_font = cache.find_font(&name, FontFontAspect::Regular, 12.0, FontStrictLevel::Any);
    assert!(!same_font.is_null());
    assert_ne!(font, same_font);
    let (font_ref, same_ref) = (font.get().unwrap(), same_font.get().unwrap());
    let (glyph, same_glyph) = (font_ref.render_glyph(A), same_ref.render_glyph(A));
    assert!(!glyph.is_null() && !same_glyph.is_null());
    assert!(!glyph.is_partner(&same_glyph));
    let plan = font_ref
        .plan_text(&text("A"), Hta::Left, Vta::Bottom)
        .unwrap();
    let same_plan = same_ref
        .plan_text(&text("A"), Hta::Left, Vta::Bottom)
        .unwrap();
    assert_eq!(1, plan.nb_regions());
    assert_eq!(1, same_plan.nb_regions());
    assert!(Arc::ptr_eq(
        plan.region_handle(0),
        same_plan.region_handle(0)
    ));
    let next_size = f64::from_bits(12.0f64.to_bits() + 1);
    let next_font = cache.find_font(
        &name,
        FontFontAspect::Regular,
        next_size,
        FontStrictLevel::Any,
    );
    assert!(!next_font.is_null());
    assert_ne!(font, next_font);
    assert_eq!(2, cache.size());
    assert!(font_ref.set_width_scaling(1.25));
    let cached = cache.find_font(&name, FontFontAspect::Regular, 12.0, FontStrictLevel::Any);
    assert!(!cached.is_null());
    assert_ne!(font, cached);
    let cached_plan = cached
        .get()
        .unwrap()
        .plan_text(&text("AB"), Hta::Left, Vta::Bottom)
        .unwrap();
    let changed_plan = font_ref
        .plan_text(&text("AB"), Hta::Left, Vta::Bottom)
        .unwrap();
    assert!(cached_plan.width() < changed_plan.width());
    assert_eq!(2, cache.size());
}

#[test]
fn caches_are_independent_and_respect_capacity() {
    let (first_cache, second_cache) = (StdPrsBRepFontCache::new(2), StdPrsBRepFontCache::new(2));
    let name = FALLBACK_REQUEST.into();
    let find = |cache: &StdPrsBRepFontCache, size: f64| {
        cache.find_font(&name, FontFontAspect::Regular, size, FontStrictLevel::Any)
    };
    let first = find(&first_cache, 12.0);
    if first.is_null() {
        return skip();
    }
    let first_again = find(&first_cache, 12.0);
    assert!(!first_again.is_null());
    assert_ne!(first, first_again);
    let other_size = find(&first_cache, 13.0);
    assert!(!other_size.is_null());
    assert_ne!(first, other_size);
    assert_eq!(2, first_cache.size());
    assert!(!find(&first_cache, 14.0).is_null());
    assert_eq!(2, first_cache.size());
    let evicted = find(&first_cache, 12.0);
    assert!(!evicted.is_null());
    assert_ne!(first, evicted);
    let independent = find(&second_cache, 12.0);
    assert!(!independent.is_null());
    assert_ne!(first, independent);
    first_cache.clear();
    assert_eq!(0, first_cache.size());
    assert!(first.get().unwrap().is_valid());
    let recreated = find(&first_cache, 12.0);
    assert!(!recreated.is_null());
    assert_ne!(first, recreated);
    recreated.get().unwrap().release();
    let still_cached = find(&first_cache, 12.0);
    assert!(!still_cached.is_null());
    assert!(still_cached.get().unwrap().is_valid());
}

#[test]
fn single_stroke_mode_rebuilds_glyph_as_wires() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let filled = font.render_glyph(B);
    assert!(!filled.is_null());
    assert!(count_sub_shapes(&filled, ShapeEnum::Face) > 0);
    font.set_single_stroke_font(true);
    let stroke = font.render_glyph(B);
    assert!(!stroke.is_null());
    assert!(!filled.is_same(&stroke));
    assert_eq!(0, count_sub_shapes(&stroke, ShapeEnum::Face));
    assert!(count_sub_shapes(&stroke, ShapeEnum::Wire) > 0);
    font.set_single_stroke_font(false);
    let restored = font.render_glyph(B);
    assert!(!restored.is_null());
    assert!(count_sub_shapes(&restored, ShapeEnum::Face) > 0);
}

#[test]
fn failed_reinitialization_preserves_current_font() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let size = font.size();
    assert!(!font.render_glyph(I).is_null());
    assert!(!font.init(&text(""), size, 0));
    assert!(!font.init(&text(""), f64::INFINITY, 0));
    assert!(font.is_valid());
    assert_eq!(size, font.size());
    assert!(!font.render_glyph(I).is_null());
}

#[test]
fn reinitialization_preserves_width_scaling() {
    let font = create_font();
    let Some(font) = font.get() else {
        return skip();
    };
    let initial = font.plan_text(&text("AB"), Hta::Left, Vta::Bottom).unwrap();
    assert!(font.set_width_scaling(1.25));
    let scaled = font.plan_text(&text("AB"), Hta::Left, Vta::Bottom).unwrap();
    assert!(scaled.width() > initial.width());
    font.release();
    assert!(font.init_by_name(&text(FALLBACK_REQUEST), FontFontAspect::Regular, 12.0));
    let reinitialized = font.plan_text(&text("AB"), Hta::Left, Vta::Bottom).unwrap();
    assert_eq!(scaled.width(), reinitialized.width());
}

#[test]
fn family_name_init_overload_forwards_to_font_lookup() {
    let font = StdPrsBRepFont::new();
    if !font.init_by_name(&text(FALLBACK_REQUEST), FontFontAspect::Regular, 12.0) {
        return skip();
    }
    assert!(font.is_valid());
    assert_eq!(12.0, font.size());
    assert!(!font.render_glyph(A).is_null());
}

#[test]
fn concurrent_text_rendering_remains_usable() {
    let font = create_font();
    if font.is_null() {
        return skip();
    }
    assert!(!font.get().unwrap().render_text(&text("ABBA")).is_null());
    let is_valid = AtomicBool::new(true);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                for _ in 0..8 {
                    if font.get().unwrap().render_text(&text("ABBA")).is_null() {
                        is_valid.store(false, Ordering::Relaxed);
                        return;
                    }
                }
            });
        }
    });
    assert!(is_valid.load(Ordering::Relaxed));
}
