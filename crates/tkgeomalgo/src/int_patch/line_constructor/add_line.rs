// Rust translation of the static helpers AppendSameVertexA, AppendSameVertexG,
// AppendSameVertexW, AppendSameVertexR and AddLine of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_LineConstructor.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AddLine`: the part of a line between two of its vertices becomes a new line of the
//! same kind, with the vertices confused with its end vertices.
use super::helpers::transitions_of;
use crate::int_patch::{
    IntPatchALine, IntPatchGLine, IntPatchIType, IntPatchLine, IntPatchLineTransitions,
    IntPatchPoint, IntPatchPointLine, IntPatchRLine, IntPatchWLine,
};
use crate::int_surf::IntSurfLineOn2S;
use crate::intf::native_int_cast;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use std::f64::consts::PI;

/// The vertices of `l` confused with its vertex `index` (within the larger tolerance),
/// the vertex `index` left out: `(rank, vertex, distance, tolerance)` for each other
/// vertex. `TabIndex` takes the line number of `index` for each confused vertex.
fn other_vertices(
    vertices: &[IntPatchPoint],
    index: i32,
) -> impl Iterator<Item = (i32, &IntPatchPoint, f64, f64)> {
    let vtx_index: &IntPatchPoint = &vertices[(index - 1) as usize];
    let thetol1: f64 = vtx_index.tolerance();
    vertices.iter().enumerate().filter_map(move |(slot, vtxi)| {
        let i: i32 = slot as i32 + 1;
        if i == index {
            return None;
        }
        let mut thetol2: f64 = vtxi.tolerance();
        if thetol2 < thetol1 {
            thetol2 = thetol1;
        }
        Some((i, vtxi, vtx_index.value().distance(&vtxi.value()), thetol2))
    })
}

/// The vertices of a line (copies), read 1-based.
fn vertices_of(
    nb: i32,
    vertex: impl Fn(i32) -> Result<IntPatchPoint, Failure>,
) -> Result<Vec<IntPatchPoint>, Failure> {
    (1..=nb).map(vertex).collect()
}

/// `AppendSameVertexA`: the vertices of the analytic line confused with its vertex
/// `index` are added to `alig`; returns their number.
fn append_same_vertex_a(
    alig: &IntPatchALine,
    l: &IntPatchALine,
    index: i32,
    tab_index: &mut [i32],
) -> Result<i32, Failure> {
    let vertices = vertices_of(l.nb_vertex(), |i| l.vertex(i))?;
    l.vertex(index)?;
    let mut a: i32 = 0;
    for (i, vtxi, d, thetol2) in other_vertices(&vertices, index) {
        if d <= thetol2 {
            alig.add_vertex(vtxi);
            a += 1;
            tab_index[i as usize] = tab_index[index as usize];
        }
    }
    Ok(a)
}

/// `AppendSameVertexG`: the vertices of the conic line confused with its vertex `index`
/// (in 3d, or by their parameter on the line) are added to `glig` at the parameter of
/// the vertex `index` shifted by `decal`; returns their number.
fn append_same_vertex_g(
    glig: &IntPatchGLine,
    l: &IntPatchGLine,
    index: i32,
    decal: f64,
    tab_index: &mut [i32],
) -> Result<i32, Failure> {
    let vertices = vertices_of(l.nb_vertex(), |i| l.vertex(i))?;
    let p1: f64 = l.vertex(index)?.parameter_on_line();
    let mut a: i32 = 0;
    for (i, vtxi, d, thetol2) in other_vertices(&vertices, index) {
        // The test on the parameters was added on August 20, 1998 ("??? mefiance ???").
        if d <= thetol2 || (p1 - vtxi.parameter_on_line()).abs() < precision::p_confusion() {
            let mut vtx: IntPatchPoint = vtxi.clone();
            vtx.set_parameter(p1 + decal);
            glig.add_vertex(&vtx)?;
            a += 1;
            tab_index[i as usize] = tab_index[index as usize];
        }
    }
    Ok(a)
}

/// `AppendSameVertexW`: the vertices of the walking line confused with its vertex
/// `index` are added to `wlig` at the parameter `par`, with the point and the
/// parameters of the vertex `index`; returns their number.
fn append_same_vertex_w(
    wlig: &IntPatchWLine,
    l: &IntPatchWLine,
    index: i32,
    par: f64,
    tab_index: &mut [i32],
) -> Result<i32, Failure> {
    let vertices = vertices_of(l.nb_vertex(), |i| l.vertex(i))?;
    let vtx_index: IntPatchPoint = l.vertex(index)?;
    let mut a: i32 = 0;
    for (i, vtxi, d, thetol2) in other_vertices(&vertices, index) {
        if d <= thetol2 {
            let mut vtx: IntPatchPoint = vtxi.clone();
            vtx.set_parameter(par);
            let ((u1, v1), (u2, v2)) = (vtx_index.parameters_on_s1(), vtx_index.parameters_on_s2());
            vtx.set_parameters(u1, v1, u2, v2);
            vtx.set_value(vtx_index.value());
            wlig.add_vertex(&vtx, false);
            a += 1;
            tab_index[i as usize] = tab_index[index as usize];
        }
    }
    Ok(a)
}

