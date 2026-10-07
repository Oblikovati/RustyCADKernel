//! `UnionPCurves` of ShapeUpgrade_UnifySameDomain.cxx: the p-curves of a merged edge on
//! the non-planar faces of the first edge of its chain, from the p-curves of the chain
//! (joined when they lie on one 2D line or circle, else concatenated as C1 B-splines),
//! reparametrized to the range of the new 3D curve.
use super::geometry::{null_pcurve, try_make_line};
use super::{ShapeUpgradeUnifySameDomain, UnifyState};
use crate::shape_analysis::ShapeAnalysisSurface;
use crate::shape_construct::ShapeConstructProjectCurveOnSurface;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCircle, Geom2dCurve, Geom2dLine,
    Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom2d_convert::{self, CompCurveToBSplineCurve, HArrayOfBSplineCurves};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Vector2;

/// The p-curves of the chain on one face, joined where they share a 2D line or circle.
#[derive(Default)]
struct PCurveRun {
    curves: Vec<Handle<dyn Geom2dCurve>>,
    firsts: Vec<f64>,
    lasts: Vec<f64>,
    forwards: Vec<bool>,
    current_type: Option<CurveType>,
}

/// The p-curve of the merged edge on one face, with its range.
struct ResultPCurve {
    curve: Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
}

/// A copy of the p-curve, the basis of a trimmed one.
fn basis_copy(pcurve: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let copy = pcurve
        .get()
        .ok_or_else(null_pcurve)?
        .copy()?
        .cast::<dyn Geom2dCurve>();
    Ok(match copy.cast::<Geom2dTrimmedCurve>().get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => copy,
    })
}

