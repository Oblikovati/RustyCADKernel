//! BRepFont regularization and glyph/text builders, the twin of
//! tools/oracle/tkv3d_probe_brep_font.cxx.
use super::common::{pnt, xy};
use super::print::{Reader, end, flag, int, real};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_graph::{BRepGraph, CompoundId};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Axis3, Transform3, Xy};
use rustycad_tkservice::font::FontGlyphOutlineSegment as Segment;
use rustycad_tkservice::font::{
    FontGlyphOutline, FontGlyphOutlineContour, FontGlyphOutlineFillRule,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tkv3d::brep_font::{
    BRepFontBuilder, BRepFontBuilderGlyphOptions, BRepFontBuilderTextOptions,
    BRepFontBuilderTextPlanItem, BRepFontPlanarRegion, BRepFontPlanarRegionCurve,
    BRepFontRegularizer, BRepFontRegularizerOptions,
};
use std::sync::Arc;

/// `v3_font_segment`: `Kind P0 P1 P2 P3`.
fn read_segment(r: &mut Reader<'_>) -> Segment {
    let kind = r.integer();
    let p: [Xy; 4] = std::array::from_fn(|_| {
        let x = r.real();
        Xy::new(x, r.real())
    });
    match kind {
        0 => Segment::line(p[0], p[3]),
        1 => Segment::quadratic(p[0], p[1], p[3]),
        _ => Segment::cubic(p[0], p[1], p[2], p[3]),
    }
}

/// `v3_font_outline`: packed contours, then uncovered segments.
fn read_outline(r: &mut Reader<'_>) -> FontGlyphOutline {
    let units = r.real();
    let rule = if r.integer() == 0 {
        FontGlyphOutlineFillRule::NonZero
    } else {
        FontGlyphOutlineFillRule::EvenOdd
    };
    let (overlaps, stroke) = (r.flag(), r.flag());
    let (mut segments, mut contours) = (Vec::new(), Vec::new());
    for _ in 0..r.integer() {
        let count = r.integer();
        let first = segments.len() as u32;
        for _ in 0..count {
            segments.push(read_segment(r));
        }
        contours.push(FontGlyphOutlineContour::new(first, count as u32));
    }
    for _ in 0..r.integer() {
        segments.push(read_segment(r));
    }
    FontGlyphOutline::create(units, rule, overlaps, stroke, segments, contours)
}

/// `v3_font_box`.
fn box_state(bounds: &BoundingBox2) {
    flag(bounds.is_void());
    if let Ok(limits) = bounds.get() {
        limits.into_iter().for_each(real);
    }
}

fn curve_state(curve: &BRepFontPlanarRegionCurve) {
    int(curve.kind() as i32);
    for point in [
        curve.start(),
        curve.control1(),
        curve.control2(),
        curve.end(),
    ] {
        xy(*point);
    }
}

/// `v3_font_region`: every pool in order.
fn region_state(g: &BRepFontPlanarRegion) {
    real(g.units_per_em());
    real(g.tolerance());
    flag(g.is_single_stroke());
    flag(g.is_empty());
    int(g.vertices().len() as i32);
    g.vertices().iter().for_each(|v| xy(*v));
    int(g.curves().len() as i32);
    g.curves().iter().for_each(curve_state);
    int(g.uses().len() as i32);
    for u in g.uses().iter() {
        int(u.curve_index().value() as i32);
        int(u.first_vertex().value() as i32);
        int(u.last_vertex().value() as i32);
        real(u.first_parameter());
        real(u.last_parameter());
        flag(u.is_reversed());
    }
    int(g.loops().len() as i32);
    for l in g.loops().iter() {
        int(l.first_use() as i32);
        int(l.nb_uses() as i32);
        real(l.signed_area());
        box_state(l.bounds());
    }
    int(g.hole_loops().len() as i32);
    g.hole_loops().iter().for_each(|h| int(h.value() as i32));
    int(g.regions().len() as i32);
    for q in g.regions().iter() {
        int(q.outer_loop().value() as i32);
        int(q.first_hole() as i32);
        int(q.nb_holes() as i32);
    }
}

/// `v3_brepfont_regularize`.
pub fn regularize_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let outline = read_outline(&mut r);
    let options = BRepFontRegularizerOptions {
        tolerance: r.real(),
    };
    let result = BRepFontRegularizer.build_with(&outline, &options);
    int(result.status_value() as i32);
    flag(result.is_done());
    flag(result.planar_region().is_some());
    if let Some(region) = result.planar_region() {
        region_state(region);
    }
    end();
}

/// `v3_brepfont_curve`.
pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let curve = BRepFontPlanarRegionCurve::new(&read_segment(&mut r));
    for _ in 0..r.integer() {
        let t = r.real();
        xy(curve.value(t));
        xy(curve.d1(t));
    }
    real(curve.signed_area());
    box_state(&curve.bounds());
    end();
}

fn count(shape: &Shape, kind: ShapeEnum) -> i32 {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).expect("probe explorer");
    let mut found = 0;
    while explorer.more() {
        found += 1;
        explorer.next().expect("probe explorer step");
    }
    found
}

fn for_each_sub_shape(shape: &Shape, kind: ShapeEnum, mut visit: impl FnMut(&Shape)) {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).expect("probe explorer");
    while explorer.more() {
        visit(explorer.current());
        explorer.next().expect("probe explorer step");
    }
}

