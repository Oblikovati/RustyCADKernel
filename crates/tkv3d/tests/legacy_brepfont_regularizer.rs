//! Translation of OCCT's GTests/BRepFont_Regularizer_Test.cxx at 3d097a0.
use rustycad_tkmath::gp::Xy;
use rustycad_tkservice::font::{FontGlyphOutline, FontGlyphOutlineContour as Contour};
use rustycad_tkservice::font::{FontGlyphOutlineFillRule as FillRule, FontGlyphOutlineSegment as Segment};
use rustycad_tkv3d::brep_font::{
    BRepFontPlanarRegion, BRepFontRegularizer, BRepFontRegularizerOptions,
    BRepFontRegularizerResult, BRepFontRegularizerStatus as Status,
};

/// `appendClosedContour`: one line per edge of the closed polygon.
fn append_closed_contour(points: &[Xy], segments: &mut Vec<Segment>, contours: &mut Vec<Contour>) {
    let first = segments.len() as u32;
    for (index, point) in points.iter().enumerate() {
        segments.push(Segment::line(*point, points[(index + 1) % points.len()]));
    }
    contours.push(Contour::new(first, points.len() as u32));
}

/// `appendStationaryCubicContour`: cubics whose controls repeat the endpoints.
fn append_stationary_cubic_contour(
    points: &[Xy],
    segments: &mut Vec<Segment>,
    contours: &mut Vec<Contour>,
) {
    let first = segments.len() as u32;
    for (index, point) in points.iter().enumerate() {
        let next = points[(index + 1) % points.len()];
        segments.push(Segment::cubic(*point, next, *point, next));
    }
    contours.push(Contour::new(first, points.len() as u32));
}

fn outline_with(
    segments: Vec<Segment>,
    contours: Vec<Contour>,
    overlaps: bool,
    rule: FillRule,
    single_stroke: bool,
) -> FontGlyphOutline {
    FontGlyphOutline::create(1000.0, rule, overlaps, single_stroke, segments, contours)
}

/// `createOutline` with the default arguments.
fn outline(segments: Vec<Segment>, contours: Vec<Contour>) -> FontGlyphOutline {
    outline_with(segments, contours, false, FillRule::NonZero, false)
}

fn xy(points: &[(f64, f64)]) -> Vec<Xy> {
    points.iter().map(|&(x, y)| Xy::new(x, y)).collect()
}

fn build(outline: &FontGlyphOutline) -> BRepFontRegularizerResult {
    BRepFontRegularizer.build(outline)
}

fn build_with_tolerance(outline: &FontGlyphOutline, tolerance: f64) -> BRepFontRegularizerResult {
    BRepFontRegularizer.build_with(outline, &BRepFontRegularizerOptions { tolerance })
}

fn region(result: &BRepFontRegularizerResult) -> &BRepFontPlanarRegion {
    result.planar_region().as_ref().expect("planar region")
}

fn contours_of(polygons: &[&[(f64, f64)]]) -> (Vec<Segment>, Vec<Contour>) {
    let (mut segments, mut contours) = (Vec::new(), Vec::new());
    for polygon in polygons {
        append_closed_contour(&xy(polygon), &mut segments, &mut contours);
    }
    (segments, contours)
}

const SQUARE_20: &[(f64, f64)] = &[(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)];

#[test]
fn closed_rectangle_builds_one_region() {
    let (segments, contours) = contours_of(&[&[(0.0, 0.0), (20.0, 0.0), (20.0, 10.0), (0.0, 10.0)]]);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::Success, result.status_value());
    let region = region(&result);
    assert_eq!(4, region.vertices().len());
    assert_eq!(4, region.curves().len());
    assert_eq!(4, region.uses().len());
    assert_eq!(1, region.loops().len());
    assert_eq!(1, region.regions().len());
    assert_eq!(0, region.regions().as_slice()[0].nb_holes());
}

#[test]
fn thin_concave_contour_finds_interior_scanline_point() {
    let (segments, contours) = contours_of(&[&[
        (0.0, 0.0),
        (1.0e12, 0.0),
        (1.0e12, 0.1),
        (0.1, 0.1),
        (0.1, 0.9),
        (1.0e12, 0.9),
        (1.0e12, 1.0),
        (0.0, 1.0),
    ]]);
    let result = build_with_tolerance(&outline(segments, contours), 0.01);
    assert_eq!(Status::Success, result.status_value());
    assert_eq!(1, region(&result).regions().len());
}