/// The parameters of the end points on the previous line or circle, sorted, when the
/// p-curve lies on it.
fn same_support(
    pcurve: &Handle<dyn Geom2dCurve>,
    adaptor: &Geom2dAdaptorCurve,
    kind: CurveType,
    previous: &Handle<dyn Geom2dCurve>,
    (first, last): (f64, f64),
) -> Result<Option<(f64, f64)>, Failure> {
    let prev = Geom2dAdaptorCurve::with_curve(previous)?;
    let tol = precision::confusion();
    let (p1, p2) = match kind {
        CurveType::Line => {
            let line = prev.line()?;
            let curve = pcurve.get().ok_or_else(null_pcurve)?;
            if !(line.contains(curve.value(first)?, tol) && line.contains(curve.value(last)?, tol))
            {
                return Ok(None);
            }
            let (p1, p2) = (adaptor.value(first)?, adaptor.value(last)?);
            (elclib::parameter(&line, p1)?, elclib::parameter(&line, p2)?)
        }
        CurveType::Circle => {
            let (circle, previous) = (adaptor.circle()?, prev.circle()?);
            if circle.location().distance(previous.location()) > tol
                || (circle.radius() - previous.radius()).abs() > tol
            {
                return Ok(None);
            }
            let (p1, p2) = (adaptor.value(first)?, adaptor.value(last)?);
            (
                elclib::parameter(&previous, p1)?,
                elclib::parameter(&previous, p2)?,
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(if p1 > p2 { (p2, p1) } else { (p1, p2) }))
}

impl PCurveRun {
    /// Adds the p-curve of one edge of the chain: joined to the last one when on the same
    /// line or circle, else appended with the tolerance of the common vertex.
    fn add(
        &mut self,
        pcurve: Handle<dyn Geom2dCurve>,
        (first, last): (f64, f64),
        forward: bool,
        (previous_edge, edge): (&Edge, &Edge),
        tolerances: &mut Vec<f64>,
    ) -> Result<(), Failure> {
        let adaptor = Geom2dAdaptorCurve::with_curve(&pcurve)?;
        let mut kind = adaptor.get_type()?;
        let mut pcurve = pcurve;
        if matches!(kind, CurveType::BSplineCurve | CurveType::BezierCurve) {
            let curve = pcurve.get().ok_or_else(null_pcurve)?;
            if let Some(line) = try_make_line(curve, first, last)? {
                pcurve = line.cast();
                adaptor.load(&pcurve)?;
                kind = CurveType::Line;
            }
        }
        let Some(&previous_forward) = self.forwards.last() else {
            self.push(basis_copy(&pcurve)?, (first, last), forward, kind);
            return Ok(());
        };
        let last_curve = self.curves.last().cloned().unwrap_or_default();
        let mut joined = None;
        if pcurve == last_curve {
            joined = Some((first, last));
        } else if Some(kind) == self.current_type {
            joined = same_support(&pcurve, &adaptor, kind, &last_curve, (first, last))?;
        }
        match joined {
            Some((new_first, new_last)) => {
                let (values, value) = if previous_forward {
                    (&mut self.lasts, new_last)
                } else {
                    (&mut self.firsts, new_first)
                };
                if let Some(slot) = values.last_mut() {
                    *slot = value;
                }
            }
            None => {
                self.push(basis_copy(&pcurve)?, (first, last), forward, kind);
                let vertex = topexp::common_vertex(previous_edge, edge).unwrap_or_default();
                tolerances.push(BRepTool::vertex_tolerance(&vertex)?);
            }
        }
        Ok(())
    }

    fn push(
        &mut self,
        curve: Handle<dyn Geom2dCurve>,
        (first, last): (f64, f64),
        forward: bool,
        kind: CurveType,
    ) {
        self.curves.push(curve);
        self.firsts.push(first);
        self.lasts.push(last);
        self.forwards.push(forward);
        self.current_type = Some(kind);
    }

    /// The single p-curve (reversed with its range for a reversed edge) or the C1
    /// concatenation of all of them.
    fn result(self, tolerances: &[f64], max_tol: &mut f64) -> Result<ResultPCurve, Failure> {
        if self.curves.len() != 1 {
            return self.concatenation(tolerances, max_tol);
        }
        let curve = self.curves[0].clone();
        let (mut first, mut last) = (self.firsts[0], self.lasts[0]);
        if !self.forwards[0] {
            let c = curve.get().ok_or_else(null_pcurve)?;
            let (new_last, new_first) = (c.reversed_parameter(first)?, c.reversed_parameter(last)?);
            c.reverse()?;
            (first, last) = (new_first, new_last);
        }
        Ok(ResultPCurve { curve, first, last })
    }

    /// "C1 concatenation for PCurveSeq".
    fn concatenation(self, tolerances: &[f64], max_tol: &mut f64) -> Result<ResultPCurve, Failure> {
        let n = self.curves.len() as i32;
        let mut splines: Array1<'static, Handle<Geom2dBSplineCurve>> =
            Array1::with_bounds(0, n - 1).map_err(|e| range_error(&e.to_string()))?;
        for i in 0..self.curves.len() {
            let trimmed = Geom2dTrimmedCurve::new(
                &self.curves[i],
                self.firsts[i],
                self.lasts[i],
                true,
                true,
            )?;
            if !self.forwards[i] {
                trimmed.reverse()?;
            }
            let trimmed: Handle<dyn Geom2dCurve> = Handle::new(trimmed).cast();
            let mut spline = geom2d_convert::curve_to_bspline_curve(
                &trimmed,
                ParameterisationType::TgtThetaOver2,
            )?;
            geom2d_convert::c0_bspline_to_c1_bspline_curve(&mut spline, precision::confusion())?;
            splines
                .set_value(i as i32, spline)
                .map_err(|e| range_error(&e.to_string()))?;
        }
        let mut tols: Array1<'static, f64> = Array1::with_bounds(0, tolerances.len() as i32 - 1)
            .map_err(|e| range_error(&e.to_string()))?;
        for (i, &tol) in tolerances.iter().enumerate() {
            tols.set_value(i as i32, tol)
                .map_err(|e| range_error(&e.to_string()))?;
            *max_tol = max_tol.max(tol);
        }
        let mut indices = Handle::default();
        let mut pieces: HArrayOfBSplineCurves = Handle::default();
        let mut closed = false;
        geom2d_convert::concat_c1(
            &mut splines,
            &tols,
            &mut indices,
            &mut pieces,
            &mut closed,
            precision::confusion(),
        )?;
        let curve = concatenated(&pieces, *max_tol)?;
        let c = curve.get().ok_or_else(null_pcurve)?;
        let (first, last) = (c.first_parameter(), c.last_parameter());
        Ok(ResultPCurve {
            curve: curve.cast(),
            first,
            last,
        })
    }
}

/// The concatenation of the C1 pieces left by ConcatC1.
fn concatenated(
    pieces: &HArrayOfBSplineCurves,
    max_tol: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let table = pieces.get().ok_or_else(null_pcurve)?.array1().clone();
    let first = table
        .value(table.lower())
        .map_err(|e| range_error(&e.to_string()))?
        .clone();
    if table.length() <= 1 {
        return Ok(first);
    }
    let mut concat = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn Geom2dBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )?;
    for i in table.lower() + 1..=table.upper() {
        let piece = table
            .value(i)
            .map_err(|e| range_error(&e.to_string()))?
            .clone();
        concat.add(&piece.cast::<dyn Geom2dBoundedCurve>(), max_tol, true)?;
    }
    Ok(concat.bspline_curve())
}

impl UnifyState {
    /// The non-planar faces of the first edge of the chain (their new faces when
    /// merged), forward, one per located surface.
    fn union_faces(&self, first_edge: &Shape) -> Result<Vec<Face>, Failure> {
        let mut faces: Vec<Face> = Vec::new();
        for shape in self.ef_map.find_from_key(first_edge)?.iter() {
            if self.face_plane_map.is_bound(shape) {
                continue;
            }
            let face = self
                .face_new_face
                .seek(shape)
                .cloned()
                .unwrap_or_else(|| shape.clone());
            let face = topods::face(&face.oriented(Orientation::Forward))?;
            if BRepAdaptorSurface::with_face(&face, false)?.get_type()? == SurfaceType::Plane {
                continue;
            }
            let (surface, location) = BRepTool::surface_with_location(&face)?;
            let mut found = false;
            for previous in &faces {
                let (prev_surface, prev_location) = BRepTool::surface_with_location(previous)?;
                if prev_surface == surface && prev_location == location {
                    found = true;
                    break;
                }
            }
            if !found {
                faces.push(face);
            }
        }
        Ok(faces)
    }

    /// `UnionPCurves(theChain, theEdge)`.
    pub(super) fn union_pcurves(&self, chain: &[Shape], edge: &Edge) -> Result<(), Failure> {
        let (curve, first3d, last3d) = BRepTool::curve(edge)?;
        let tol_edge = BRepTool::edge_tolerance(edge)?;
        let mut max_tol = tol_edge;
        let mut faces = self.union_faces(&chain[0])?;
        let mut results = Vec::new();
        let mut is_seam = false;
        let mut j = 0;
        while j < faces.len() {
            let run = chain_run(chain, &mut faces, j, &mut is_seam)?;
            let (run, tolerances) = run;
            results.push(run.result(&tolerances, &mut max_tol)?);
            j += 1;
        }
        let range3d = (first3d, last3d);
        if results
            .iter()
            .any(|r| ((last3d - first3d) - (r.last - r.first)).abs() > max_tol)
        {
            project_bad_ranges(&mut results, &faces, &curve, range3d)?;
        }
        for result in results.iter_mut() {
            if (first3d - result.first).abs() > max_tol || (last3d - result.last).abs() > max_tol {
                reparametrize(result, range3d)?;
            }
        }
        store(edge, &results, &faces, is_seam, tol_edge)
    }
}

/// The run of p-curves of the chain on face `j`; on the only face, a seam edge of the
/// chain adds the face again for the second p-curves.
fn chain_run(
    chain: &[Shape],
    faces: &mut Vec<Face>,
    j: usize,
    is_seam: &mut bool,
) -> Result<(PCurveRun, Vec<f64>), Failure> {
    let mut run = PCurveRun::default();
    let mut tolerances = Vec::new();
    let mut previous = Edge::default();
    for shape in chain {
        let mut edge = topods::edge(shape)?;
        let forward = edge.orientation() != Orientation::Reversed;
        if *is_seam && j == 1 {
            edge.reverse(); // "second pass"
        }
        let face = faces[j].clone();
        let found = BRepTool::curve_on_face(&edge, &face)?;
        if found.pcurve.is_null() {
            continue;
        }
        let mut range = (found.first, found.last);
        if faces.len() == 1 {
            let mut reversed = edge.clone();
            reversed.reverse();
            let second = BRepTool::curve_on_face(&reversed, &face)?;
            range = (second.first, second.last);
            if second.pcurve != found.pcurve {
                *is_seam = true;
                faces.push(face.clone());
            }
        }
        run.add(
            found.pcurve,
            range,
            forward,
            (&previous, &edge),
            &mut tolerances,
        )?;
        previous = edge;
    }
    if run.curves.is_empty() {
        return Err(Failure::with_type(
            "Standard_NoSuchObject",
            "ShapeUpgrade_UnifySameDomain::UnionPCurves: no p-curve of the chain on the face; expected at least one",
            "",
        ));
    }
    Ok((run, tolerances))
}

/// "Check the results for consistency": every p-curve projected anew from the 3D curve
/// (or its B-spline reparametrized) on the range of the 3D curve.
fn project_bad_ranges(
    results: &mut [ResultPCurve],
    faces: &[Face],
    curve: &Handle<dyn GeomCurve>,
    range3d: (f64, f64),
) -> Result<(), Failure> {
    for (result, face) in results.iter_mut().zip(faces) {
        let surface = BRepTool::surface(face)?;
        let analysis = Handle::new(ShapeAnalysisSurface::new(&surface)?);
        let projector = ShapeConstructProjectCurveOnSurface::new();
        projector.init_analysis(&analysis, precision::confusion());
        let mut projected = Handle::default();
        if projector.perform(curve, range3d, &mut projected, (-1., -1.))? {
            result.curve = projected;
        } else {
            result.curve = reparametrized_bspline(result, range3d)?;
        }
        (result.first, result.last) = range3d;
    }
    Ok(())
}

/// The B-spline of the trimmed p-curve with its knots mapped onto the 3D range.
fn reparametrized_bspline(
    result: &ResultPCurve,
    (first3d, last3d): (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let trimmed: Handle<dyn Geom2dCurve> = Handle::new(Geom2dTrimmedCurve::new(
        &result.curve,
        result.first,
        result.last,
        true,
        true,
    )?)
    .cast();
    let spline =
        geom2d_convert::curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)?;
    let s = spline.get().ok_or_else(null_pcurve)?;
    let mut knots = s.knots();
    bsplclib::reparametrize(first3d, last3d, &mut knots)?;
    s.set_knots(&knots)?;
    Ok(spline.cast())
}

/// "Reparametrize pcurves if needed": a line moved along itself, a circle rotated, any
/// other curve as a reparametrized B-spline.
fn reparametrize(result: &mut ResultPCurve, range3d: (f64, f64)) -> Result<(), Failure> {
    let adaptor = Geom2dAdaptorCurve::with_curve(&result.curve)?;
    result.curve = match adaptor.get_type()? {
        CurveType::Line => {
            let direction = adaptor.line()?.direction();
            let offset = Vector2::new(-direction.x() * range3d.0, -direction.y() * range3d.0);
            let point = adaptor.value(result.first)?.translated(offset);
            Handle::new(Geom2dLine::from_point(point, direction)).cast()
        }
        CurveType::Circle => {
            let circle = adaptor.circle()?;
            let mut position = circle.position();
            position.rotate(circle.location(), result.first - range3d.0);
            Handle::new(Geom2dCircle::new(position, circle.radius())?).cast()
        }
        _ => reparametrized_bspline(result, range3d)?,
    };
    (result.first, result.last) = range3d;
    Ok(())
}

/// The p-curves stored on the edge: the pair of a seam on the face, or one per face.
fn store(
    edge: &Edge,
    results: &[ResultPCurve],
    faces: &[Face],
    is_seam: bool,
    tol_edge: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    if is_seam {
        return builder.update_edge_pcurves_on_face(
            edge,
            (&results[0].curve, &results[1].curve),
            &faces[0],
            tol_edge,
        );
    }
    for (result, face) in results.iter().zip(faces) {
        builder.update_edge_pcurve_on_face(edge, &result.curve, face, tol_edge)?;
    }
    Ok(())
}

fn range_error(detail: &str) -> Failure {
    Failure::with_type(
        "Standard_RangeError",
        &format!(
            "ShapeUpgrade_UnifySameDomain::UnionPCurves: {detail}; expected a non-empty array"
        ),
        "",
    )
}

impl ShapeUpgradeUnifySameDomain {
    /// `UnionPCurves(theChain, theEdge)` (protected): the p-curves of the merged edge
    /// built from those of the chain; `unifier.union_pcurves(&chain, &edge)?`.
    pub fn union_pcurves(&self, chain: &[Shape], edge: &Edge) -> Result<(), Failure> {
        self.state.read().union_pcurves(chain, edge)
    }
}
