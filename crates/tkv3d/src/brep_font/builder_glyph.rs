//! `BRepFont_Builder::BuildGlyph`: the TopoDS face, wire or compound of one glyph region.
use super::builder::{BRepFontBuilder, BRepFontBuilderGlyphOptions};
use super::planar_region::{BRepFontPlanarRegion, BRepFontPlanarRegionUse};
use super::planar_region_curve::BRepFontPlanarRegionCurve as Curve;
use super::regularizer::is_valid_real;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkgeombase::gc::MakeSegment2d;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkgeombase::geom2d_convert::CompCurveToBSplineCurve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{self, Axis3, Plane3, Point2, Point3, Xy};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::font::FontGlyphOutlineSegmentKind as Kind;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// `isFiniteScaled`.
fn is_finite_scaled(point: &Xy, scale: f64) -> bool {
    is_valid_real(point.x() * scale) && is_valid_real(point.y() * scale)
}

fn scaled(point: &Xy, scale: f64) -> Point2 {
    Point2::new(point.x() * scale, point.y() * scale)
}

/// `makeCurve2d`: the scaled 2D segment or Bezier of a region curve; null when a scaled
/// coordinate is not finite or the segment is degenerate.
pub(super) fn make_curve_2d(curve: &Curve, scale: f64) -> Handle<dyn Geom2dCurve> {
    let defining_valid = is_finite_scaled(curve.start(), scale)
        && is_finite_scaled(curve.end(), scale)
        && (curve.kind() == Kind::Line || is_finite_scaled(curve.control1(), scale))
        && (curve.kind() != Kind::CubicBezier || is_finite_scaled(curve.control2(), scale));
    if !is_valid_real(scale) || scale <= 0.0 || !defining_valid {
        return Handle::default();
    }
    if curve.kind() == Kind::Line {
        let segment =
            MakeSegment2d::from_points(scaled(curve.start(), scale), scaled(curve.end(), scale));
        return segment
            .and_then(|maker| maker.value())
            .map(|segment| segment.cast())
            .unwrap_or_default();
    }
    let mut poles = vec![
        scaled(curve.start(), scale),
        scaled(curve.control1(), scale),
    ];
    if curve.kind() == Kind::CubicBezier {
        poles.push(scaled(curve.control2(), scale));
    }
    poles.push(scaled(curve.end(), scale));
    Geom2dBezierCurve::new(&Array1::from_slice(&mut poles))
        .map(|bezier| Handle::new(bezier).cast())
        .unwrap_or_default()
}

/// `curveParameter`: a normalized parameter mapped onto the curve's range.
pub(super) fn curve_parameter(curve: &Handle<dyn Geom2dCurve>, normalized: f64) -> f64 {
    let first = curve.first_parameter();
    first + (curve.last_parameter() - first) * normalized
}

/// A usable 2D curve has a nonempty range.
pub(super) fn has_range(curve: &Handle<dyn Geom2dCurve>) -> bool {
    !curve.is_null() && curve.last_parameter() > curve.first_parameter()
}

/// A use range lies inside `[0, 1]` and increases.
pub(super) fn has_valid_range(use_: &BRepFontPlanarRegionUse) -> bool {
    use_.first_parameter() >= 0.0
        && use_.last_parameter() <= 1.0
        && use_.last_parameter() > use_.first_parameter()
}

/// The XOY plane of every glyph face.
pub(super) fn glyph_plane() -> Handle<dyn GeomSurface> {
    let plane = Plane3::from_position(Axis3::from_axis2(gp::xoy()));
    Handle::new(GeomPlane::from_pln(plane)).cast()
}

/// The shared inputs of the wires of one glyph (`buildWire` arguments).
struct WireInputs<'a> {
    region: &'a BRepFontPlanarRegion,
    tolerance: f64,
    surface: Handle<dyn GeomSurface>,
    vertices: Vec<Vertex>,
    curves_2d: Vec<Handle<dyn Geom2dCurve>>,
    curves_3d: Vec<Handle<dyn GeomCurve>>,
    concatenate: bool,
}

/// `buildWire`: the wire of one loop (one edge per use, or one composite B-spline edge).
fn build_wire(inputs: &WireInputs<'_>, loop_index: usize) -> Option<Wire> {
    let region = inputs.region;
    let loop_ = region.loops().as_slice().get(loop_index)?;
    let first = loop_.first_use() as usize;
    let end = first + loop_.nb_uses() as usize;
    if loop_.nb_uses() == 0 || end > region.uses().len() {
        return None;
    }
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let mut composite = CompCurveToBSplineCurve::new(ParameterisationType::TgtThetaOver2);
    let uses = &region.uses().as_slice()[first..end];
    let mut previous = uses[0].first_vertex().value();
    for use_ in uses {
        let (first_vertex, last_vertex) = (use_.first_vertex().value(), use_.last_vertex().value());
        let curve_index = use_.curve_index().value() as usize;
        let nb_vertices = region.vertices().len() as u32;
        if curve_index >= inputs.curves_2d.len()
            || first_vertex >= nb_vertices
            || last_vertex >= nb_vertices
            || first_vertex != previous
        {
            return None;
        }
        previous = last_vertex;
        add_use_to_wire(inputs, use_, &mut composite, &mut wire).ok()??;
    }
    if inputs.concatenate {
        add_composite_edge(inputs, &composite, &mut wire).ok()??;
    }
    wire.set_closed(previous == uses[0].first_vertex().value())
        .ok()?;
    Some(wire)
}

