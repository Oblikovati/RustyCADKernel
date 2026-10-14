//! `BRepFont_Builder::AddGlyph` and `AddText`: glyph and text topology in a caller-owned
//! BRepGraph, with one shared definition per glyph region.
use super::builder::{BRepFontBuilder, BRepFontBuilderGlyphOptions, BRepFontBuilderTextOptions};
use super::builder::{BRepFontBuilderTextPlan, is_valid_plan, item_translation, pen_transform};
use super::builder_glyph::{curve_parameter, glyph_plane, glyph_scale, has_range, has_valid_range};
use super::builder_glyph::{make_curve_2d, scaled_points};
use super::planar_region::BRepFontPlanarRegion;
use rustycad_tkbrep::brep_graph::{BRepGraph, CoEdgeId, NodeId, VertexId, WireId};
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::gp;
use rustycad_tkmath::toploc::Location;

/// `arrayLength`: a size that fits a native `int` array length.
fn array_length(value: usize) -> Option<i32> {
    i32::try_from(value).ok()
}

/// `isStructurallyValid`: every index and range of the region is consistent.
fn is_structurally_valid(region: &BRepFontPlanarRegion) -> bool {
    let (nb_curves, nb_vertices) = (region.curves().len() as u32, region.vertices().len() as u32);
    let uses_valid = region.uses().iter().all(|use_| {
        use_.curve_index().value() < nb_curves
            && use_.first_vertex().value() < nb_vertices
            && use_.last_vertex().value() < nb_vertices
            && has_valid_range(use_)
    });
    let loops_valid = region.loops().iter().all(|loop_| {
        loop_.nb_uses() != 0
            && array_length(loop_.nb_uses() as usize).is_some()
            && u64::from(loop_.first_use()) + u64::from(loop_.nb_uses())
                <= region.uses().len() as u64
    });
    if !uses_valid || !loops_valid {
        return false;
    }
    if region.is_single_stroke() {
        return !region.loops().is_empty() && array_length(region.loops().len()).is_some();
    }
    if region.regions().is_empty() || array_length(region.regions().len()).is_none() {
        return false;
    }
    region.regions().iter().all(|material| {
        let first = u64::from(material.first_hole());
        let end = first + u64::from(material.nb_holes());
        material.outer_loop().value() < region.loops().len() as u32
            && array_length(material.nb_holes() as usize).is_some()
            && end <= region.hole_loops().len() as u64
            && region.hole_loops().as_slice()[first as usize..end as usize]
                .iter()
                .all(|hole| hole.value() < region.loops().len() as u32)
    })
}

/// The scaled 2D and 3D curves of one glyph.
type GlyphCurves = (Vec<Handle<dyn Geom2dCurve>>, Vec<Handle<dyn GeomCurve>>);

/// The scaled 2D and 3D curves of a region, or None when one is unusable.
fn graph_curves(region: &BRepFontPlanarRegion, scale: f64) -> Result<Option<GlyphCurves>, Failure> {
    let (mut curves_2d, mut curves_3d) = (Vec::new(), Vec::new());
    for curve in region.curves().iter() {
        let curve_2d = make_curve_2d(curve, scale);
        if !has_range(&curve_2d) {
            return Ok(None);
        }
        let curve_3d = geom_lib::to_3d(gp::xoy(), &curve_2d)?;
        if curve_3d.is_null() {
            return Ok(None);
        }
        curves_2d.push(curve_2d);
        curves_3d.push(curve_3d);
    }
    Ok(Some((curves_2d, curves_3d)))
}

/// The graph nodes built so far for one glyph.
struct GraphGlyph<'a> {
    region: &'a BRepFontPlanarRegion,
    tolerance: f64,
    curves_2d: Vec<Handle<dyn Geom2dCurve>>,
    curves_3d: Vec<Handle<dyn GeomCurve>>,
    co_edges: Vec<CoEdgeId>,
    wires: Vec<WireId>,
}

fn add_co_edges(
    graph: &mut BRepGraph,
    glyph: &mut GraphGlyph<'_>,
    vertices: &[VertexId],
) -> Option<()> {
    for use_ in glyph.region.uses().iter() {
        let curve_index = use_.curve_index().value() as usize;
        let (first, last) = (
            use_.first_vertex().value() as usize,
            use_.last_vertex().value() as usize,
        );
        if curve_index >= glyph.curves_2d.len()
            || first >= vertices.len()
            || last >= vertices.len()
            || !has_valid_range(use_)
        {
            return None;
        }
        let curve_2d = &glyph.curves_2d[curve_index];
        let (u1, u2) = (
            curve_parameter(curve_2d, use_.first_parameter()),
            curve_parameter(curve_2d, use_.last_parameter()),
        );
        let (start, end) = if use_.is_reversed() {
            (vertices[last], vertices[first])
        } else {
            (vertices[first], vertices[last])
        };
        let edge = graph.editor().edges().add(
            start,
            end,
            &glyph.curves_3d[curve_index],
            u1,
            u2,
            glyph.tolerance,
        );
        if !edge.is_valid() {
            return None;
        }
        let orientation = if use_.is_reversed() {
            ParityOrientation::REVERSED
        } else {
            ParityOrientation::FORWARD
        };
        let co_edge = graph.editor().co_edges().add(edge, orientation);
        if !co_edge.is_valid() {
            return None;
        }
        glyph.co_edges.push(co_edge);
    }
    Some(())
}