#[test]
fn nested_contour_builds_hole() {
    let (segments, contours) =
        contours_of(&[SQUARE_20, &[(5.0, 5.0), (5.0, 15.0), (15.0, 15.0), (15.0, 5.0)]]);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::Success, result.status_value());
    let region = region(&result);
    assert_eq!(2, region.loops().len());
    assert_eq!(1, region.regions().len());
    assert_eq!(1, region.regions().as_slice()[0].nb_holes());
    assert_eq!(1, region.hole_loops().len());
}

#[test]
fn non_zero_fill_drops_same_winding_nested_contour() {
    let (segments, contours) =
        contours_of(&[SQUARE_20, &[(5.0, 5.0), (15.0, 5.0), (15.0, 15.0), (5.0, 15.0)]]);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::Success, result.status_value());
    let region = region(&result);
    assert_eq!(1, region.loops().len());
    assert_eq!(4, region.uses().len());
    assert_eq!(1, region.regions().len());
    assert!(region.hole_loops().is_empty());
}

#[test]
fn even_odd_fill_keeps_same_winding_nested_contour_as_hole() {
    let (segments, contours) =
        contours_of(&[SQUARE_20, &[(5.0, 5.0), (15.0, 5.0), (15.0, 15.0), (5.0, 15.0)]]);
    let result = build(&outline_with(segments, contours, false, FillRule::EvenOdd, false));
    assert_eq!(Status::Success, result.status_value());
    let region = region(&result);
    assert_eq!(2, region.loops().len());
    assert_eq!(1, region.regions().len());
    assert_eq!(1, region.regions().as_slice()[0].nb_holes());
}

#[test]
fn three_nested_winding_transitions_build_island_region() {
    let (segments, contours) = contours_of(&[
        &[(0.0, 0.0), (30.0, 0.0), (30.0, 30.0), (0.0, 30.0)],
        &[(5.0, 5.0), (5.0, 25.0), (25.0, 25.0), (25.0, 5.0)],
        &[(10.0, 10.0), (20.0, 10.0), (20.0, 20.0), (10.0, 20.0)],
    ]);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::Success, result.status_value());
    let region = region(&result);
    assert_eq!(3, region.loops().len());
    assert_eq!(2, region.regions().len());
    assert_eq!(1, region.hole_loops().len());
    assert_eq!(1, region.regions().as_slice()[0].nb_holes());
    assert_eq!(0, region.regions().as_slice()[1].nb_holes());
}

#[test]
fn disjoint_contours_build_separate_regions() {
    let (segments, contours) = contours_of(&[
        &[(0.0, 0.0), (5.0, 0.0), (5.0, 5.0), (0.0, 5.0)],
        &[(20.0, 0.0), (25.0, 0.0), (25.0, 5.0), (20.0, 5.0)],
    ]);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::Success, result.status_value());
    assert_eq!(2, region(&result).regions().len());
}

#[test]
fn concave_stationary_cubics_find_interior_point() {
    let (mut segments, mut contours) = (Vec::new(), Vec::new());
    let points = xy(&[
        (0.0, 0.0),
        (10.0, 0.0),
        (10.0, 3.0),
        (3.0, 3.0),
        (3.0, 7.0),
        (10.0, 7.0),
        (10.0, 10.0),
        (0.0, 10.0),
    ]);
    append_stationary_cubic_contour(&points, &mut segments, &mut contours);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::Success, result.status_value());
    let region = region(&result);
    assert_eq!(1, region.regions().len());
    assert_eq!(0, region.regions().as_slice()[0].nb_holes());
}

#[test]
fn intersecting_contour_returns_no_region() {
    let (segments, contours) = contours_of(&[&[(0.0, 0.0), (10.0, 10.0), (2.0, 8.0), (10.0, 0.0)]]);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::IntersectingContours, result.status_value());
    assert!(!result.is_done());
    assert!(result.planar_region().is_none());
}