/// `v3_font_shape`.
fn shape_state(shape: &Shape) {
    flag(shape.is_null());
    if shape.is_null() {
        return;
    }
    int(shape.shape_type().expect("probe shape type") as i32);
    for kind in [
        ShapeEnum::Face,
        ShapeEnum::Wire,
        ShapeEnum::Edge,
        ShapeEnum::Vertex,
    ] {
        int(count(shape, kind));
    }
    let analyzer = BRepCheckAnalyzer::new(shape, true, false, false).expect("probe analyzer");
    flag(analyzer.is_valid().expect("probe validity"));
    for_each_sub_shape(shape, ShapeEnum::Vertex, |vertex| {
        pnt(BRepTool::pnt(&topods::vertex(vertex).expect("probe vertex")).expect("probe point"));
    });
    for_each_sub_shape(shape, ShapeEnum::Edge, |edge| {
        let (first, last) =
            BRepTool::range(&topods::edge(edge).expect("probe edge")).expect("probe range");
        real(first);
        real(last);
        int(edge.orientation() as i32);
    });
}

fn graph_counts(g: &BRepGraph) {
    let topo = g.topo();
    for value in [
        topo.vertices().nb_active(),
        topo.edges().nb_active(),
        topo.co_edges().nb_active(),
        topo.wires().nb_active(),
        topo.faces().nb_active(),
        topo.compounds().nb_active(),
    ] {
        int(value as i32);
    }
}

/// `v3_brepfont_glyph`.
pub fn glyph_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let outline = read_outline(&mut r);
    let options = BRepFontRegularizerOptions {
        tolerance: r.real(),
    };
    let glyph = BRepFontBuilderGlyphOptions {
        size: r.real(),
        tolerance: r.real(),
        to_concatenate_contours: r.flag(),
    };
    let result = BRepFontRegularizer.build_with(&outline, &options);
    flag(result.planar_region().is_some());
    if let Some(region) = result.planar_region() {
        shape_state(&BRepFontBuilder::build_glyph(region, &glyph));
        let mut graph = BRepGraph::new();
        let root = BRepFontBuilder::add_glyph(&mut graph, region, &glyph);
        flag(root.is_valid());
        if root.is_valid() {
            int(root.kind.0);
        }
        graph_counts(&graph);
    }
    end();
}

fn trsf_state(t: &Transform3) {
    for i in 1..=3 {
        for j in 1..=4 {
            real(t.value(i, j).expect("probe matrix entry"));
        }
    }
}

/// `v3_brepfont_text`.
pub fn text_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let outline = read_outline(&mut r);
    let options = BRepFontRegularizerOptions {
        tolerance: r.real(),
    };
    let (size, tolerance) = (r.real(), r.real());
    let items: Vec<BRepFontBuilderTextPlanItem> = (0..r.integer())
        .map(|_| {
            let region = r.integer() as usize;
            let x = r.real();
            BRepFontBuilderTextPlanItem::new(region, Xy::new(x, r.real()))
        })
        .collect();
    let (location, normal, x_direction, concatenate) = (r.pnt(), r.dir(), r.dir(), r.flag());
    let mut result = BRepFontRegularizer.build_with(&outline, &options);
    let Some(region) = result.change_planar_region().take() else {
        flag(false);
        end();
        return;
    };
    let empty = FontGlyphOutline::create(
        1000.0,
        FontGlyphOutlineFillRule::NonZero,
        false,
        false,
        Vec::new(),
        Vec::new(),
    );
    let empty_region = BRepFontRegularizer
        .build(&empty)
        .planar_region()
        .clone()
        .expect("empty region");
    let regions = vec![Arc::new(region), Arc::new(empty_region)];
    let plan = BRepFontBuilder::create_text_plan(
        size,
        tolerance,
        Xy::new(0.0, 0.0),
        Xy::new(10.0, 10.0),
        regions,
        items,
    );
    flag(plan.is_some());
    let Some(plan) = plan else {
        end();
        return;
    };
    let mut text = BRepFontBuilderTextOptions {
        to_concatenate_contours: concatenate,
        ..Default::default()
    };
    match Axis3::new(location, normal, x_direction) {
        Ok(pen) => text.pen = pen,
        Err(failure) => print!("{} ", failure.exception_type()),
    }
    text_shape_state(&BRepFontBuilder::build_text_with(&plan, &text));
    let mut graph = BRepGraph::new();
    let root = BRepFontBuilder::add_text_with(&mut graph, &plan, &text);
    flag(root.is_valid());
    graph_counts(&graph);
    if root.is_valid() {
        let references = graph
            .refs()
            .children()
            .ids_of(CompoundId::from_node_id(root))
            .to_vec();
        int(references.len() as i32);
        for reference in references {
            let location = graph.refs().generic().local_location(reference.into());
            flag(location.is_identity());
            trsf_state(&location.transformation());
        }
    }
    end();
}

fn text_shape_state(shape: &Shape) {
    flag(shape.is_null());
    if shape.is_null() {
        return;
    }
    int(shape.nb_children());
    let mut iterator = ShapeIterator::new(shape, true, true);
    while iterator.more() {
        trsf_state(
            &iterator
                .value()
                .expect("probe child")
                .location()
                .transformation(),
        );
        iterator.next();
    }
}