// One use: appended to the composite curve, or built as an edge of the wire.
fn add_use_to_wire(
    inputs: &WireInputs<'_>,
    use_: &BRepFontPlanarRegionUse,
    composite: &mut CompCurveToBSplineCurve,
    wire: &mut Wire,
) -> Result<Option<()>, Failure> {
    let curve_index = use_.curve_index().value() as usize;
    let curve_2d = &inputs.curves_2d[curve_index];
    let has_3d = inputs
        .curves_3d
        .get(curve_index)
        .is_some_and(|curve| !curve.is_null());
    if !has_range(curve_2d) || (!inputs.concatenate && !has_3d) || !has_valid_range(use_) {
        return Ok(None);
    }
    let first = curve_parameter(curve_2d, use_.first_parameter());
    let last = curve_parameter(curve_2d, use_.last_parameter());
    if inputs.concatenate {
        let trimmed = Geom2dTrimmedCurve::new(curve_2d, first, last, !use_.is_reversed(), true)?;
        let added = composite.add(&Handle::new(trimmed).cast(), inputs.tolerance, true)?;
        return Ok(added.then_some(()));
    }
    let (first_vertex, last_vertex) = (
        use_.first_vertex().value() as usize,
        use_.last_vertex().value() as usize,
    );
    if first_vertex >= inputs.vertices.len() || last_vertex >= inputs.vertices.len() {
        return Ok(None);
    }
    let (edge_first, edge_last) = if use_.is_reversed() {
        (
            &inputs.vertices[last_vertex],
            &inputs.vertices[first_vertex],
        )
    } else {
        (
            &inputs.vertices[first_vertex],
            &inputs.vertices[last_vertex],
        )
    };
    let maker = BRepLibMakeEdge::from_curve_vertices_params(
        &inputs.curves_3d[curve_index],
        edge_first,
        edge_last,
        first,
        last,
    )?;
    if !maker.is_done() {
        return Ok(None);
    }
    let mut edge = maker.edge()?;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve(
        &edge,
        curve_2d,
        &inputs.surface,
        &Location::default(),
        inputs.tolerance,
        None,
    )?;
    if use_.is_reversed() {
        edge.reverse();
    }
    builder.add(wire, &edge)?;
    Ok(Some(()))
}

// The single B-spline edge of a concatenated contour.
fn add_composite_edge(
    inputs: &WireInputs<'_>,
    composite: &CompCurveToBSplineCurve,
    wire: &mut Wire,
) -> Result<Option<()>, Failure> {
    let curve_2d: Handle<dyn Geom2dCurve> = composite.bspline_curve().cast();
    if curve_2d.is_null() {
        return Ok(None);
    }
    let curve_3d = geom_lib::to_3d(gp::xoy(), &curve_2d)?;
    if curve_3d.is_null() {
        return Ok(None);
    }
    let maker = BRepLibMakeEdge::from_curve(&curve_3d)?;
    if !maker.is_done() {
        return Ok(None);
    }
    let edge = maker.edge()?;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve(
        &edge,
        &curve_2d,
        &inputs.surface,
        &Location::default(),
        inputs.tolerance,
        None,
    )?;
    builder.add(wire, &edge)?;
    Ok(Some(()))
}

/// The option checks shared by `BuildGlyph` and `AddGlyph`; the scale on success.
pub(super) fn glyph_scale(
    region: &BRepFontPlanarRegion,
    options: &BRepFontBuilderGlyphOptions,
) -> Option<f64> {
    let valid = is_valid_real(region.units_per_em())
        && region.units_per_em() > 0.0
        && is_valid_real(options.size)
        && options.size > 0.0
        && is_valid_real(options.tolerance)
        && options.tolerance > 0.0;
    if !valid || region.is_empty() {
        return None;
    }
    let scale = options.size / region.units_per_em();
    is_valid_real(scale).then_some(scale)
}

/// The scaled glyph vertices; None when a scaled coordinate is not finite.
pub(super) fn scaled_points(region: &BRepFontPlanarRegion, scale: f64) -> Option<Vec<Point3>> {
    region
        .vertices()
        .iter()
        .map(|point| {
            let (x, y) = (point.x() * scale, point.y() * scale);
            (is_valid_real(x) && is_valid_real(y)).then(|| Point3::new(x, y, 0.0))
        })
        .collect()
}