fn add_wires(graph: &mut BRepGraph, glyph: &mut GraphGlyph<'_>) -> Option<()> {
    for loop_ in glyph.region.loops().iter() {
        let first = loop_.first_use() as usize;
        let end = first + loop_.nb_uses() as usize;
        if loop_.nb_uses() == 0 || end > glyph.co_edges.len() {
            return None;
        }
        array_length(loop_.nb_uses() as usize)?;
        let wire = graph.editor().wires().add(&glyph.co_edges[first..end]);
        if !wire.is_valid() {
            return None;
        }
        glyph.wires.push(wire);
    }
    Some(())
}

// Binds the coedges of a face loop to the face, with their p-curves.
fn bind_loop_to_face(
    graph: &mut BRepGraph,
    glyph: &GraphGlyph<'_>,
    loop_index: usize,
    face: rustycad_tkbrep::brep_graph::FaceId,
) -> Result<(), Failure> {
    let loop_ = &glyph.region.loops().as_slice()[loop_index];
    for offset in 0..loop_.nb_uses() as usize {
        let use_index = loop_.first_use() as usize + offset;
        let use_ = &glyph.region.uses().as_slice()[use_index];
        let curve_2d = &glyph.curves_2d[use_.curve_index().value() as usize];
        let mut co_edges = graph.editor().co_edges();
        co_edges.set_face_id(glyph.co_edges[use_index], face)?;
        let (u1, u2) = (
            curve_parameter(curve_2d, use_.first_parameter()),
            curve_parameter(curve_2d, use_.last_parameter()),
        );
        co_edges.set_pcurve(glyph.co_edges[use_index], curve_2d, u1, u2)?;
    }
    Ok(())
}

// One face per material region; the roots of the glyph.
fn add_faces(
    graph: &mut BRepGraph,
    glyph: &GraphGlyph<'_>,
) -> Result<Option<Vec<NodeId>>, Failure> {
    let (region, loops) = (glyph.region, glyph.region.loops().as_slice());
    let surface: Handle<dyn GeomSurface> = glyph_plane();
    let mut roots = Vec::with_capacity(region.regions().len());
    for material in region.regions().iter() {
        let outer = material.outer_loop().value() as usize;
        if outer >= glyph.wires.len() {
            return Ok(None);
        }
        if loops[outer].signed_area() < 0.0 {
            graph.editor().wires().reverse(glyph.wires[outer]);
        }
        let first = material.first_hole() as usize;
        let end = first + material.nb_holes() as usize;
        if end > region.hole_loops().len()
            || (material.nb_holes() > 0 && array_length(material.nb_holes() as usize).is_none())
        {
            return Ok(None);
        }
        let mut holes = Vec::with_capacity(end - first);
        for hole in &region.hole_loops().as_slice()[first..end] {
            let hole = hole.value() as usize;
            if hole >= glyph.wires.len() {
                return Ok(None);
            }
            if loops[hole].signed_area() > 0.0 {
                graph.editor().wires().reverse(glyph.wires[hole]);
            }
            holes.push(glyph.wires[hole]);
        }
        let face =
            graph
                .editor()
                .faces()
                .add(&surface, glyph.wires[outer], &holes, glyph.tolerance);
        if !face.is_valid() {
            return Ok(None);
        }
        roots.push(NodeId::from(face));
        bind_loop_to_face(graph, glyph, outer, face)?;
        for hole in &region.hole_loops().as_slice()[first..end] {
            bind_loop_to_face(graph, glyph, hole.value() as usize, face)?;
        }
    }
    Ok(Some(roots))
}

fn add_glyph_nodes(
    graph: &mut BRepGraph,
    region: &BRepFontPlanarRegion,
    options: &BRepFontBuilderGlyphOptions,
) -> Result<Option<NodeId>, Failure> {
    let Some(scale) = glyph_scale(region, options).filter(|_| is_structurally_valid(region)) else {
        return Ok(None);
    };
    let Some(points) = scaled_points(region, scale) else {
        return Ok(None);
    };
    let Some((curves_2d, curves_3d)) = graph_curves(region, scale)? else {
        return Ok(None);
    };
    let mut vertices = Vec::with_capacity(points.len());
    for point in points {
        let vertex = graph.editor().vertices().add(point, options.tolerance);
        if !vertex.is_valid() {
            return Ok(None);
        }
        vertices.push(vertex);
    }
    let mut glyph = GraphGlyph {
        region,
        tolerance: options.tolerance,
        curves_2d,
        curves_3d,
        co_edges: Vec::new(),
        wires: Vec::new(),
    };
    if add_co_edges(graph, &mut glyph, &vertices).is_none()
        || add_wires(graph, &mut glyph).is_none()
    {
        return Ok(None);
    }
    let roots = if region.is_single_stroke() {
        glyph.wires.iter().map(|wire| NodeId::from(*wire)).collect()
    } else {
        let Some(roots) = add_faces(graph, &glyph)? else {
            return Ok(None);
        };
        roots
    };
    Ok(root_node(graph, &roots))
}

