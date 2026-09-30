// Rust translation of BRepMAT2d_Explorer::Add(Spine, aFace, aNewFace) and the
// file-static MakeEdge, GetCurveType and AdjustCurveEnd of BRepMAT2d_Explorer.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The contour of one wire: the trimmed p-curves of its edges, the gaps above 1e-8
//! between consecutive curves closed, and the wire rebuilt from the new edges.
use super::explorer::{BRepMAT2dContour, ModifiedShapes};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gc::MakeSegment2d;
use rustycad_tkgeombase::geom2d_convert;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point2;

/// `aTolConf`: the gap closed between two curves (1e-8, "in accordance with the
/// further algo", instead of `Precision::Confusion()`).
const GAP_TOLERANCE: f64 = 1.0e-8;

/// What `Add` leaves after the curves: the wire to add to the new face and the
/// modified shapes (each edge and the spine) when a gap was closed.
pub(super) struct WireResult {
    pub(super) wire: Wire,
    pub(super) modified: Vec<(Shape, Shape)>,
}

/// The state of `Add` over one wire: the contour being filled, the new face, the edge
/// map (`anOldNewE`), the first and previous edges and curves and the ends met.
struct WireGaps<'a> {
    contour: &'a mut BRepMAT2dContour,
    face: &'a Face,
    new_face: &'a Face,
    old_new: ModifiedShapes,
    is_modif: bool,
    first_edge: Edge,
    prev_edge: Edge,
    first_curve: Handle<Geom2dTrimmedCurve>,
    p_first: Point2,
    p_last: Point2,
}

/// The wire explorer of `Add` on the face, positioned on the first edge, or `None`
/// for a wire without edges (no contour is created then).
pub(super) fn wire_explorer(spine: &Wire, face: &Face) -> Result<Option<WireExplorer>, Failure> {
    let explorer = WireExplorer::with_wire_on_face(spine, face)?;
    Ok(explorer.more().then_some(explorer))
}

/// The rest of `Add` after `NewContour`: fills `contour` with the curves of the wire in
/// explorer order (the native one appends them as it goes), closes the gaps, then
/// builds the new wire.
pub(super) fn fill_contour(
    mut explorer: WireExplorer,
    spine: &Wire,
    closed: bool,
    (face, new_face): (&Face, &Face),
    contour: &mut BRepMAT2dContour,
) -> Result<WireResult, Failure> {
    let first_edge = explorer.current().clone();
    let first_curve = trimmed_pcurve(&first_edge, face)?;
    let mut gaps = WireGaps {
        contour,
        face,
        new_face,
        old_new: ModifiedShapes::default(),
        is_modif: false,
        prev_edge: first_edge.clone(),
        p_first: first_curve.value(first_curve.first_parameter())?,
        p_last: first_curve.value(first_curve.last_parameter())?,
        first_edge,
        first_curve,
    };
    gaps.old_new.add(
        gaps.first_edge.clone().into(),
        gaps.first_edge.clone().into(),
    );
    gaps.contour.append(gaps.first_curve.cast());
    explorer.next()?;
    while explorer.more() {
        gaps.add_next_edge(explorer.current())?;
        explorer.next()?;
    }
    if closed && gaps.p_last.distance(gaps.p_first) > GAP_TOLERANCE {
        gaps.close_wire()?;
    }
    gaps.finish(spine, closed)
}