fn wire_inputs<'a>(
    region: &'a BRepFontPlanarRegion,
    options: &BRepFontBuilderGlyphOptions,
    scale: f64,
) -> Result<Option<WireInputs<'a>>, Failure> {
    let concatenate = options.to_concatenate_contours;
    let Some(points) = scaled_points(region, scale) else {
        return Ok(None);
    };
    let builder = BRepBuilder::default();
    let mut vertices = Vec::new();
    for point in points.iter().filter(|_| !concatenate) {
        let mut vertex = Vertex::default();
        builder.make_vertex_at(&mut vertex, *point, options.tolerance)?;
        vertices.push(vertex);
    }
    let (mut curves_2d, mut curves_3d) = (Vec::new(), Vec::new());
    for curve in region.curves().iter() {
        let curve_2d = make_curve_2d(curve, scale);
        if !has_range(&curve_2d) {
            return Ok(None);
        }
        if !concatenate {
            curves_3d.push(geom_lib::to_3d(gp::xoy(), &curve_2d)?);
        }
        curves_2d.push(curve_2d);
    }
    let surface = glyph_plane();
    let tolerance = options.tolerance;
    Ok(Some(WireInputs {
        region,
        tolerance,
        surface,
        vertices,
        curves_2d,
        curves_3d,
        concatenate,
    }))
}

// A face per material region (outer loop counter-clockwise, holes clockwise).
fn add_region_faces(
    inputs: &WireInputs<'_>,
    wires: &[Wire],
    compound: &mut Compound,
) -> Result<Option<()>, Failure> {
    let (region, builder) = (inputs.region, BRepBuilder::default());
    let loops = region.loops().as_slice();
    for material in region.regions().iter() {
        let outer = material.outer_loop().value() as usize;
        let Some(outer_wire) = wires.get(outer) else {
            return Ok(None);
        };
        let mut outer_wire = outer_wire.clone();
        if loops[outer].signed_area() < 0.0 {
            outer_wire.reverse();
        }
        let mut face = Face::default();
        builder.make_face_on_surface(&mut face, &inputs.surface, inputs.tolerance)?;
        builder.add(&mut face, &outer_wire)?;
        let first_hole = material.first_hole() as usize;
        if first_hole + material.nb_holes() as usize > region.hole_loops().len() {
            return Ok(None);
        }
        for hole in
            &region.hole_loops().as_slice()[first_hole..first_hole + material.nb_holes() as usize]
        {
            let Some(hole_wire) = wires.get(hole.value() as usize) else {
                return Ok(None);
            };
            let mut hole_wire = hole_wire.clone();
            if loops[hole.value() as usize].signed_area() > 0.0 {
                hole_wire.reverse();
            }
            builder.add(&mut face, &hole_wire)?;
        }
        builder.add(compound, &face)?;
    }
    Ok(Some(()))
}

fn build_glyph_shape(
    region: &BRepFontPlanarRegion,
    options: &BRepFontBuilderGlyphOptions,
) -> Result<Option<Shape>, Failure> {
    let Some(scale) = glyph_scale(region, options) else {
        return Ok(None);
    };
    let Some(inputs) = wire_inputs(region, options, scale)? else {
        return Ok(None);
    };
    let Some(wires) = (0..region.loops().len())
        .map(|index| build_wire(&inputs, index))
        .collect::<Option<Vec<Wire>>>()
    else {
        return Ok(None);
    };
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    if region.is_single_stroke() {
        for wire in &wires {
            builder.add(&mut compound, wire)?;
        }
    } else if add_region_faces(&inputs, &wires, &mut compound)?.is_none() {
        return Ok(None);
    }
    Ok(single_or_compound(compound))
}

/// The only child of a one-child compound, the compound otherwise, None when empty.
pub(super) fn single_or_compound(compound: Compound) -> Option<Shape> {
    match compound.nb_children() {
        0 => None,
        1 => ShapeIterator::new(&compound, true, true)
            .value()
            .ok()
            .cloned(),
        _ => Some(compound.into_shape()),
    }
}

impl BRepFontBuilder {
    /// `BuildGlyph`: the face, wire or compound of one glyph; a null shape on failure or
    /// for an empty region. A geometry exception of the native toolkits is also a failure.
    /// Example: `BRepFontBuilder::build_glyph(&region, &BRepFontBuilderGlyphOptions::default())`.
    pub fn build_glyph(
        region: &BRepFontPlanarRegion,
        options: &BRepFontBuilderGlyphOptions,
    ) -> Shape {
        build_glyph_shape(region, options)
            .ok()
            .flatten()
            .unwrap_or_default()
    }
}