#[test]
fn retracing_quadratic_returns_no_region() {
    let p = |x, y| Xy::new(x, y);
    let segments = vec![
        Segment::quadratic(p(0.0, 0.0), p(10.0, 0.0), p(0.0, 0.0)),
        Segment::line(p(0.0, 0.0), p(0.0, 10.0)),
        Segment::line(p(0.0, 10.0), p(-10.0, 10.0)),
        Segment::line(p(-10.0, 10.0), p(-10.0, 0.0)),
        Segment::line(p(-10.0, 0.0), p(0.0, 0.0)),
    ];
    let result = build(&outline(segments, vec![Contour::new(0, 5)]));
    assert_eq!(Status::IntersectingContours, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn quadratic_and_adjacent_line_detect_second_intersection() {
    let p = |x, y| Xy::new(x, y);
    let segments = vec![
        Segment::quadratic(p(0.0, 0.0), p(5.0, 10.0), p(10.0, 0.0)),
        Segment::line(p(10.0, 0.0), p(0.0, 8.0)),
        Segment::line(p(0.0, 8.0), p(0.0, 0.0)),
    ];
    let result = build(&outline(segments, vec![Contour::new(0, 3)]));
    assert_eq!(Status::IntersectingContours, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn quadratic_tangency_with_line_is_intersection() {
    let p = |x, y| Xy::new(x, y);
    let mut segments = vec![
        Segment::quadratic(p(0.0, 0.0), p(5.0, 10.0), p(10.0, 0.0)),
        Segment::line(p(10.0, 0.0), p(0.0, 0.0)),
    ];
    let mut contours = vec![Contour::new(0, 2)];
    let square = xy(&[(4.0, 5.0), (6.0, 5.0), (6.0, 7.0), (4.0, 7.0)]);
    append_closed_contour(&square, &mut segments, &mut contours);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::IntersectingContours, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn marked_self_intersecting_contour_is_reported() {
    let (segments, contours) = contours_of(&[&[(0.0, 0.0), (10.0, 10.0), (2.0, 8.0), (10.0, 0.0)]]);
    let result = build(&outline_with(segments, contours, true, FillRule::NonZero, false));
    assert_eq!(Status::SelfIntersectingContour, result.status_value());
    assert!(!result.is_done());
    assert!(result.planar_region().is_none());
}

#[test]
fn overlap_metadata_without_intersection_builds_normally() {
    let (segments, contours) = contours_of(&[&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]]);
    let result = build(&outline_with(segments, contours, true, FillRule::NonZero, false));
    assert_eq!(Status::Success, result.status_value());
    assert!(result.planar_region().is_some());
}

#[test]
fn marked_inter_loop_overlap_builds_separate_regions() {
    let (segments, contours) = contours_of(&[
        &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
        &[(8.0, 2.0), (12.0, 2.0), (12.0, 8.0), (8.0, 8.0)],
    ]);
    let result = build(&outline_with(segments, contours, true, FillRule::NonZero, false));
    assert_eq!(Status::Success, result.status_value());
    assert_eq!(2, region(&result).regions().len());
}

#[test]
fn touching_contours_are_rejected_as_intersections() {
    let (segments, contours) = contours_of(&[
        &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
        &[(10.0, 10.0), (20.0, 10.0), (20.0, 20.0), (10.0, 20.0)],
    ]);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::IntersectingContours, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn single_stroke_accepts_open_contour() {
    let p = |x, y| Xy::new(x, y);
    let segments = vec![
        Segment::line(p(0.0, 0.0), p(10.0, 0.0)),
        Segment::quadratic(p(10.0, 0.0), p(15.0, 5.0), p(20.0, 0.0)),
    ];
    let outline = outline_with(segments, vec![Contour::new(0, 2)], false, FillRule::NonZero, true);
    let result = build(&outline);
    assert_eq!(Status::Success, result.status_value());
    let region = region(&result);
    assert!(region.is_single_stroke());
    assert_eq!(1, region.loops().len());
    assert!(region.regions().is_empty());
}

#[test]
fn open_contour_is_rejected() {
    let p = |x, y| Xy::new(x, y);
    let segments = vec![
        Segment::line(p(0.0, 0.0), p(10.0, 0.0)),
        Segment::line(p(11.0, 0.0), p(0.0, 0.0)),
    ];
    let result = build(&outline(segments, vec![Contour::new(0, 2)]));
    assert_eq!(Status::OpenContour, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn tolerance_heals_endpoints_and_removes_point_like_segments() {
    let p = |x, y| Xy::new(x, y);
    let segments = vec![
        Segment::line(p(0.0, 0.0), p(10.0, 0.0)),
        Segment::line(p(10.0, 0.0), p(10.0005, 0.0)),
        Segment::line(p(10.0005, 0.0), p(10.0, 10.0)),
        Segment::line(p(10.0, 10.0), p(0.0, 10.0)),
        Segment::line(p(0.0, 10.0), p(0.0, -0.0005)),
        Segment::line(p(0.0, -0.0005), p(0.0, 0.0)),
    ];
    let result = build_with_tolerance(&outline(segments, vec![Contour::new(0, 6)]), 0.001);
    assert_eq!(Status::Success, result.status_value());
    let region = region(&result);
    assert_eq!(4, region.vertices().len());
    assert_eq!(4, region.curves().len());
    assert_eq!(4, region.uses().len());
    assert_eq!(1, region.loops().len());
    assert_eq!(4, region.loops().as_slice()[0].nb_uses());
    assert_eq!(
        [0.0, 0.0, 10.0, 10.0],
        region.loops().as_slice()[0].bounds().get().unwrap()
    );
    for use_ in region.uses().iter() {
        let curve = &region.curves().as_slice()[use_.curve_index().value() as usize];
        let vertices = region.vertices().as_slice();
        let first = vertices[use_.first_vertex().value() as usize];
        let last = vertices[use_.last_vertex().value() as usize];
        assert_eq!(0.0, (*curve.start() - first).square_modulus());
        assert_eq!(0.0, (*curve.end() - last).square_modulus());
    }
}

#[test]
fn point_like_contour_becomes_empty() {
    let segments = vec![Segment::quadratic(
        Xy::new(0.0, 0.0),
        Xy::new(0.0004, 0.0003),
        Xy::new(0.0002, 0.0001),
    )];
    let result = build_with_tolerance(&outline(segments, vec![Contour::new(0, 1)]), 0.001);
    assert_eq!(Status::Empty, result.status_value());
    let region = region(&result);
    assert!(region.is_empty());
    assert!(region.vertices().is_empty());
    assert!(region.curves().is_empty());
    assert!(region.uses().is_empty());
}

#[test]
fn non_finite_geometry_is_rejected() {
    let segments = vec![Segment::line(Xy::new(0.0, 0.0), Xy::new(f64::NAN, 1.0))];
    let result = build(&outline(segments, vec![Contour::new(0, 1)]));
    assert_eq!(Status::InvalidOutline, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn finite_area_overflow_is_rejected() {
    let limit = f64::MAX;
    let (segments, contours) = contours_of(&[&[
        (limit, limit),
        (-limit, limit),
        (-limit, -limit),
        (limit, -limit),
    ]]);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::InvalidOutline, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn orphaned_segment_is_rejected() {
    let (mut segments, contours) =
        contours_of(&[&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]]);
    segments.push(Segment::line(Xy::new(20.0, 0.0), Xy::new(21.0, 0.0)));
    let result = build(&outline(segments, contours));
    assert_eq!(Status::InvalidOutline, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn empty_segments_with_contour_is_rejected() {
    let result = build(&outline(Vec::new(), vec![Contour::new(0, 1)]));
    assert_eq!(Status::InvalidOutline, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn non_packed_contour_ranges_are_rejected() {
    let (mut segments, mut contours) =
        contours_of(&[&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]]);
    segments.push(Segment::line(Xy::new(20.0, 0.0), Xy::new(21.0, 0.0)));
    let square = xy(&[(30.0, 0.0), (40.0, 0.0), (40.0, 10.0), (30.0, 10.0)]);
    append_closed_contour(&square, &mut segments, &mut contours);
    let result = build(&outline(segments, contours));
    assert_eq!(Status::InvalidOutline, result.status_value());
    assert!(result.planar_region().is_none());
}

#[test]
fn invalid_tolerance_and_units_are_rejected() {
    let (segments, contours) = contours_of(&[&[(0.0, 0.0), (10.0, 0.0), (0.0, 10.0)]]);
    let triangle = outline(segments, contours);
    assert_eq!(
        Status::InvalidOutline,
        build_with_tolerance(&triangle, 0.0).status_value()
    );
    let zero_units = FontGlyphOutline::create(
        0.0,
        FillRule::NonZero,
        false,
        false,
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(Status::InvalidOutline, build(&zero_units).status_value());
}
