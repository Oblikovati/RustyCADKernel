//! The new edges of the curves of the structure (TopOpeBRepBuild_BuildEdges.cxx): the
//! curves coinciding in 2d merged into equivalence groups sharing their edges and end
//! vertices, then each curve split by its points.
use super::TopOpeBRepBuildBuilder;
use super::build::null_hds;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildEdgeBuilder, TopOpeBRepBuildLoopSet, TopOpeBRepBuildPaveClassifier,
    TopOpeBRepBuildPaveSet,
};
use crate::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSCurveExplorer, TopOpeBRepDSHDataStructure, TopOpeBRepDSKind,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::geom2d_int::GInter;

/// `TopOpeBRepBuild_CurveRange(theCurve, thePCurve, theFirst, theLast)`: the valid
/// (finite, increasing) range of the curve, else of the p-curve, else of the 3d curve.
fn curve_range(c: &TopOpeBRepDSCurve, pc: Option<&Handle<dyn Geom2dCurve>>) -> Option<(f64, f64)> {
    let valid =
        |(f, l): (f64, f64)| f < l && !precision::is_infinite(f) && !precision::is_infinite(l);
    if let Some(r) = c.range().filter(|r| valid(*r)) {
        return Some(r);
    }
    if let Some(p) = pc.and_then(|p| p.get()) {
        let r = (p.first_parameter(), p.last_parameter());
        if valid(r) {
            return Some(r);
        }
    }
    let c3d = c.curve().get()?;
    Some((c3d.first_parameter(), c3d.last_parameter())).filter(|r| valid(*r))
}

/// `TopOpeBRepBuild_HasCompleteCoincidence(...)`: one segment covering both curves;
/// whether it reverses one relative to the other.
fn has_complete_coincidence(
    inter: &GInter,
    c1: &Geom2dAdaptorCurve,
    c2: &Geom2dAdaptorCurve,
) -> Result<Option<bool>, Failure> {
    if inter.nb_segments()? != 1 {
        return Ok(None);
    }
    let seg = inter.segment(1)?;
    if !seg.has_first_point() || !seg.has_last_point() {
        return Ok(None);
    }
    let (fp, lp) = (seg.first_point()?, seg.last_point()?);
    let (f1, l1, f2, l2) = (
        fp.param_on_first(),
        lp.param_on_first(),
        fp.param_on_second(),
        lp.param_on_second(),
    );
    let tol = precision::p_confusion();
    let complete = (f1.min(l1) - c1.first_parameter()?).abs() <= tol
        && (f1.max(l1) - c1.last_parameter()?).abs() <= tol
        && (f2.min(l2) - c2.first_parameter()?).abs() <= tol
        && (f2.max(l2) - c2.last_parameter()?).abs() <= tol;
    Ok(complete.then_some((l1 - f1) * (l2 - f2) < 0.0))
}

/// `TopOpeBRepBuild_FindCurveEnd(...)`: the point of the curve nearest to its start or
/// end, when within PConfusion (its index and whether it is a point).
fn find_curve_end(
    h: &TopOpeBRepDSHDataStructure,
    ic: i32,
    start: bool,
) -> Result<Option<(i32, bool)>, Failure> {
    let Some((first, last)) = curve_range(&h.curve(ic), None) else {
        return Ok(None);
    };
    let par = if start { first } else { last };
    let (mut best, mut found) = (f64::MAX, (0, false));
    let mut it = h.curve_points(ic)?;
    while it.more() {
        let d = (it.parameter()? - par).abs();
        if d < best {
            (best, found) = (d, (it.current()?, it.is_point()?));
        }
        it.next()?;
    }
    Ok((best <= precision::p_confusion()).then_some(found))
}

/// The indices of the curves of the structure (all, or the kept ones).
pub(crate) fn curve_indices(h: &TopOpeBRepDSHDataStructure, find_keep: bool) -> Vec<i32> {
    let ds = h.ds();
    let mut cex = TopOpeBRepDSCurveExplorer::new(&ds, find_keep);
    let mut v = Vec::new();
    while cex.more() {
        v.push(cex.index());
        cex.next();
    }
    v
}