impl WireGaps<'_> {
    /// One edge after the first: its curve, the gap to the previous curve closed.
    fn add_next_edge(&mut self, edge: &Edge) -> Result<(), Failure> {
        self.old_new.add(edge.clone().into(), edge.clone().into());
        let mut current = trimmed_pcurve(edge, self.face)?;
        let p_cur_first = current.value(current.first_parameter())?;
        if self.p_last.distance(p_cur_first) > GAP_TOLERANCE {
            self.is_modif = true;
            current = self.join(current, edge, self.p_last, p_cur_first)?;
        }
        self.p_last = current.value(current.last_parameter())?;
        self.contour.append(current.cast());
        self.prev_edge = edge.clone();
        Ok(())
    }

    /// The gap before `current`: the curve of the lower (or equal) type moves its end
    /// to the other one: `current` starts at `p_last`, or the previous curve ends at
    /// `p_cur_first`; the moved curve's edge is rebuilt. Returns the current curve.
    fn join(
        &mut self,
        current: Handle<Geom2dTrimmedCurve>,
        edge: &Edge,
        p_last: Point2,
        p_cur_first: Point2,
    ) -> Result<Handle<Geom2dTrimmedCurve>, Failure> {
        let last_index = self.contour.len();
        let previous: Handle<dyn Geom2dCurve> = self.contour.value(last_index)?.clone();
        if curve_type(&current.cast()) as i32 <= curve_type(&previous) as i32 {
            let adjusted = adjust_curve_end(&current.cast(), p_last, true)?;
            self.replace_edge(edge, &adjusted)?;
            return Ok(adjusted);
        }
        let adjusted = adjust_curve_end(&previous, p_cur_first, false)?;
        *self.contour.change_value(last_index)? = adjusted.cast();
        let prev_edge = self.prev_edge.clone();
        self.replace_edge(&prev_edge, &adjusted)?;
        Ok(current)
    }

    /// The closing gap of a closed wire, between the last curve and the first one.
    fn close_wire(&mut self) -> Result<(), Failure> {
        self.is_modif = true;
        let last_index = self.contour.len();
        let previous: Handle<dyn Geom2dCurve> = self.contour.value(last_index)?.clone();
        if curve_type(&self.first_curve.cast()) as i32 <= curve_type(&previous) as i32 {
            self.first_curve = adjust_curve_end(&self.first_curve.cast(), self.p_last, true)?;
            *self.contour.change_value(1)? = self.first_curve.cast();
            let first_edge = self.first_edge.clone();
            let first_curve = self.first_curve.clone();
            return self.replace_edge(&first_edge, &first_curve);
        }
        let p_first = self.first_curve.value(self.first_curve.first_parameter())?;
        let adjusted = adjust_curve_end(&previous, p_first, false)?;
        *self.contour.change_value(last_index)? = adjusted.cast();
        let prev_edge = self.prev_edge.clone();
        self.replace_edge(&prev_edge, &adjusted)
    }

    /// A new edge on the new face for `curve` (reversed back for a REVERSED edge), with
    /// the vertices and the orientation of `old`, recorded as its replacement.
    fn replace_edge(
        &mut self,
        old: &Edge,
        curve: &Handle<Geom2dTrimmedCurve>,
    ) -> Result<(), Failure> {
        let first = topexp::first_vertex(old, false);
        let last = topexp::last_vertex(old, false);
        let pcurve: Handle<dyn Geom2dCurve> = match old.orientation() == Orientation::Forward {
            true => curve.cast(),
            false => curve.reversed()?,
        };
        let mut new_edge = make_edge(&pcurve, self.new_face, &first, &last)?;
        new_edge.set_orientation(old.orientation());
        *self.old_new.change_from_key(old)? = new_edge.into();
        Ok(())
    }

    /// The wire to add: the spine itself, or a new wire of the edges of the map (each
    /// edge once, in first-use order) when a gap was closed, which then records every
    /// edge and the spine as modified.
    fn finish(self, spine: &Wire, closed: bool) -> Result<WireResult, Failure> {
        if !self.is_modif {
            return Ok(WireResult {
                wire: spine.clone(),
                modified: Vec::new(),
            });
        }
        let builder = BRepBuilder::default();
        let mut wire = Wire::default();
        builder.make_wire(&mut wire);
        let mut modified = Vec::with_capacity(self.old_new.extent() + 1);
        for i in 1..=self.old_new.extent() {
            let (key, new_edge) = (self.old_new.find_key(i)?, self.old_new.find_from_index(i)?);
            builder.add(&mut wire, new_edge)?;
            modified.push((key.clone(), new_edge.clone()));
        }
        if closed {
            wire.set_closed(true)?;
        }
        modified.push((spine.clone().into(), wire.clone().into()));
        Ok(WireResult { wire, modified })
    }
}