// The single root, or a compound of the roots.
fn root_node(graph: &mut BRepGraph, roots: &[NodeId]) -> Option<NodeId> {
    match roots.len() {
        0 => None,
        1 => Some(roots[0]),
        count => {
            array_length(count)?;
            let compound = graph.editor().compounds().add(roots);
            compound.is_valid().then(|| NodeId::from(compound))
        }
    }
}

impl BRepFontBuilder {
    /// `AddGlyph`: adds one glyph to an existing graph; its root node, or an invalid node
    /// on failure or for an empty region (nodes added before a late failure remain, as
    /// natively). Example: `BRepFontBuilder::add_glyph(&mut graph, &region, &options)`.
    pub fn add_glyph(
        graph: &mut BRepGraph,
        region: &BRepFontPlanarRegion,
        options: &BRepFontBuilderGlyphOptions,
    ) -> NodeId {
        add_glyph_nodes(graph, region, options)
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    /// `AddText(theGraph, thePlan)` with default options; `BRepFontBuilder::add_text(&mut graph, &plan)`.
    pub fn add_text(graph: &mut BRepGraph, plan: &BRepFontBuilderTextPlan) -> NodeId {
        Self::add_text_with(graph, plan, &BRepFontBuilderTextOptions::default())
    }

    /// `AddText(theGraph, thePlan, theOptions)`: one glyph definition per region and one
    /// located compound child per item; an invalid node on failure or for empty text.
    /// Example: `BRepFontBuilder::add_text_with(&mut graph, &plan, &options)`.
    pub fn add_text_with(
        graph: &mut BRepGraph,
        plan: &BRepFontBuilderTextPlan,
        options: &BRepFontBuilderTextOptions,
    ) -> NodeId {
        if !is_valid_plan(plan, &options.pen) {
            return NodeId::default();
        }
        let Some(glyph_roots) = add_glyph_roots(graph, plan) else {
            return NodeId::default();
        };
        add_text_root(graph, plan, options, &glyph_roots)
            .ok()
            .flatten()
            .unwrap_or_default()
    }
}

// The glyph definition of every referenced region (invalid for empty regions).
fn add_glyph_roots(
    graph: &mut BRepGraph,
    plan: &BRepFontBuilderTextPlan,
) -> Option<Vec<Option<NodeId>>> {
    let options = BRepFontBuilderGlyphOptions {
        size: plan.size(),
        tolerance: plan.tolerance(),
        to_concatenate_contours: false,
    };
    let mut roots: Vec<Option<NodeId>> = vec![None; plan.nb_regions()];
    let mut has_geometry = false;
    for item in plan.items() {
        let slot = roots.get_mut(item.region())?;
        if slot.is_none() {
            let region = plan.region(item.region());
            let root = BRepFontBuilder::add_glyph(graph, region, &options);
            if !root.is_valid() && !region.is_empty() {
                return None;
            }
            *slot = Some(root);
        }
        has_geometry = has_geometry || slot.is_some_and(NodeId::is_valid);
    }
    has_geometry.then_some(roots)
}

fn add_text_root(
    graph: &mut BRepGraph,
    plan: &BRepFontBuilderTextPlan,
    options: &BRepFontBuilderTextOptions,
    glyph_roots: &[Option<NodeId>],
) -> Result<Option<NodeId>, Failure> {
    let text_root = graph.editor().compounds().add(&[]);
    if !text_root.is_valid() {
        return Ok(None);
    }
    let pen = pen_transform(&options.pen);
    for item in plan.items() {
        let Some(glyph_root) = glyph_roots[item.region()].filter(|root| root.is_valid()) else {
            continue;
        };
        let reference =
            graph
                .editor()
                .compounds()
                .append(text_root, glyph_root, ParityOrientation::FORWARD)?;
        if !reference.is_valid() {
            return Ok(None);
        }
        let location = pen.multiplied(item_translation(item));
        if !is_identity(&location) {
            graph
                .editor()
                .generic()
                .set_child_ref_local_location(reference, &Location::new(location))?;
        }
    }
    Ok(Some(NodeId::from(text_root)))
}

/// `isIdentity`: the exact identity matrix and zero translation.
fn is_identity(transform: &rustycad_tkmath::gp::Transform3) -> bool {
    (1..=3).all(|row| {
        (1..=4).all(|column| {
            let identity = if column == row { 1.0 } else { 0.0 };
            transform
                .value(row, column)
                .is_ok_and(|value| value == identity)
        })
    })
}