impl TopOpeBRepBuildBuilder {
    /// The copy (with its vertices) of the existing edge of a curve, shared by the
    /// curves on it, as the new edge of the curve.
    fn existing_edge_copy(&mut self, ic: i32, existing: &Shape) -> Result<(), Failure> {
        if !self.my_coincident_edges.is_bound(existing) {
            let mut copied = Shape::default();
            self.my_build_tool.copy_edge(existing, &mut copied)?;
            for v in Explorer::new(existing, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                self.my_build_tool
                    .add_edge_vertex_from(existing, &mut copied, &v)?;
            }
            self.my_coincident_edges.bind(existing.clone(), copied);
        }
        let an_edge = self
            .my_coincident_edges
            .find(existing)?
            .oriented(Orientation::Forward);
        self.change_new_edges(ic).append(an_edge);
        Ok(())
    }

    /// `BuildEdges(iC, HDS)`: the edges of the curve `ic`: those of its reference
    /// curve, the copy of its existing edge, or its edge split by its points, then
    /// recomputed (moved to a new curve) or updated; `b.build_edges_of(1, &hds)?`.
    pub fn build_edges_of(
        &mut self,
        ic: i32,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
    ) -> Result<(), Failure> {
        let h = hds.get().ok_or_else(null_hds)?;
        let c = h.curve(ic);
        if c.curve().is_null() && c.get_sci1().is_null() && c.get_sci2().is_null() {
            return Ok(());
        }
        let rc = c.equivalent_curve();
        if rc > 0 && rc != ic {
            if self.new_edges(rc).is_empty() {
                self.build_edges_of(rc, hds)?;
            }
            *self.change_new_edges(ic) = self.new_edges(rc).clone();
            return Ok(());
        }
        if !c.existing_edge().is_null() {
            return self.existing_edge_copy(ic, c.existing_edge().as_ref());
        }
        let mut an_edge = Shape::default();
        self.my_build_tool
            .make_edge_on_ds(&mut an_edge, &c, &h.ds())?;
        let mut pvs = TopOpeBRepBuildPaveSet::new(&an_edge)?;
        let mut cpit = h.curve_points(ic)?;
        self.fill_vertex_set(&mut cpit, State::In, &mut pvs)?;
        let mut vcl = TopOpeBRepBuildPaveClassifier::new(&an_edge)?;
        if pvs.has_equal_parameters()? {
            vcl.set_first_parameter(pvs.equal_parameters());
        }
        vcl.closed_vertices(pvs.closed_vertices());
        pvs.init_loop()?;
        if !pvs.more_loop() {
            return Ok(());
        }
        let mut edbu = TopOpeBRepBuildEdgeBuilder::with_paves(&mut pvs, &mut vcl, false)?;
        let mut el = std::mem::take(self.change_new_edges(ic));
        let r = self.make_edges(&an_edge, &mut edbu, &mut el);
        *self.change_new_edges(ic) = el.clone();
        r?;
        self.recompute_new_edges(ic, &c, &an_edge, &mut el, hds)
    }

    /// The new edges of the curve recomputed onto a new curve (the curve then removed),
    /// else updated from the edge of the curve.
    fn recompute_new_edges(
        &mut self,
        ic: i32,
        c: &TopOpeBRepDSCurve,
        an_edge: &Shape,
        el: &mut rustycad_tkernel::ncollection::List<Shape>,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
    ) -> Result<(), Failure> {
        let old = topods::edge(an_edge)?;
        let mut inew_c = -1;
        for e in el.iter_mut() {
            let mut ne = topods::edge(e)?;
            let r = self.my_build_tool.recompute_curves(c, &old, &mut ne, hds)?;
            *e = ne.clone().into_shape();
            if r != 0 {
                inew_c = r;
            }
            if inew_c != -1 {
                self.change_new_edges(inew_c).append(ne.into_shape());
            }
        }
        *self.change_new_edges(ic) = el.clone();
        let h = hds.get().ok_or_else(null_hds)?;
        if inew_c != -1 {
            h.remove_curve(ic);
            return Ok(());
        }
        for e in el.iter() {
            let mut ne = e.clone();
            self.my_build_tool.update_edge(an_edge, &mut ne)?;
        }
        Ok(())
    }