/// `AddLine` on an analytic line.
fn add_aline(
    l: &Handle<dyn IntPatchLine>,
    (i, j): (i32, i32),
    tab_index: &mut [i32],
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
) -> Result<(), Failure> {
    let aline: Handle<IntPatchALine> = l.cast::<IntPatchALine>();
    let alig = IntPatchALine::new(&aline.curve(), l.is_tangent(), transitions_of(&**l)?);
    let mut index_last_vertex: i32 = if i == j { 1 } else { 2 };
    alig.add_vertex(&aline.vertex(i)?);
    index_last_vertex += append_same_vertex_a(&alig, &aline, i, tab_index)?;
    if i != j {
        alig.add_vertex(&aline.vertex(j)?);
        index_last_vertex += append_same_vertex_a(&alig, &aline, j, tab_index)?;
    }
    alig.set_first_point(1);
    alig.set_last_point(index_last_vertex);
    slin.push(Handle::new(alig).cast());
    Ok(())
}

/// A new walking line on `points`, with the transitions, the arcs and the periods of
/// `wline`.
fn wline_copy(
    wline: &IntPatchWLine,
    points: &Handle<IntSurfLineOn2S>,
    transitions: IntPatchLineTransitions,
) -> IntPatchWLine {
    let tang: bool = wline.is_tangent();
    let wlig: IntPatchWLine = match transitions {
        IntPatchLineTransitions::Undecided => IntPatchWLine::new_undecided(points, tang),
        IntPatchLineTransitions::Touch(s1, s2) => IntPatchWLine::new_touch(points, tang, s1, s2),
        IntPatchLineTransitions::Types(t1, t2) => IntPatchWLine::new(points, tang, t1, t2),
    };
    if wline.has_arc_on_s1() {
        wlig.set_arc_on_s1(&wline.get_arc_on_s1());
    }
    if wline.has_arc_on_s2() {
        wlig.set_arc_on_s2(&wline.get_arc_on_s2());
    }
    wlig
}

/// `AddLine` on a walking line: the points between the vertices `i` and `j`.
fn add_wline(
    l: &Handle<dyn IntPatchLine>,
    (i, j): (i32, i32),
    tab_index: &mut [i32],
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
) -> Result<(), Failure> {
    let wline: Handle<IntPatchWLine> = l.cast::<IntPatchWLine>();
    let points: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    let param_min: i32 = native_int_cast(wline.vertex(i)?.parameter_on_line());
    let param_max: i32 = native_int_cast(wline.vertex(j)?.parameter_on_line());
    for k in param_min..=param_max {
        points.add(&wline.point(k)?);
    }
    let wlig: IntPatchWLine = wline_copy(&wline, &points, transitions_of(&**l)?);
    let mut index_last_vertex: i32 = if i == j { 1 } else { 2 };
    let mut vtx: IntPatchPoint = wline.vertex(i)?;
    vtx.set_parameter(1.0);
    wlig.add_vertex(&vtx, false);
    index_last_vertex += append_same_vertex_w(&wlig, &wline, i, 1.0, tab_index)?;
    if i != j {
        let last: f64 = f64::from(points.nb_points());
        let mut vtx: IntPatchPoint = wline.vertex(j)?;
        vtx.set_parameter(last);
        wlig.add_vertex(&vtx, false);
        index_last_vertex += append_same_vertex_w(&wlig, &wline, j, last, tab_index)?;
    }
    wlig.set_first_point(1);
    wlig.set_last_point(index_last_vertex);
    wlig.set_period(
        wline.u1_period(),
        wline.v1_period(),
        wline.u2_period(),
        wline.v2_period(),
    );
    wlig.compute_vertex_parameters(precision::confusion())?;
    slin.push(Handle::new(wlig).cast());
    Ok(())
}

