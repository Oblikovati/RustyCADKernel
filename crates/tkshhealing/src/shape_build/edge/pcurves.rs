// Rust translation of ShapeBuild_Edge::RemovePCurve, ReplacePCurve, ReassignPCurve,
// TransformPCurve and the static CountPCurves of ShapeBuild_Edge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Removing, replacing, reassigning and transforming the p-curves of an edge.
use super::{ShapeBuildEdge, copied_pcurve, representations};
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom2d_convert::{ApproxCurve, curve_to_bspline_curve};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib::line_parameter_2d;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{
    Axis2d, Direction2, GeneralTransform2, Point2, Transform2, TransformForm, Vector2, oy2d,
};
use rustycad_tkmath::toploc::Location;

/// A null p-curve handle (`occ::handle<Geom2d_Curve> c2dNull`).
fn null_pcurve() -> Handle<dyn Geom2dCurve> {
    Handle::default()
}

impl ShapeBuildEdge {
    /// `RemovePCurve(edge, face)`: removes the p-curve (both of a seam) of the edge on the
    /// face; `ShapeBuildEdge.remove_pcurve_on_face(&e, &f)?`.
    pub fn remove_pcurve_on_face(&self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let null = null_pcurve();
        //: S4136: the tolerance passed is 0 (the edge tolerance was used before)
        if BRepTool::is_closed_on_face(edge, face)? {
            return builder.update_edge_pcurves_on_face(edge, (&null, &null), face, 0.0);
        }
        builder.update_edge_pcurve_on_face(edge, &null, face, 0.0)
    }

    /// `RemovePCurve(edge, surf)`: on the surface at the identity location;
    /// `ShapeBuildEdge.remove_pcurve_on_surface(&e, &s)?`.
    pub fn remove_pcurve_on_surface(
        &self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        self.remove_pcurve_on_located_surface(edge, surface, &Location::default())
    }

    /// `RemovePCurve(edge, surf, loc)`: removes the p-curve (both of a seam) on the
    /// located surface; `ShapeBuildEdge.remove_pcurve_on_located_surface(&e, &s, &l)?`.
    pub fn remove_pcurve_on_located_surface(
        &self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let null = null_pcurve();
        if BRepTool::is_closed_on_surface(edge, surface, location)? {
            return builder.update_edge_pcurves(edge, (&null, &null), surface, location, 0.0, None);
        }
        builder.update_edge_pcurve(edge, &null, surface, location, 0.0, None)
    }

    /// `ReplacePCurve(edge, pcurve, face)`: sets `pcurve` as the p-curve of the edge on
    /// the face; on a seam (two p-curves on the face taken FORWARD) only the one of the
    /// edge's orientation is replaced. The range on the face is then set to the range of
    /// the p-curve of the reversed edge (the last one read); `ShapeBuildEdge.replace_pcurve(&e, &c, &f)?`.
    pub fn replace_pcurve(
        &self,
        edge: &Edge,
        pcurve: &Handle<dyn Geom2dCurve>,
        face: &Face,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let edge_rev = Edge::unchecked(edge.reversed());
        // reverse face to take second pcurve for seams like SA_Edge::PCurve() does
        let forward = Face::unchecked(face.oriented(Orientation::Forward));
        let pcurve0 = BRepTool::curve_on_face(edge, &forward)?.pcurve;
        let other = BRepTool::curve_on_face(&edge_rev, &forward)?;
        if other.pcurve.is_null() || other.pcurve == pcurve0 {
            builder.update_edge_pcurve_on_face(edge, pcurve, face, 0.0)?;
        } else if edge.orientation() == Orientation::Forward {
            builder.update_edge_pcurves_on_face(edge, (pcurve, &other.pcurve), face, 0.0)?;
        } else {
            builder.update_edge_pcurves_on_face(edge, (&other.pcurve, pcurve), face, 0.0)?;
        }
        builder.range_on_face(edge, face, other.first, other.last)
    }
}

/// The static `CountPCurves(edge, face)`: the number of p-curves STORED in the edge for
/// the face (a plane face may return one that is not stored): 2 for a seam, 1, or 0.
fn count_pcurves(edge: &Edge, face: &Face) -> Result<i32, Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let relative = location.predivided(edge.location())?;
    for representation in representations(edge)? {
        let gc = as_gcurve(&representation);
        let Some(gc) = gc.get() else { continue };
        if gc.is_curve_on_surface_at(&surface, &relative) {
            return Ok(if gc.is_curve_on_closed_surface() {
                2
            } else {
                1
            });
        }
    }
    Ok(0)
}

impl ShapeBuildEdge {
    /// `ReassignPCurve(edge, old, sub)`: moves the p-curve of the edge on face `old` to
    /// face `sub` (one of the two of a seam, the other one stays on `old`; on `sub` it
    /// becomes the first of a seam when `sub` has one already). False when the edge has
    /// no p-curve on `old`; `ShapeBuildEdge.reassign_pcurve(&e, &old, &sub)?`.
    pub fn reassign_pcurve(&self, edge: &Edge, old: &Face, sub: &Face) -> Result<bool, Failure> {
        let mut npcurves = count_pcurves(edge, old)?;
        let found = BRepTool::curve_on_face(edge, old)?;
        if found.pcurve.is_null() {
            return Ok(false);
        }
        if npcurves == 0 {
            npcurves = 1; // gka
        }
        let builder = BRepBuilder::default();
        let (mut first, mut last) = (found.first, found.last);
        if npcurves > 1 {
            let other = BRepTool::curve_on_face(&Edge::unchecked(edge.reversed()), old)?;
            (first, last) = (other.first, other.last);
            builder.update_edge_pcurve_on_face(edge, &other.pcurve, old, 0.0)?;
            builder.range_on_face(edge, old, first, last)?;
        } else {
            self.remove_pcurve_on_face(edge, old)?;
        }
        self.add_pcurve_as_first(edge, &found.pcurve, sub)?;
        builder.range_on_face(edge, sub, first, last)?;
        Ok(true)
    }