    /// The curves of each surface coinciding completely in 2d merged as equivalent.
    fn merge_surface_curves(h: &TopOpeBRepDSHDataStructure) -> Result<(), Failure> {
        for is in 1..=h.nb_surfaces() {
            let mut scs = Vec::new();
            let mut it = h.surface_curves(is)?;
            while it.more() {
                let pc = it.pcurve()?;
                let c = h.curve(it.current()?);
                if !pc.is_null()
                    && !c.curve().is_null()
                    && let Some((f, l)) = curve_range(&c, Some(&pc))
                {
                    scs.push((it.current()?, pc, f, l));
                }
                it.next()?;
            }
            for i in 0..scs.len() {
                for j in i + 1..scs.len() {
                    let (a, b) = (&scs[i], &scs[j]);
                    let c1 = Geom2dAdaptorCurve::with_range(&a.1, a.2, a.3)?;
                    let c2 = Geom2dAdaptorCurve::with_range(&b.1, b.2, b.3)?;
                    let inter = GInter::between(
                        &c1,
                        &c2,
                        precision::p_confusion(),
                        precision::p_confusion(),
                    )?;
                    if let Some(rev) = has_complete_coincidence(&inter, &c1, &c2)? {
                        h.change_ds().merge_equivalent_curves(a.0, b.0, rev);
                    }
                }
            }
        }
        Ok(())
    }

    /// Each curve of a group bound to the root of its group.
    fn root_equivalent_curves(h: &TopOpeBRepDSHDataStructure) {
        for ic in curve_indices(h, false) {
            if h.curve(ic).equivalent_curve() <= 0 {
                continue;
            }
            let mut ds = h.change_ds();
            let (root, rev) = ds.find_equivalent_curve(ic);
            ds.change_curve(ic).set_equivalent_curve(root, rev);
        }
    }

    /// The vertices of the representative ends of the curve ends, by representative.
    fn equivalent_point_vertices(
        &self,
        h: &TopOpeBRepDSHDataStructure,
    ) -> Result<DataMap<i32, Shape>, Failure> {
        let mut m = DataMap::default();
        for ic in curve_indices(h, false) {
            for start in [true, false] {
                let (rcurve, rstart) = h.ds().find_equivalent_curve_point(ic, start);
                if rcurve == ic && rstart == start {
                    continue;
                }
                let key = 2 * rcurve + if rstart { 0 } else { 1 };
                if m.is_bound(&key) {
                    continue;
                }
                if let Some((ig, is_point)) = find_curve_end(h, rcurve, rstart)? {
                    let v = if is_point {
                        self.new_vertex(ig)?.clone()
                    } else {
                        h.shape(ig, true)
                    };
                    m.bind(key, v);
                }
            }
        }
        Ok(m)
    }

    /// The new vertices of the curve ends set to those of their representatives.
    fn share_end_vertices(
        &mut self,
        h: &TopOpeBRepDSHDataStructure,
        m: &DataMap<i32, Shape>,
    ) -> Result<(), Failure> {
        for ic in curve_indices(h, false) {
            for start in [true, false] {
                let (rcurve, rstart) = h.ds().find_equivalent_curve_point(ic, start);
                let Ok(v) = m.find(&(2 * rcurve + if rstart { 0 } else { 1 })) else {
                    continue;
                };
                if let Some((ig, true)) = find_curve_end(h, ic, start)? {
                    *self.change_new_vertex(ig)? = v.clone();
                }
            }
        }
        Ok(())
    }