/// `new Geom2d_TrimmedCurve(C2d, UFirst, ULast)` of the p-curve of `edge` on `face`,
/// reversed for a REVERSED edge (Standard_ConstructionError without a p-curve).
fn trimmed_pcurve(edge: &Edge, face: &Face) -> Result<Handle<Geom2dTrimmedCurve>, Failure> {
    let pcurve = BRepTool::curve_on_face(edge, face)?;
    let trimmed = Handle::new(Geom2dTrimmedCurve::new(
        &pcurve.pcurve,
        pcurve.first,
        pcurve.last,
        true,
        true,
    )?);
    if edge.orientation() == Orientation::Reversed {
        trimmed.reverse()?;
    }
    Ok(trimmed)
}

/// `MakeEdge`: a new edge with the p-curve on the face (tolerance
/// `Precision::Confusion()`), the two vertices (FORWARD then REVERSED) and the
/// parameter range of the curve.
fn make_edge(
    curve: &Handle<dyn Geom2dCurve>,
    face: &Face,
    first: &Vertex,
    last: &Vertex,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let (f_par, l_par) = (curve.first_parameter(), curve.last_parameter());
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    builder.update_edge_pcurve_on_face(&edge, curve, face, precision::confusion())?;
    builder.add(&mut edge, &first.oriented(Orientation::Forward))?;
    builder.add(&mut edge, &last.oriented(Orientation::Reversed))?;
    builder.range(&edge, f_par, l_par, false)?;
    Ok(edge)
}

/// `GetCurveType`: the kind of the curve, or of the basis of a trimmed curve, by its
/// exact dynamic type (`GeomAbs_OtherCurve` for any other type).
pub(super) fn curve_type(curve: &Handle<dyn Geom2dCurve>) -> CurveType {
    let trimmed: Handle<Geom2dTrimmedCurve> = curve.cast();
    let target: Handle<dyn Geom2dCurve> = match trimmed.get() {
        Some(inner) if curve.is_instance_name("Geom2d_TrimmedCurve") => inner.basis_curve(),
        _ => curve.clone(),
    };
    const KINDS: [(&str, CurveType); 8] = [
        ("Geom2d_Circle", CurveType::Circle),
        ("Geom2d_Line", CurveType::Line),
        ("Geom2d_Ellipse", CurveType::Ellipse),
        ("Geom2d_Parabola", CurveType::Parabola),
        ("Geom2d_Hyperbola", CurveType::Hyperbola),
        ("Geom2d_BezierCurve", CurveType::BezierCurve),
        ("Geom2d_BSplineCurve", CurveType::BSplineCurve),
        ("Geom2d_OffsetCurve", CurveType::OffsetCurve),
    ];
    KINDS
        .iter()
        .find(|(name, _)| target.is_instance_name(name))
        .map_or(CurveType::OtherCurve, |(_, kind)| *kind)
}

/// `AdjustCurveEnd`: a line becomes the segment from `point` to its last point (or
/// from its first point to `point`); any other curve becomes its quasi-angular
/// B-spline with the first (or last) pole moved to `point`, trimmed to its range.
fn adjust_curve_end(
    curve: &Handle<dyn Geom2dCurve>,
    point: Point2,
    is_first: bool,
) -> Result<Handle<Geom2dTrimmedCurve>, Failure> {
    let bounded = curve.get().ok_or_else(null_curve)?;
    if curve_type(curve) == CurveType::Line {
        if is_first {
            let end = bounded.value(bounded.last_parameter())?;
            return MakeSegment2d::from_points(point, end)?.value();
        }
        let start = bounded.value(bounded.first_parameter())?;
        return MakeSegment2d::from_points(start, point)?.value();
    }
    let bspline =
        geom2d_convert::curve_to_bspline_curve(curve, ParameterisationType::QuasiAngular)?;
    let index: i32 = if is_first { 1 } else { bspline.nb_poles() };
    bspline.set_pole(index, point)?;
    let basis: Handle<dyn Geom2dCurve> = bspline.cast();
    let (first, last) = (bspline.first_parameter(), bspline.last_parameter());
    Ok(Handle::new(Geom2dTrimmedCurve::new(
        &basis, first, last, true, true,
    )?))
}

/// The curves of a contour are never null natively (they are dereferenced unchecked).
fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepMAT2d_Explorer: null contour curve; expected a trimmed 2d curve",
        "",
    )
}
