//! Translation of OCCT's GTests/BRepFont_Builder_Test.cxx at 3d097a0. The tests named
//! after `BRepFont_BuilderGraphTest` and `BRepFont_BuilderTextTest` keep their suites as
//! name prefixes. Font-dependent cases skip, as natively, when FreeType or the embedded
//! fallback glyph is unavailable.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_graph::{BRepGraph, CompoundId};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator};
use rustycad_tkernel::standard::RuntimeTyped;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{self, Axis3, Point3, Xy};
use rustycad_tkservice::font::FontGlyphOutlineSegment as Segment;
use rustycad_tkservice::font::{FontFTFont, FontFTFontParams, FontFontMgr, FontGlyphOutline};
use rustycad_tkservice::font::{
    FontGlyphOutlineContour as Contour, FontGlyphOutlineFillRule as FillRule,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use rustycad_tkv3d::brep_font::{
    BRepFontBuilder, BRepFontBuilderGlyphOptions, BRepFontBuilderTextOptions,
    BRepFontBuilderTextPlan, BRepFontBuilderTextPlanItem, BRepFontPlanarRegion,
    BRepFontRegularizer, BRepFontRegularizerOptions,
};
use std::sync::Arc;

fn append_closed_contour(
    points: &[(f64, f64)],
    segments: &mut Vec<Segment>,
    contours: &mut Vec<Contour>,
) {
    let first = segments.len() as u32;
    for index in 0..points.len() {
        let (a, b) = (points[index], points[(index + 1) % points.len()]);
        segments.push(Segment::line(Xy::new(a.0, a.1), Xy::new(b.0, b.1)));
    }
    contours.push(Contour::new(first, points.len() as u32));
}

fn regularize(outline: &FontGlyphOutline, tolerance: f64) -> Option<BRepFontPlanarRegion> {
    let mut result =
        BRepFontRegularizer.build_with(outline, &BRepFontRegularizerOptions { tolerance });
    result.change_planar_region().take()
}

fn polygons_region(polygons: &[&[(f64, f64)]], overlaps: bool) -> Option<BRepFontPlanarRegion> {
    let (mut segments, mut contours) = (Vec::new(), Vec::new());
    for polygon in polygons {
        append_closed_contour(polygon, &mut segments, &mut contours);
    }
    let outline = FontGlyphOutline::create(
        1000.0,
        FillRule::NonZero,
        overlaps,
        false,
        segments,
        contours,
    );
    regularize(&outline, 1.0e-7)
}

fn create_triangle() -> Option<BRepFontPlanarRegion> {
    polygons_region(&[&[(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)]], false)
}

fn create_region_with_hole() -> Option<BRepFontPlanarRegion> {
    polygons_region(
        &[
            &[(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)],
            &[(5.0, 5.0), (5.0, 15.0), (15.0, 15.0), (15.0, 5.0)],
        ],
        false,
    )
}

fn create_same_winding_nested_region() -> Option<BRepFontPlanarRegion> {
    polygons_region(
        &[
            &[(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)],
            &[(5.0, 5.0), (15.0, 5.0), (15.0, 15.0), (5.0, 15.0)],
        ],
        false,
    )
}

fn create_single_stroke_region() -> Option<BRepFontPlanarRegion> {
    let p = |x, y| Xy::new(x, y);
    let segments = vec![
        Segment::line(p(0.0, 0.0), p(10.0, 0.0)),
        Segment::quadratic(p(10.0, 0.0), p(15.0, 5.0), p(20.0, 0.0)),
    ];
    let outline = FontGlyphOutline::create(
        1000.0,
        FillRule::NonZero,
        false,
        true,
        segments,
        vec![Contour::new(0, 2)],
    );
    regularize(&outline, 1.0e-7)
}

fn create_overlapping_region() -> Option<BRepFontPlanarRegion> {
    polygons_region(
        &[
            &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
            &[(5.0, 0.0), (15.0, 0.0), (15.0, 10.0), (5.0, 10.0)],
        ],
        true,
    )
}

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
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

fn options(size: f64, concatenate: bool) -> BRepFontBuilderGlyphOptions {
    BRepFontBuilderGlyphOptions {
        size,
        tolerance: 1.0e-7,
        to_concatenate_contours: concatenate,
    }
}

#[test]
fn builds_face() {
    let region = create_triangle().unwrap();
    let shape = BRepFontBuilder::build_glyph(&region, &options(10.0, false));
    assert!(!shape.is_null());
    assert!(is_valid(&shape));
}

#[test]
fn concatenates_contour_into_one_bspline_edge() {
    let region = create_triangle().unwrap();
    let default_shape =
        BRepFontBuilder::build_glyph(&region, &BRepFontBuilderGlyphOptions::default());
    assert!(!default_shape.is_null());
    assert_eq!(3, count(&default_shape, ShapeEnum::Edge));
    let shape = BRepFontBuilder::build_glyph(&region, &options(10.0, true));
    assert!(!shape.is_null());
    assert!(is_valid(&shape));
    let mut explorer = Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    assert!(explorer.more());
    let edge = topods::edge(explorer.current()).unwrap();
    let (curve, first, last) = BRepTool::curve(&edge).unwrap();
    assert!(curve.is_kind(GeomBSplineCurve::static_type()));
    explorer.next().unwrap();
    assert!(!explorer.more());
    assert!(first < last);
}

#[test]
fn composite_contours_preserve_face_with_hole() {
    let region = create_region_with_hole().unwrap();
    let shape = BRepFontBuilder::build_glyph(&region, &options(10.0, true));
    assert!(!shape.is_null());
    assert!(is_valid(&shape));
    assert_eq!(1, count(&shape, ShapeEnum::Face));
    assert_eq!(2, count(&shape, ShapeEnum::Wire));
    assert_eq!(2, count(&shape, ShapeEnum::Edge));
}

#[test]
fn non_boundary_contours_are_excluded() {
    let region = create_same_winding_nested_region().unwrap();
    assert_eq!(4, region.uses().len());
    let shape = BRepFontBuilder::build_glyph(&region, &BRepFontBuilderGlyphOptions::default());
    assert!(!shape.is_null());
    assert_eq!(4, count(&shape, ShapeEnum::Edge));
}

#[test]
fn composite_single_stroke_builds_one_open_edge() {
    let region = create_single_stroke_region().unwrap();
    let glyph_options = BRepFontBuilderGlyphOptions {
        to_concatenate_contours: true,
        ..Default::default()
    };
    let shape = BRepFontBuilder::build_glyph(&region, &glyph_options);
    assert!(!shape.is_null());
    assert!(is_valid(&shape));
    assert_eq!(0, count(&shape, ShapeEnum::Face));
    assert_eq!(1, count(&shape, ShapeEnum::Wire));
    assert_eq!(1, count(&shape, ShapeEnum::Edge));
}

#[test]
fn builds_marked_overlap_as_separate_faces() {
    let region = create_overlapping_region().unwrap();
    let glyph_options = BRepFontBuilderGlyphOptions {
        size: 1000.0,
        ..Default::default()
    };
    let shape = BRepFontBuilder::build_glyph(&region, &glyph_options);
    assert!(!shape.is_null());
    assert!(is_valid(&shape));
    let mut properties = GProps::new();
    BRepGProp::surface_properties(&shape, &mut properties, false, false).unwrap();
    assert!((200.0 - properties.mass()).abs() <= 1.0e-7);
    assert_eq!(2, count(&shape, ShapeEnum::Face));
}

#[test]
fn rejects_non_finite_options() {
    let region = create_triangle().unwrap();
    let glyph_options = BRepFontBuilderGlyphOptions {
        size: f64::INFINITY,
        ..Default::default()
    };
    assert!(BRepFontBuilder::build_glyph(&region, &glyph_options).is_null());
}

#[test]
fn rejects_finite_coordinate_overflow() {
    let region = create_triangle().unwrap();
    let mut glyph_options = BRepFontBuilderGlyphOptions {
        size: f64::MAX,
        ..Default::default()
    };
    assert!(BRepFontBuilder::build_glyph(&region, &glyph_options).is_null());
    glyph_options.to_concatenate_contours = true;
    assert!(BRepFontBuilder::build_glyph(&region, &glyph_options).is_null());
}

#[test]
fn graph_adds_face_to_existing_graph() {
    let region = create_triangle().unwrap();
    let mut graph = BRepGraph::new();
    let existing = graph.editor().compounds().add(&[]);
    assert!(existing.is_valid());
    let root = BRepFontBuilder::add_glyph(&mut graph, &region, &options(10.0, false));
    assert!(root.is_valid());
    assert_eq!(1, graph.topo().compounds().nb_active());
    assert_eq!(1, graph.topo().faces().nb_active());
}

#[test]
fn graph_rejects_non_finite_options_without_changing_graph() {
    let region = create_triangle().unwrap();
    let mut graph = BRepGraph::new();
    let glyph_options = BRepFontBuilderGlyphOptions {
        tolerance: f64::INFINITY,
        ..Default::default()
    };
    assert!(!BRepFontBuilder::add_glyph(&mut graph, &region, &glyph_options).is_valid());
    assert_eq!(0, graph.topo().vertices().nb_active());
}

#[test]
fn graph_single_stroke_adds_wire_without_face() {
    let region = create_single_stroke_region().unwrap();
    let mut graph = BRepGraph::new();
    let root =
        BRepFontBuilder::add_glyph(&mut graph, &region, &BRepFontBuilderGlyphOptions::default());
    assert!(root.is_valid());
    assert_eq!(2, graph.topo().edges().nb_active());
    assert_eq!(1, graph.topo().wires().nb_active());
    assert_eq!(0, graph.topo().faces().nb_active());
}

#[test]
fn graph_adds_marked_overlap_as_separate_faces() {
    let region = create_overlapping_region().unwrap();
    let mut graph = BRepGraph::new();
    let root =
        BRepFontBuilder::add_glyph(&mut graph, &region, &BRepFontBuilderGlyphOptions::default());
    assert!(root.is_valid());
    assert_eq!(2, graph.topo().faces().nb_active());
}

#[test]
fn graph_rejects_finite_coordinate_overflow() {
    let region = create_triangle().unwrap();
    let mut graph = BRepGraph::new();
    let glyph_options = BRepFontBuilderGlyphOptions {
        size: f64::MAX,
        ..Default::default()
    };
    assert!(!BRepFontBuilder::add_glyph(&mut graph, &region, &glyph_options).is_valid());
    assert_eq!(0, graph.topo().vertices().nb_active());
}

/// `createRepeatedGlyphPlan`: two placements of the embedded fallback font's 'I'.
fn create_repeated_glyph_plan() -> Option<BRepFontBuilderTextPlan> {
    let font = FontFTFont::default();
    let params = FontFTFontParams {
        point_size: 72,
        resolution: 4800,
        ..FontFTFontParams::default()
    };
    if !font.init_from(
        &FontFontMgr::embed_fallback_font(),
        &"Embedded fallback".into(),
        &params,
        0,
    ) {
        return None;
    }
    let glyph = font.resolve_glyph('I' as u32)?;
    let outline = font.load_glyph_outline(&glyph)?;
    let mut result =
        BRepFontRegularizer.build_with(&outline, &BRepFontRegularizerOptions { tolerance: 0.01 });
    if !result.is_done() {
        return None;
    }
    let region = Arc::new(result.change_planar_region().take()?);
    let items = vec![
        BRepFontBuilderTextPlanItem::new(0, Xy::new(0.0, 0.0)),
        BRepFontBuilderTextPlanItem::new(0, Xy::new(20.0, 0.0)),
    ];
    BRepFontBuilder::create_text_plan(
        12.0,
        1.0e-7,
        Xy::new(0.0, 0.0),
        Xy::new(32.0, 12.0),
        vec![region],
        items,
    )
}

fn create_empty_plan() -> Option<BRepFontBuilderTextPlan> {
    BRepFontBuilder::create_text_plan(
        12.0,
        1.0e-7,
        Xy::new(0.0, 0.0),
        Xy::new(0.0, 0.0),
        Vec::new(),
        Vec::new(),
    )
}

#[test]
fn text_empty_text_does_not_create_topology() {
    let plan = create_empty_plan().unwrap();
    assert!(BRepFontBuilder::build_text(&plan).is_null());
    let mut graph = BRepGraph::new();
    assert!(!BRepFontBuilder::add_text(&mut graph, &plan).is_valid());
    assert_eq!(0, graph.topo().compounds().nb_active());
}

#[test]
fn text_invalid_text_plan_options_are_rejected() {
    let plan = BRepFontBuilder::create_text_plan(
        0.0,
        1.0e-7,
        Xy::new(0.0, 0.0),
        Xy::new(0.0, 0.0),
        Vec::new(),
        Vec::new(),
    );
    assert!(plan.is_none());
}

#[test]
fn text_non_finite_pen_is_rejected_without_changing_graph() {
    let Some(plan) = create_repeated_glyph_plan() else {
        eprintln!("SKIPPED: FreeType or the embedded fallback glyph is unavailable");
        return;
    };
    let mut text_options = BRepFontBuilderTextOptions::default();
    text_options
        .pen
        .set_location(Point3::new(f64::INFINITY, 0.0, 0.0));
    assert!(BRepFontBuilder::build_text_with(&plan, &text_options).is_null());
    let mut graph = BRepGraph::new();
    assert!(!BRepFontBuilder::add_text_with(&mut graph, &plan, &text_options).is_valid());
    assert_eq!(0, graph.topo().vertices().nb_active());
}

#[test]
fn text_repeated_glyph_uses_located_topods_occurrences() {
    let Some(plan) = create_repeated_glyph_plan() else {
        eprintln!("SKIPPED: FreeType or the embedded fallback glyph is unavailable");
        return;
    };
    let text_options = BRepFontBuilderTextOptions {
        pen: Axis3::new(Point3::new(3.0, 4.0, 5.0), gp::dz(), gp::dx()).unwrap(),
        to_concatenate_contours: false,
    };
    let shape = BRepFontBuilder::build_text_with(&plan, &text_options);
    assert!(!shape.is_null());
    assert_eq!(2, shape.nb_children());
    let mut iterator = ShapeIterator::new(&shape, true, true);
    let first = iterator.value().unwrap().clone();
    iterator.next();
    let second = iterator.value().unwrap().clone();
    assert!(first.is_partner(&second));
    assert!(!first.location().is_equal(second.location()));
    let origin = Point3::new(0.0, 0.0, 0.0).transformed(first.location().transformation());
    assert_eq!(3.0, origin.x());
    assert_eq!(4.0, origin.y());
    assert_eq!(5.0, origin.z());
}

#[test]
fn text_prebuilt_glyph_shapes_are_placed_without_rebuilding() {
    let Some(plan) = create_repeated_glyph_plan() else {
        eprintln!("SKIPPED: FreeType or the embedded fallback glyph is unavailable");
        return;
    };
    let glyph_options = BRepFontBuilderGlyphOptions {
        size: plan.size(),
        tolerance: plan.tolerance(),
        to_concatenate_contours: false,
    };
    let glyphs = vec![BRepFontBuilder::build_glyph(plan.region(0), &glyph_options)];
    assert!(!glyphs[0].is_null());
    let text = BRepFontBuilder::build_text_from_glyphs(
        &plan,
        &glyphs,
        &BRepFontBuilderTextOptions::default(),
    );
    assert!(!text.is_null());
    let mut iterator = ShapeIterator::new(&text, true, true);
    assert!(iterator.more());
    assert!(glyphs[0].is_partner(iterator.value().unwrap()));
    iterator.next();
    assert!(iterator.more());
    assert!(glyphs[0].is_partner(iterator.value().unwrap()));
}

#[test]
fn text_repeated_glyph_shares_one_graph_definition() {
    let Some(plan) = create_repeated_glyph_plan() else {
        eprintln!("SKIPPED: FreeType or the embedded fallback glyph is unavailable");
        return;
    };
    let mut graph = BRepGraph::new();
    let existing = graph.editor().compounds().add(&[]);
    assert!(existing.is_valid());
    let root = BRepFontBuilder::add_text(&mut graph, &plan);
    assert!(root.is_valid());
    assert_eq!(2, graph.topo().compounds().nb_active());
    let references = graph
        .refs()
        .children()
        .ids_of(CompoundId::from_node_id(root))
        .to_vec();
    assert_eq!(2, references.len());
    let first_definition = graph.refs().generic().child_node(references[0].into());
    let last_definition = graph.refs().generic().child_node(references[1].into());
    assert_eq!(first_definition, last_definition);
    assert!(
        graph
            .refs()
            .generic()
            .local_location(references[0].into())
            .is_identity()
    );
    assert!(
        !graph
            .refs()
            .generic()
            .local_location(references[1].into())
            .is_identity()
    );
}