    /// The new vertices of the points of the equivalent curves set to those of the
    /// coincident points of their reference curves.
    fn share_point_vertices(&mut self, h: &TopOpeBRepDSHDataStructure) -> Result<(), Failure> {
        for ic in curve_indices(h, false) {
            let rc = h.curve(ic).equivalent_curve();
            if rc <= 0 || rc == ic {
                continue;
            }
            let mut it = h.curve_points(ic)?;
            while it.more() {
                if it.is_point()? {
                    let ip = it.current()?;
                    let p = h.point(ip)?;
                    let mut rit = h.curve_points(rc)?;
                    while rit.more() {
                        if rit.is_point()? {
                            let rp = h.point(rit.current()?)?;
                            if p.point().distance(&rp.point()) <= p.tolerance().max(rp.tolerance())
                            {
                                *self.change_new_vertex(ip)? =
                                    self.new_vertex(rit.current()?)?.clone();
                                break;
                            }
                        }
                        rit.next()?;
                    }
                }
                it.next()?;
            }
        }
        Ok(())
    }

    /// The kept curves after the first kept curve with a mother removed, the curve
    /// count cut before it.
    fn cut_mothered_curves(h: &TopOpeBRepDSHDataStructure) -> Result<(), Failure> {
        let ick = {
            let all = curve_indices(h, false);
            let ds = h.ds();
            all.into_iter().find(|&ic| {
                TopOpeBRepDSCurveExplorer::new(&ds, false).is_curve_keep(ic)
                    && ds.curve(ic).mother() != 0
            })
        };
        if let Some(ick) = ick {
            for ic in curve_indices(h, true) {
                h.change_ds().remove_curve(ic);
            }
            h.change_ds().change_nb_curves(ick - 1);
        }
        Ok(())
    }

    /// The points of the structure referenced by no curve or edge interference removed.
    fn remove_unused_points(h: &TopOpeBRepDSHDataStructure) -> Result<(), Failure> {
        let np = h.nb_points();
        let mut tp = vec![0; usize::try_from(np).unwrap_or(0) + 1];
        let mut bump = |k: i32| -> Result<(), Failure> {
            let slot = usize::try_from(k)
                .ok()
                .and_then(|k| tp.get_mut(k))
                .ok_or_else(|| {
                    Failure::with_type(
                        "Standard_OutOfRange",
                        "BuildEdges: point index out of range",
                        "NCollection_Array1",
                    )
                })?;
            *slot += 1;
            Ok(())
        };
        let mut count = |l: &crate::top_ope_brep_ds::ListOfInterference,
                         gbound: bool|
         -> Result<(), Failure> {
            for i in l.iter() {
                let i = i.get().ok_or_else(null_hds)?;
                if i.geometry_type() == TopOpeBRepDSKind::Point && (!gbound || i.geometry() <= np) {
                    bump(i.geometry())?;
                }
                if i.support_type() == TopOpeBRepDSKind::Point {
                    bump(i.support())?;
                }
            }
            Ok(())
        };
        let kept = curve_indices(h, true);
        let ds = h.ds();
        for ic in kept {
            count(ds.curve_interferences(ic), true)?;
        }
        for is in 1..=ds.nb_shapes() {
            let s = ds.shape(is, true);
            if !s.is_null() && s.shape_type()? == ShapeEnum::Edge {
                count(ds.shape_interferences_index(is, true)?, false)?;
            }
        }
        drop(ds);
        for ip in 1..=np {
            if tp[usize::try_from(ip).unwrap_or(0)] == 0 {
                h.change_ds().remove_point(ip)?;
            }
        }
        Ok(())
    }

    /// `BuildEdges(HDS)`: the curves coinciding in 2d on a surface made equivalent, the
    /// vertices of their ends and points shared, the curves from a mother cut, the
    /// edges of the curves without mother built and the unused points removed;
    /// `b.build_edges(&hds)?`.
    pub fn build_edges(&mut self, hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
        let h = hds.get().ok_or_else(null_hds)?;
        self.my_new_edges.clear(true);
        self.my_coincident_edges.clear(true);
        Self::merge_surface_curves(h)?;
        Self::root_equivalent_curves(h);
        let m = self.equivalent_point_vertices(h)?;
        self.share_end_vertices(h, &m)?;
        self.share_point_vertices(h)?;
        Self::cut_mothered_curves(h)?;
        for ic in curve_indices(h, false) {
            if h.curve(ic).mother() == 0 {
                self.build_edges_of(ic, hds)?;
            }
        }
        Self::remove_unused_points(h)
    }
}
