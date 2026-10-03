//! Native dummy-seam removal used by `FixNotchedEdges` and `FixTails`.
use super::ShapeFixWire;
use crate::shape_build::{ShapeBuildEdge, ShapeBuildVertex};
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepGCurve};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::toploc::Location;

impl ShapeFixWire {
    pub(super) fn fix_dummy_seam(&self, index: i32) -> Result<(), Failure> {
        let wire_data = self.wire_data()?;
        let count = wire_data.nb_edges();
        let following = if index == count { 1 } else { index + 1 };
        let first = wire_data.edge(index)?;
        let second = wire_data.edge(following)?;
        let vertices = DummySeamVertices::from_edges(&first, &second)?;
        self.replace_seam_context(&vertices, &first, &second)?;
        self.replace_adjacent_wire_edges(&wire_data, &vertices, index, following, count)?;
        remove_seam_edges(&wire_data, index, following)?;
        Ok(())
    }

    fn replace_seam_context(
        &self,
        vertices: &DummySeamVertices,
        first: &Edge,
        second: &Edge,
    ) -> Result<(), Failure> {
        let context = self.context();
        if context.is_null() {
            return Ok(());
        }
        context.replace(
            &second.clone().into_shape(),
            &vertices.replacement.clone().into_shape(),
        )?;
        let reversed = Edge::unchecked(vertices.replacement.reversed());
        context.replace(&first.clone().into_shape(), &reversed.into_shape())?;
        self.replace_context_vertices(context, vertices)
    }

    fn replace_context_vertices(
        &self,
        context: Handle<crate::shape_build::ShapeBuildReShape>,
        vertices: &DummySeamVertices,
    ) -> Result<(), Failure> {
        for (source, orientation) in [
            (&vertices.first, vertices.first.orientation()),
            (&vertices.last, vertices.last.orientation()),
        ] {
            let mut replacement = vertices.combined.clone();
            replacement.set_orientation(orientation);
            context.replace(&source.clone().into_shape(), &replacement.into_shape())?;
        }
        Ok(())
    }

    fn replace_adjacent_wire_edges(
        &self,
        wire_data: &Handle<ShapeExtendWireData>,
        vertices: &DummySeamVertices,
        index: i32,
        following: i32,
        count: i32,
    ) -> Result<(), Failure> {
        let previous = if index == 1 { count } else { index - 1 };
        let next = if following == count { 1 } else { following + 1 };
        let previous_edge = wire_data.edge(previous)?;
        let next_edge = wire_data.edge(next)?;
        let previous_copy = ShapeBuildEdge.copy_replace_vertices(
            &previous_edge,
            &Vertex::default(),
            &vertices.combined,
        )?;
        let next_copy = ShapeBuildEdge.copy_replace_vertices(
            &next_edge,
            &vertices.combined,
            &Vertex::default(),
        )?;
        wire_data.set(&previous_copy, previous)?;
        wire_data.set(&next_copy, next)?;
        self.replace_adjacent_context(&previous_edge, &previous_copy, &next_edge, &next_copy)
    }

    fn replace_adjacent_context(
        &self,
        previous: &Edge,
        previous_copy: &Edge,
        next: &Edge,
        next_copy: &Edge,
    ) -> Result<(), Failure> {
        let context = self.context();
        if context.is_null() {
            return Ok(());
        }
        context.replace(
            &previous.clone().into_shape(),
            &previous_copy.clone().into_shape(),
        )?;
        context.replace(&next.clone().into_shape(), &next_copy.clone().into_shape())
    }
}

struct DummySeamVertices {
    first: Vertex,
    last: Vertex,
    combined: Vertex,
    replacement: Edge,
}

impl DummySeamVertices {
    fn from_edges(first: &Edge, second: &Edge) -> Result<Self, Failure> {
        let analysis = crate::shape_analysis::ShapeAnalysisEdge::new();
        let first_vertex = analysis.first_vertex(first);
        let last_vertex = analysis.last_vertex(second);
        let combined = ShapeBuildVertex.combine_vertices(&first_vertex, &last_vertex, 1.0001)?;
        let shared = shared_start_vertex(second, &first_vertex, &last_vertex, &combined);
        let replacement = copy_seam_edge(second, first, &shared, &combined)?;
        Ok(Self {
            first: first_vertex,
            last: last_vertex,
            combined,
            replacement,
        })
    }
}

fn shared_start_vertex(second: &Edge, first: &Vertex, last: &Vertex, combined: &Vertex) -> Vertex {
    let start = crate::shape_analysis::ShapeAnalysisEdge::new().first_vertex(second);
    if start.is_same(first) || start.is_same(last) {
        combined.clone()
    } else {
        start
    }
}

