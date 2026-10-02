// Rust translation of ModelingAlgorithms/TKShHealing/ShapeBuild/ShapeBuild_Edge.hxx and the
// members CopyReplaceVertices, CopyRanges, SetRange3d, CopyPCurves and Copy with the static
// AdjustByPeriod and IsPeriodic of ShapeBuild_Edge.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeBuild_Edge`: copying edges with other vertices, ranges or p-curves
//! (docs/shhealing-design.md section 3).
mod make;
mod pcurves;

use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepCurveRepresentation, BRepGCurve};
use rustycad_tkbrep::topods::{self, Edge, Face, ShapeIterator, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCurve, Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{GeomCurve, GeomOffsetCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// `ShapeBuild_Edge`: low-level operators to build and copy edges, their 3D curves and
/// their p-curves.
///
/// ```
/// use rustycad_tkbrep::topods::Edge;
/// use rustycad_tkshhealing::shape_build::ShapeBuildEdge;
/// assert!(ShapeBuildEdge.copy(&Edge::default(), true).is_err());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeBuildEdge;

/// The curve representations of an edge (`TEdge->ChangeCurves()`), in order.
pub(crate) fn representations(
    edge: &Edge,
) -> Result<Vec<Handle<dyn BRepCurveRepresentation>>, Failure> {
    Ok(edge_tshape(edge, "ShapeBuild_Edge")?.curves())
}

/// The static `AdjustByPeriod` of ShapeBuild_Edge.cxx ("added, cause invoke ShapeAnalysis
/// leads to cyclic dependency"): the multiple of `period` that brings `val` nearest to
/// `to_val`, 0 within half a period.
pub(crate) fn adjust_by_period(val: f64, to_val: f64, period: f64) -> f64 {
    let diff = val - to_val;
    let d = diff.abs();
    let p = period.abs();
    if d <= 0.5 * p {
        return 0.0;
    }
    if p < 1e-100 {
        return diff;
    }
    (if diff > 0.0 { -p } else { p }) * f64::from((d / p + 0.5) as i32)
}

/// The static `IsPeriodic(Geom_Curve)` (15.11.2002 PTV OCC966): the periodicity of the
/// basis curve under any offset and trimmed curves.
pub(crate) fn is_periodic_3d(curve: &Handle<dyn GeomCurve>) -> bool {
    let mut current = curve.clone();
    loop {
        if let Some(offset) = current.cast::<GeomOffsetCurve>().get() {
            current = offset.basis_curve();
        } else if let Some(trimmed) = current.cast::<GeomTrimmedCurve>().get() {
            current = trimmed.basis_curve();
        } else {
            return current.is_periodic();
        }
    }
}

/// The `IsPeriodic(Geom2d_Curve)` of ShapeBuild_Edge.cxx, the same for a p-curve.
pub(crate) fn is_periodic_2d(curve: &Handle<dyn Geom2dCurve>) -> bool {
    let mut current = curve.clone();
    loop {
        if let Some(offset) = current.cast::<Geom2dOffsetCurve>().get() {
            current = offset.basis_curve();
        } else if let Some(trimmed) = current.cast::<Geom2dTrimmedCurve>().get() {
            current = trimmed.basis_curve();
        } else {
            return current.is_periodic();
        }
    }
}

impl ShapeBuildEdge {
    /// `CopyReplaceVertices(edge, V1, V2)`: a copy of the edge (a new TEdge with the same
    /// representations) whose FORWARD vertex is `v1` and REVERSED vertex `v2`; a null
    /// vertex keeps the vertex of the edge, and when both are null the INTERNAL and
    /// EXTERNAL vertices are kept too. The ranges are copied (S4054: 3D and p-curve ranges
    /// may differ); `ShapeBuildEdge.copy_replace_vertices(&e, &v1, &v2)?`.
    pub fn copy_replace_vertices(
        &self,
        edge: &Edge,
        v1: &Vertex,
        v2: &Vertex,
    ) -> Result<Edge, Failure> {
        let (mut new_v1, mut new_v2) = (v1.clone(), v2.clone());
        let mut non_manifold = Vec::new();
        if new_v1.is_null() || new_v2.is_null() {
            let both_null = v1.is_null() && v2.is_null();
            collect_vertices(
                edge,
                &mut new_v1,
                &mut new_v2,
                both_null.then_some(&mut non_manifold),
            )?;
        }
        new_v1.set_orientation(Orientation::Forward);
        new_v2.set_orientation(Orientation::Reversed);
        let mut copy = topods::edge(&edge.empty_copied()?)?;
        let builder = BRepBuilder::default();
        for vertex in [&new_v1, &new_v2].into_iter().filter(|v| !v.is_null()) {
            builder.add(&mut copy, vertex)?;
        }
        for vertex in &non_manifold {
            builder.add(&mut copy, vertex)?;
        }
        self.copy_ranges(&copy, edge, 0.0, 1.0)?;
        Ok(copy)
    }

    /// Copies an edge with replacement vertices and a p-curve range on its face;
    /// `ShapeBuildEdge.copy_replace_vertices_with_pcurve_range(&e,&v1,&v2,&pc,&f,(0.2,0.8))?`.
    pub fn copy_replace_vertices_with_pcurve_range(
        &self,
        edge: &Edge,
        first_vertex: &Vertex,
        last_vertex: &Vertex,
        pcurve: &Handle<dyn Geom2dCurve>,
        face: &Face,
        range: (f64, f64),
    ) -> Result<Edge, Failure> {
        let copy = self.copy_replace_vertices(edge, first_vertex, last_vertex)?;
        set_face_pcurve_range(&copy, pcurve, face, range)?;
        Ok(copy)
    }

    /// Splits a forward edge at a shared vertex while preserving its curve representations.
    /// The parameter uses the face p-curve range; `edge.split_at_parameter(&e, 0.5, &v, &f)?`.
    pub fn split_at_parameter(
        &self,
        edge: &Edge,
        face: &Face,
        parameter: f64,
        vertex: &Vertex,
    ) -> Result<(Edge, Edge), Failure> {
        let orientation = edge.orientation();
        let mut forward = edge.clone();
        forward.set_orientation(Orientation::Forward);
        let (first_vertex, last_vertex) = rustycad_tkbrep::topexp::vertices(&forward, false);
        let (first, last) = rustycad_tkbrep::brep::BRepTool::range_on_face(&forward, face)?;
        let fraction = (parameter - first) / (last - first);
        if !matches!(orientation, Orientation::Forward | Orientation::Reversed)
            || !(0.0 < fraction && fraction < 1.0)
        {
            return Err(invalid_split_parameter(edge, parameter, (first, last)));
        }
        let mut first_half = self.copy_replace_vertices(&forward, &first_vertex, vertex)?;
        let mut second_half = self.copy_replace_vertices(&forward, vertex, &last_vertex)?;
        self.copy_ranges(&first_half, &forward, 0.0, fraction)?;
        self.copy_ranges(&second_half, &forward, fraction, 1.0)?;
        if orientation == Orientation::Reversed {
            first_half.set_orientation(Orientation::Reversed);
            second_half.set_orientation(Orientation::Reversed);
            std::mem::swap(&mut first_half, &mut second_half);
        }
        BRepBuilder::default().same_range(&first_half, false)?;
        BRepBuilder::default().same_parameter(&first_half, false)?;
        BRepBuilder::default().same_range(&second_half, false)?;
        BRepBuilder::default().same_parameter(&second_half, false)?;
        Ok((first_half, second_half))
    }

    /// Copies a B-spline p-curve and moves its endpoint poles;
    /// `ShapeBuildEdge.copy_bspline_pcurve_with_endpoints(&pc, point, point)?`.
    pub fn copy_bspline_pcurve_with_endpoints(
        &self,
        pcurve: &Handle<dyn Geom2dCurve>,
        first: rustycad_tkmath::gp::Point2,
        last: rustycad_tkmath::gp::Point2,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let source = pcurve.cast::<Geom2dBSplineCurve>();
        let source = source
            .get()
            .ok_or_else(|| unsupported_pcurve_copy(pcurve))?;
        let copy = Geom2dBSplineCurve::copy_of(source)?;
        copy.set_pole(1, first)?;
        copy.set_pole(copy.nb_poles(), last)?;
        Ok(Handle::new(copy).cast())
    }
}

/// Replaces the p-curve representation and clears SameRange when its range changes.
fn set_face_pcurve_range(
    edge: &Edge,
    pcurve: &Handle<dyn Geom2dCurve>,
    face: &Face,
    (first, last): (f64, f64),
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.update_edge_pcurve_on_face(edge, pcurve, face, 0.0)?;
    builder.range_on_face(edge, face, first, last)?;
    builder.same_range(edge, false)
}

/// Reports the precise curve kind required by the endpoint-pole operation.
fn unsupported_pcurve_copy(pcurve: &Handle<dyn Geom2dCurve>) -> Failure {
    let found = pcurve.dynamic_type().map_or("null", |kind| kind.name());
    Failure::with_message(
        "Standard_NotImplemented",
        &format!(
            "ShapeBuild_Edge pcurve endpoint copy requires Geom2d_BSplineCurve; found {found}"
        ),
    )
}

fn invalid_split_parameter(edge: &Edge, parameter: f64, range: (f64, f64)) -> Failure {
    Failure::with_message(
        "Standard_DomainError",
        &format!(
            "ShapeBuild_Edge::SplitAtParameter: edge orientation {:?}, parameter {parameter}; expected a forward or reversed edge parameter inside {:?}",
            edge.orientation(),
            range
        ),
    )
}

/// The vertex loop of `CopyReplaceVertices`: fills the null vertices with the first
/// FORWARD and REVERSED vertices of the edge (iterated with its orientation when FORWARD or
/// REVERSED), and collects the other vertices when asked.
fn collect_vertices(
    edge: &Edge,
    first: &mut Vertex,
    last: &mut Vertex,
    mut non_manifold: Option<&mut Vec<Vertex>>,
) -> Result<(), Failure> {
    let cum_ori = matches!(
        edge.orientation(),
        Orientation::Forward | Orientation::Reversed
    );
    let mut it = ShapeIterator::new(edge, cum_ori, true);
    while it.more() {
        let vertex = topods::vertex(it.value()?)?;
        it.next();
        match vertex.orientation() {
            Orientation::Forward if first.is_null() => *first = vertex,
            Orientation::Reversed if last.is_null() => *last = vertex,
            Orientation::Forward | Orientation::Reversed => {}
            _ => {
                if let Some(list) = non_manifold.as_deref_mut() {
                    list.push(vertex);
                }
            }
        }
    }
    Ok(())
}

impl ShapeBuildEdge {
    /// `CopyRanges(toedge, fromedge, alpha, beta)`: for each 3D curve and p-curve of
    /// `from`, the range `[first + alpha * len, first + beta * len]` goes to the first
    /// representation of the same kind (same surface and location for a p-curve) of `to`;
    /// on a periodic curve a range starting before the curve (beyond `PConfusion`) or at
    /// or after its end is shifted by periods (PTV 22.03.2002) and the edge loses its
    /// SameRange and SameParameter flags; `ShapeBuildEdge.copy_ranges(&to, &from, 0.0, 1.0)?`.
    pub fn copy_ranges(
        &self,
        to: &Edge,
        from: &Edge,
        alpha: f64,
        beta: f64,
    ) -> Result<(), Failure> {
        for representation in representations(from)? {
            let from_gc = as_gcurve(&representation);
            let Some(gc) = from_gc.get() else { continue };
            let Some(target) = CopyTarget::of(gc)? else {
                continue;
            };
            let to_list = representations(to)?;
            let found = to_list
                .iter()
                .map(as_gcurve)
                .find(|to_gc| target.matches(to_gc));
            if let Some(to_gc) = found {
                copy_range(to, gc, &to_gc, alpha, beta)?;
            }
        }
        Ok(())
    }
}

/// What a representation of `from` looks for in `to`: a 3D curve, or a p-curve on the
/// same surface and location.
enum CopyTarget {
    Curve3d,
    PCurve(Handle<dyn GeomSurface>, Location),
}

impl CopyTarget {
    /// `None` for a representation without a curve, or that is neither a 3D curve nor a
    /// curve on surface ("only 3d curves and pcurves are treated").
    fn of(gc: &dyn BRepGCurve) -> Result<Option<Self>, Failure> {
        if gc.is_curve3d() {
            return Ok((!gc.curve3d()?.is_null()).then_some(Self::Curve3d));
        }
        if gc.pcurve()?.is_null() || !gc.is_curve_on_surface() {
            return Ok(None);
        }
        Ok(Some(Self::PCurve(gc.surface()?, gc.location())))
    }

    fn matches(&self, to_gc: &Handle<dyn BRepGCurve>) -> bool {
        let Some(gc) = to_gc.get() else { return false };
        match self {
            Self::Curve3d => gc.is_curve3d(),
            Self::PCurve(surface, location) => {
                gc.is_curve_on_surface()
                    && gc.surface().is_ok_and(|s| &s == surface)
                    && gc.location() == *location
            }
        }
    }
}

/// The body of the `CopyRanges` inner loop for the found representation.
fn copy_range(
    to: &Edge,
    from_gc: &dyn BRepGCurve,
    to_gc: &Handle<dyn BRepGCurve>,
    alpha: f64,
    beta: f64,
) -> Result<(), Failure> {
    let (first, last) = (from_gc.first(), from_gc.last());
    let len = last - first;
    let (mut new_f, mut new_l) = (first + alpha * len, first + beta * len);
    if let Some((period, crv_f, crv_l)) = periodic_range(to_gc)? {
        let before = (new_f - crv_f).abs() > precision::p_confusion() && new_f < crv_f;
        if before || new_f >= crv_l {
            let shift = adjust_by_period(new_f, 0.5 * (crv_f + crv_l), period);
            new_f += shift;
            new_l += shift;
            BRepBuilder::default().same_range(to, false)?;
            BRepBuilder::default().same_parameter(to, false)?;
        }
    }
    to_gc.set_range(new_f, new_l)
}

/// The period and the parameter range of the curve of a `BRep_Curve3D` or
/// `BRep_CurveOnSurface` whose (basis) curve is periodic; `None` otherwise.
fn periodic_range(gc: &Handle<dyn BRepGCurve>) -> Result<Option<(f64, f64, f64)>, Failure> {
    if gc.is_kind_name("BRep_Curve3D") {
        let curve = gc.curve3d()?;
        if curve.is_null() || !is_periodic_3d(&curve) {
            return Ok(None);
        }
        return Ok(Some((
            curve.period()?,
            curve.first_parameter(),
            curve.last_parameter(),
        )));
    }
    if gc.is_kind_name("BRep_CurveOnSurface") {
        let curve = gc.pcurve()?;
        if curve.is_null() || !is_periodic_2d(&curve) {
            return Ok(None);
        }
        return Ok(Some((
            curve.period()?,
            curve.first_parameter(),
            curve.last_parameter(),
        )));
    }
    Ok(None)
}

impl ShapeBuildEdge {
    /// `SetRange3d(edge, first, last)`: the range of the first 3D curve representation only;
    /// `ShapeBuildEdge.set_range_3d(&e, 0.0, 1.0)?`.
    pub fn set_range_3d(&self, edge: &Edge, first: f64, last: f64) -> Result<(), Failure> {
        let found = representations(edge)?
            .iter()
            .map(as_gcurve)
            .find(|gc| gc.get().is_some_and(|gc| gc.is_curve3d()));
        match found {
            Some(gc) => gc.set_range(first, last),
            None => Ok(()),
        }
    }

    /// `Copy(edge, sharepcurves)`: [`Self::copy_replace_vertices`] with the vertices of the
    /// edge; unless `share_pcurves`, the p-curves are replaced by copies
    /// ([`Self::copy_pcurves`]); `ShapeBuildEdge.copy(&e, true)?`.
    pub fn copy(&self, edge: &Edge, share_pcurves: bool) -> Result<Edge, Failure> {
        let copy = self.copy_replace_vertices(edge, &Vertex::default(), &Vertex::default())?;
        if !share_pcurves {
            self.copy_pcurves(&copy, edge)?;
        }
        Ok(copy)
    }

    /// `CopyPCurves(toedge, fromedge)`: each curve on surface of `from` is copied into `to`:
    /// over the representation on the same surface and location when `to` has one, else
    /// into a copy of the representation appended to `to`; the p-curves are copies and the
    /// location becomes `(fromLoc * L).Predivided(toLoc)` (bug OCC209);
    /// `ShapeBuildEdge.copy_pcurves(&to, &from)?`.
    pub fn copy_pcurves(&self, to: &Edge, from: &Edge) -> Result<(), Failure> {
        for representation in representations(from)? {
            let from_gc = as_gcurve(&representation);
            let Some(gc) = from_gc.get() else { continue };
            if gc.is_curve_on_surface() {
                copy_pcurve(to, from, gc)?;
            }
        }
        Ok(())
    }
}

/// One curve on surface of `CopyPCurves`.
fn copy_pcurve(to: &Edge, from: &Edge, from_gc: &dyn BRepGCurve) -> Result<(), Failure> {
    let target = CopyTarget::PCurve(from_gc.surface()?, from_gc.location());
    let existing = representations(to)?
        .iter()
        .map(as_gcurve)
        .find(|gc| target.matches(gc));
    let to_gc = match existing {
        Some(gc) => gc,
        None => {
            let copy = from_gc.copy()?;
            edge_tshape(to, "ShapeBuild_Edge::CopyPCurves")?
                .change_curves(|list| list.push(copy.clone()));
            as_gcurve(&copy)
        }
    };
    to_gc.set_pcurve(&copied_pcurve(&from_gc.pcurve()?)?)?;
    let location = from
        .location()
        .multiplied(&from_gc.location())?
        .predivided(to.location())?;
    to_gc.set_location(&location);
    if from_gc.is_curve_on_closed_surface() {
        to_gc.set_pcurve2(&copied_pcurve(&from_gc.pcurve2()?)?)?;
    }
    Ok(())
}

/// `occ::down_cast<Geom2d_Curve>(pcurve->Copy())`; Standard_NullObject for a null p-curve.
pub(crate) fn copied_pcurve(
    pcurve: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let Some(curve) = pcurve.get() else {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "ShapeBuild_Edge: null p-curve to copy; expected a Geom2d_Curve",
        ));
    };
    Ok(curve.copy()?.cast())
}
