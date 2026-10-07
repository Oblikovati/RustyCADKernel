//! `GlueEdgesWith3DCurves` of ShapeUpgrade_UnifySameDomain.cxx: the 3D curves of a chain
//! converted to C1 B-splines, concatenated (GeomConvert::ConcatC1 then
//! GeomConvert_CompCurveToBSplineCurve) into the curve of one new edge.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkgeombase::geom_convert::{self, CompCurveToBSplineCurve, HArrayOfBSplineCurves};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeEdge};

/// The C1 B-spline of the 3D curve of the edge on its range, reversed when asked.
fn c1_bspline(edge: &Edge, reverse: bool) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?).cast();
    let mut spline =
        geom_convert::curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)?;
    geom_convert::c0_bspline_to_c1_bspline_curve(&mut spline, precision::confusion(), 1.0e-7)?;
    if reverse {
        spline.get().ok_or_else(null_curve)?.reverse()?;
    }
    Ok(spline)
}

/// The C1 B-splines of a chain, the tolerances of its common vertices and the largest
/// vertex tolerance.
type ChainCurves = (
    Array1<'static, Handle<GeomBSplineCurve>>,
    Array1<'static, f64>,
    f64,
);

/// The curves of the chain oriented from `first_vertex`, the tolerances of the common
/// vertices and the largest vertex tolerance.
fn chain_curves(chain: &[Shape], first_vertex: &Vertex) -> Result<ChainCurves, Failure> {
    let count = chain.len() as i32;
    let mut splines =
        Array1::with_bounds(0, count - 1).map_err(|e| out_of_range(&e.to_string()))?;
    // The last tolerance stays unset natively (0 from the cleared allocation).
    let mut tolerances =
        Array1::with_bounds(0, count - 1).map_err(|e| out_of_range(&e.to_string()))?;
    let mut previous_edge = topods::edge(&chain[0])?;
    let mut previous_vertex = first_vertex.clone();
    let mut max_tolerance = 0.0_f64;
    for (i, shape) in chain.iter().enumerate() {
        let edge = topods::edge(shape)?;
        let (first, last) = topexp::vertices(&edge, false);
        let reverse = !first.is_same(&previous_vertex);
        max_tolerance = max_tolerance
            .max(BRepTool::vertex_tolerance(&first)?)
            .max(BRepTool::vertex_tolerance(&last)?);
        if i > 0 {
            let common = topexp::common_vertex(&previous_edge, &edge).unwrap_or_default();
            set(
                &mut tolerances,
                i as i32 - 1,
                BRepTool::vertex_tolerance(&common)?,
            )?;
        }
        set(&mut splines, i as i32, c1_bspline(&edge, reverse)?)?;
        previous_vertex = if reverse { first } else { last };
        previous_edge = edge;
    }
    Ok((splines, tolerances, max_tolerance))
}

fn set<T>(array: &mut Array1<'_, T>, index: i32, value: T) -> Result<(), Failure> {
    array
        .set_value(index, value)
        .map_err(|e| out_of_range(&e.to_string()))
}

/// The concatenation of the C1 pieces left by ConcatC1.
fn concatenated(
    pieces: &HArrayOfBSplineCurves,
    max_tolerance: f64,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let table = pieces.get().ok_or_else(null_curve)?.array1().clone();
    let first = table
        .value(table.lower())
        .map_err(|e| out_of_range(&e.to_string()))?
        .clone();
    if table.length() <= 1 {
        return Ok(first);
    }
    let mut concat = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn GeomBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )?;
    for i in table.lower() + 1..=table.upper() {
        let piece = table
            .value(i)
            .map_err(|e| out_of_range(&e.to_string()))?
            .clone();
        concat.add(
            &piece.cast::<dyn GeomBoundedCurve>(),
            max_tolerance,
            true,
            true,
            0,
        )?;
    }
    Ok(concat.bspline_curve())
}

/// `GlueEdgesWith3DCurves(aChain, FirstVertex, LastVertex)`: the new edge on the
/// concatenated curve between the two vertices, made same parameter.
pub(super) fn glue_edges_with_3d_curves(
    chain: &[Shape],
    first_vertex: &Vertex,
    last_vertex: &Vertex,
) -> Result<Edge, Failure> {
    let (mut splines, tolerances, max_tolerance) = chain_curves(chain, first_vertex)?;
    let mut indices = Handle::default();
    let mut pieces: HArrayOfBSplineCurves = Handle::default();
    let mut closed = false;
    geom_convert::concat_c1(
        &mut splines,
        &tolerances,
        &mut indices,
        &mut pieces,
        &mut closed,
        precision::confusion(),
    )?;
    let result = concatenated(&pieces, max_tolerance)?;
    let curve = result.get().ok_or_else(null_curve)?;
    let (first, last) = (curve.first_parameter(), curve.last_parameter());
    let edge = BRepLibMakeEdge::from_curve_vertices_params(
        &result.clone().cast(),
        first_vertex,
        last_vertex,
        first,
        last,
    )?
    .edge()?;
    let builder = BRepBuilder::default();
    builder.same_range(&edge, false)?;
    builder.same_parameter(&edge, false)?;
    brep_lib::same_parameter_shape(edge.as_ref(), max_tolerance, true)?;
    Ok(edge)
}

fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_UnifySameDomain: null B-spline; expected a curve of the chain",
        "",
    )
}

fn out_of_range(detail: &str) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("ShapeUpgrade_UnifySameDomain: {detail}; expected an index of the chain"),
        "",
    )
}