fn copy_seam_edge(
    second: &Edge,
    first: &Edge,
    start: &Vertex,
    combined: &Vertex,
) -> Result<Edge, Failure> {
    let copied = ShapeBuildEdge.copy_replace_vertices(second, start, combined)?;
    copy_reverse_pcurves(&copied, first, first.orientation() == second.orientation())?;
    let builder = BRepBuilder::default();
    builder.same_range(&copied, false)?;
    builder.same_parameter(&copied, false)?;
    Ok(copied)
}

fn remove_seam_edges(
    wire_data: &Handle<ShapeExtendWireData>,
    first: i32,
    second: i32,
) -> Result<(), Failure> {
    let (lower, upper) = if first < second {
        (first, second)
    } else {
        (second, first)
    };
    wire_data.remove(upper)?;
    wire_data.remove(lower)
}

fn copy_reverse_pcurves(to: &Edge, from: &Edge, reverse: bool) -> Result<(), Failure> {
    let source = edge_tshape(from, "ShapeFix_Wire::CopyReversePCurves")?.curves();
    for representation in source {
        let curve = as_gcurve(&representation);
        let Some(curve) = curve.get() else { continue };
        if curve.is_curve_on_surface() {
            copy_reverse_pcurve(to, from, curve, reverse)?;
        }
    }
    Ok(())
}

fn copy_reverse_pcurve(
    to: &Edge,
    from: &Edge,
    source: &dyn BRepGCurve,
    reverse: bool,
) -> Result<(), Failure> {
    let surface = source.surface()?;
    let location = source.location();
    let Some(_target) = matching_surface_representation(to, &surface, &location)? else {
        add_reverse_pcurve(to, from, source, reverse)?;
        return Ok(());
    };
    Ok(())
}

fn matching_surface_representation(
    edge: &Edge,
    surface: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    location: &Location,
) -> Result<Option<Handle<dyn BRepGCurve>>, Failure> {
    let curves = edge_tshape(edge, "ShapeFix_Wire::CopyReversePCurves")?.curves();
    for representation in curves {
        let curve = as_gcurve(&representation);
        let Some(candidate) = curve.get() else {
            continue;
        };
        if candidate.is_curve_on_surface()
            && candidate.surface()? == *surface
            && candidate.location() == *location
        {
            return Ok(Some(curve));
        }
    }
    Ok(None)
}

fn add_reverse_pcurve(
    to: &Edge,
    from: &Edge,
    source: &dyn BRepGCurve,
    reverse: bool,
) -> Result<(), Failure> {
    let (first, last) = source.range();
    let pcurve = copied_curve(&source.pcurve()?, first, last, reverse)?;
    let target = copy_curve_representation(to, source)?;
    target.set_range(pcurve.first, pcurve.last)?;
    target.set_pcurve(&pcurve.curve)?;
    let relative = from
        .location()
        .multiplied(&source.location())?
        .predivided(to.location())?;
    target.set_location(&relative);
    copy_closed_pcurve(
        target
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire::CopyReversePCurves target vanished"))?,
        source,
    )?;
    Ok(())
}

struct ReversedPCurve {
    curve: Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
}

fn copied_curve(
    source: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    reverse: bool,
) -> Result<ReversedPCurve, Failure> {
    let curve = source.copy()?.cast::<dyn Geom2dCurve>();
    if !reverse {
        return Ok(ReversedPCurve { curve, first, last });
    }
    let first = curve.reversed_parameter(first)?;
    let last = curve.reversed_parameter(last)?;
    curve.reverse()?;
    Ok(ReversedPCurve {
        curve,
        first: last,
        last: first,
    })
}

fn copy_curve_representation(
    edge: &Edge,
    source: &dyn BRepGCurve,
) -> Result<Handle<dyn BRepGCurve>, Failure> {
    let representation = source.copy()?;
    edge_tshape(edge, "ShapeFix_Wire::CopyReversePCurves")?
        .change_curves(|curves| curves.push(representation.clone()));
    Ok(as_gcurve(&representation))
}

fn copy_closed_pcurve(target: &dyn BRepGCurve, source: &dyn BRepGCurve) -> Result<(), Failure> {
    if source.is_curve_on_closed_surface() {
        let copy = source.pcurve2()?.copy()?.cast::<dyn Geom2dCurve>();
        target.set_pcurve2(&copy)?;
    }
    Ok(())
}