/// `AddLine` on a restriction: the vertices from `i` to `j` (`AppendSameVertexR` is
/// compiled out natively, `XPU1009` being undefined, and the last index is 2 even for
/// one vertex).
fn add_rline(
    l: &Handle<dyn IntPatchLine>,
    (i, j): (i32, i32),
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
) -> Result<(), Failure> {
    let rline: Handle<IntPatchRLine> = l.cast::<IntPatchRLine>();
    let tang: bool = l.is_tangent();
    let rlig: IntPatchRLine = match transitions_of(&**l)? {
        IntPatchLineTransitions::Undecided => IntPatchRLine::new_undecided(tang),
        IntPatchLineTransitions::Touch(s1, s2) => IntPatchRLine::new_touch(tang, s1, s2),
        IntPatchLineTransitions::Types(t1, t2) => IntPatchRLine::new(tang, t1, t2),
    };
    if rline.is_arc_on_s1() {
        rlig.set_arc_on_s1(&rline.arc_on_s1());
    }
    if rline.is_arc_on_s2() {
        rlig.set_arc_on_s2(&rline.arc_on_s2());
    }
    let mut index_last_vertex: i32 = 2;
    rlig.add_vertex(&rline.vertex(i)?, false);
    for k in i + 1..j {
        rlig.add_vertex(&rline.vertex(k)?, false);
        index_last_vertex += 1;
    }
    if i != j {
        rlig.add_vertex(&rline.vertex(j)?, false);
    }
    rlig.set_first_point(1);
    rlig.set_last_point(index_last_vertex);
    rlig.compute_vertex_parameters(precision::confusion())?;
    slin.push(Handle::new(rlig).cast());
    Ok(())
}

/// A new conic line of the geometry of `gline`, with `transitions`.
fn gline_copy(
    gline: &IntPatchGLine,
    typ: IntPatchIType,
    transitions: IntPatchLineTransitions,
) -> Result<IntPatchGLine, Failure> {
    let tang: bool = gline.is_tangent();
    Ok(match typ {
        IntPatchIType::Lin => IntPatchGLine::from_line(&gline.line()?, tang, transitions)?,
        IntPatchIType::Parabola => {
            IntPatchGLine::from_parabola(&gline.parabola()?, tang, transitions)
        }
        IntPatchIType::Hyperbola => {
            IntPatchGLine::from_hyperbola(&gline.hyperbola()?, tang, transitions)
        }
        IntPatchIType::Circle => IntPatchGLine::from_circle(&gline.circle()?, tang, transitions),
        _ => IntPatchGLine::from_ellipse(&gline.ellipse()?, tang, transitions),
    })
}

/// `AddLine` on a conic line; on a circle or an ellipse with `i > j`, the vertex `j` is
/// taken one turn later.
fn add_gline(
    l: &Handle<dyn IntPatchLine>,
    (i, j): (i32, i32),
    tab_index: &mut [i32],
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
) -> Result<(), Failure> {
    let typ: IntPatchIType = l.arc_type();
    let gline: Handle<IntPatchGLine> = l.cast::<IntPatchGLine>();
    let glig: IntPatchGLine = gline_copy(&gline, typ, transitions_of(&**l)?)?;
    let mut index_last_vertex: i32 = if i == j { 1 } else { 2 };
    glig.add_vertex(&gline.vertex(i)?)?;
    index_last_vertex += append_same_vertex_g(&glig, &gline, i, 0.0, tab_index)?;
    if i != j {
        let closed: bool = matches!(typ, IntPatchIType::Circle | IntPatchIType::Ellipse);
        let decal: f64 = if closed && i > j { PI + PI } else { 0.0 };
        let mut vtx: IntPatchPoint = gline.vertex(j)?;
        if decal != 0.0 {
            vtx.set_parameter(gline.vertex(j)?.parameter_on_line() + PI + PI);
        }
        glig.add_vertex(&vtx)?;
        index_last_vertex += append_same_vertex_g(&glig, &gline, j, decal, tab_index)?;
    }
    glig.set_first_point(1);
    glig.set_last_point(index_last_vertex);
    slin.push(Handle::new(glig).cast());
    Ok(())
}

/// `AddLine(L, i, j, TypeS1, TypeS2, TabIndex, slin)`: appends to `slin` the part of `l`
/// between its vertices `i` and `j` (the vertex `i` alone when `i == j`).
pub(super) fn add_line(
    l: &Handle<dyn IntPatchLine>,
    ij: (i32, i32),
    tab_index: &mut [i32],
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
) -> Result<(), Failure> {
    match l.arc_type() {
        IntPatchIType::Analytic => add_aline(l, ij, tab_index, slin),
        IntPatchIType::Walking => add_wline(l, ij, tab_index, slin),
        IntPatchIType::Restriction => add_rline(l, ij, slin),
        _ => add_gline(l, ij, tab_index, slin),
    }
}