    /// The end of `ReassignPCurve`: `pc` alone on `sub`, or as the first p-curve of a seam
    /// with the one `sub` has (`B.UpdateEdge` does not check the edge orientation).
    fn add_pcurve_as_first(
        &self,
        edge: &Edge,
        pc: &Handle<dyn Geom2dCurve>,
        sub: &Face,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        if count_pcurves(edge, sub)? < 1 {
            return builder.update_edge_pcurve_on_face(edge, pc, sub, 0.0);
        }
        let pcs = BRepTool::curve_on_face(&Edge::unchecked(edge.reversed()), sub)?.pcurve;
        if edge.orientation() == Orientation::Reversed {
            return builder.update_edge_pcurves_on_face(edge, (&pcs, pc), sub, 0.0);
        }
        builder.update_edge_pcurves_on_face(edge, (pc, &pcs), sub, 0.0)
    }

    /// `TransformPCurve(pcurve, trans, uFact, aFirst, aLast)`: a copy of the p-curve
    /// transformed by `trans` (the range follows), then scaled by `u_fact` along U (the
    /// affinity of axis OY): a line stays a line through the scaled ends of the range, a
    /// Bezier curve has its poles scaled, a conic is approximated (C1, 100 segments, degree
    /// 6) on its range or converted, and any other curve converted to a B-spline whose
    /// poles are scaled. Returns the curve and the new range;
    /// `let (c, f, l) = ShapeBuildEdge.transform_pcurve(&pc, t, 2.0, 0.0, 1.0)?;`.
    pub fn transform_pcurve(
        &self,
        pcurve: &Handle<dyn Geom2dCurve>,
        trans: Transform2,
        u_fact: f64,
        first: f64,
        last: f64,
    ) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
        let mut result = copied_pcurve(pcurve)?;
        let (mut first, mut last) = (first, last);
        if trans.form() != TransformForm::Identity {
            result.transform(trans)?;
            first = result.transformed_parameter(first, trans);
            last = result.transformed_parameter(last, trans);
        }
        if u_fact == 1.0 {
            return Ok((result, first, last));
        }
        if let Some(trimmed) = result.cast::<Geom2dTrimmedCurve>().get() {
            result = trimmed.basis_curve();
        }
        let mut affinity = GeneralTransform2::default();
        affinity.set_affinity(oy2d(), u_fact);
        scaled_along_u(&result, &affinity, first, last)
    }
}

/// The affinity applied to a point.
fn scaled(affinity: &GeneralTransform2, point: Point2) -> Point2 {
    let mut xy = point.xy();
    affinity.transforms(&mut xy);
    Point2::from(xy)
}

/// The kinds of `TransformPCurve` once the curve is transformed: a line, a Bezier curve,
/// or a curve converted to a B-spline.
fn scaled_along_u(
    curve: &Handle<dyn Geom2dCurve>,
    affinity: &GeneralTransform2,
    first: f64,
    last: f64,
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    if let Some(line) = curve.cast::<Geom2dLine>().get() {
        return scaled_line(line, affinity, first, last);
    }
    if let Some(bezier) = curve.cast::<Geom2dBezierCurve>().get() {
        for i in 1..=bezier.nb_poles() {
            bezier.set_pole(i, scaled(affinity, bezier.pole(i)?))?;
        }
        return Ok((curve.clone(), first, last));
    }
    let (bspline, first, last) = as_bspline(curve, first, last)?;
    for i in 1..=bspline.nb_poles() {
        bspline.set_pole(i, scaled(affinity, bspline.pole(i)?))?;
    }
    Ok((bspline.cast(), first, last))
}

/// A line through the scaled points of the range, parameterized from them.
fn scaled_line(
    line: &Geom2dLine,
    affinity: &GeneralTransform2,
    first: f64,
    last: f64,
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let pf = scaled(affinity, line.d0(first)?);
    let pl = scaled(affinity, line.d0(last)?);
    let axis = Axis2d::new(pf, Direction2::from_vector(Vector2::between(pf, pl))?);
    let (first, last) = (line_parameter_2d(axis, pf), line_parameter_2d(axis, pl));
    Ok((Handle::new(Geom2dLine::new(axis)).cast(), first, last))
}

/// The B-spline of `TransformPCurve`: a conic approximated on its range (the range
/// becomes the B-spline's), another curve converted with the quasi-angular
/// parameterisation, a B-spline as is.
fn as_bspline(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(Handle<Geom2dBSplineCurve>, f64, f64), Failure> {
    if curve.is_kind_name("Geom2d_Conic") {
        // protection against parabols ets
        let trimmed: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast();
        let approx =
            ApproxCurve::from_curve(&trimmed, precision::approximation(), Shape::C1, 100, 6)?;
        let bspline = if approx.has_result() {
            approx.curve()
        } else {
            curve_to_bspline_curve(&trimmed, ParameterisationType::QuasiAngular)?
        };
        let (f, l) = (bspline.first_parameter(), bspline.last_parameter());
        return Ok((bspline, f, l));
    }
    let bspline = curve.cast::<Geom2dBSplineCurve>();
    if bspline.is_some() {
        return Ok((bspline, first, last));
    }
    Ok((
        curve_to_bspline_curve(curve, ParameterisationType::QuasiAngular)?,
        first,
        last,
    ))
}
