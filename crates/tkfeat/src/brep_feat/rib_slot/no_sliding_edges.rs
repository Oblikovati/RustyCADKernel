// Rust translation of the wire edges of BRepFeat_RibSlot::NoSlidingProfile of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_RibSlot.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges of the form wire in the profile of a rib not sliding: each rebuilt on its
//! curve from the end of the chain (the lengthened end edges when the ends stop on the
//! faces), closed on the first vertex of the chain when it ends there, and recorded in
//! the profile map of the edge it comes from.
use super::RibSlot;
use super::extreme::Extremes;
use super::geometry::edge_curve;
use super::no_sliding::{Chain, Ends};
use super::sliding::ProfileInput;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

impl RibSlot {
    /// The edge `e` (or its new copy `src`) recorded for `key` in the profile map and
    /// added to the chain.
    fn record(&mut self, key: &Edge, e: &Edge, ch: &mut Chain) -> Result<(), Failure> {
        self.lf_map
            .try_bound(key.clone().into_shape(), List::default())
            .append(e.clone().into_shape());
        ch.add(e, false)
    }

    /// The edge on the curve of `src` from the end of the chain (or from the start of
    /// `src`) to the end of `src`, closed on the first vertex of the chain when the end
    /// is on it; `oriented` takes the end with the orientation of the edge, as natively
    /// everywhere but one case.
    fn closing_edge(
        &self,
        src: &Edge,
        oriented_end: bool,
        my_tol: f64,
        ch: &Chain,
    ) -> Result<Edge, Failure> {
        let (cc, _, _) = edge_curve(src)?;
        let end = topexp::last_vertex(src, oriented_end);
        let pp = BRepTool::pnt(&end)?;
        let v2 = ch.closing(pp, end, my_tol);
        let v1 = if ch.the_last_edge.is_null() {
            topexp::first_vertex(src, true)
        } else {
            topexp::last_vertex(&ch.the_last_edge, true)
        };
        BRepLibMakeEdge::from_curve_vertices(&cc, &v1, &v2)?.edge()
    }

    /// The edge `src` itself as the first edge of the chain, else rebuilt on its curve
    /// from the end of the chain to its end.
    fn chained_edge(&self, src: &Edge, ch: &Chain) -> Result<Edge, Failure> {
        if ch.the_last_edge.is_null() {
            return Ok(src.clone());
        }
        let (cc, _, _) = edge_curve(src)?;
        let v1 = topexp::last_vertex(&ch.the_last_edge, true);
        let v2 = topexp::last_vertex(src, true);
        BRepLibMakeEdge::from_curve_vertices(&cc, &v1, &v2)?.edge()
    }

    /// The wire of one edge: the lengthened edge when an end stops on a face, else the
    /// edge rebuilt (its end taken without orientation after a chain, as natively).
    fn single_edge(
        &mut self,
        input: &ProfileInput,
        x: &Extremes,
        en: &Ends,
        ch: &mut Chain,
    ) -> Result<(), Failure> {
        let the_edge = if x.on_first_face || x.on_last_face {
            self.closing_edge(&en.false_only_one, true, input.my_tol, ch)?
        } else {
            // Attention case Wire Reversed -> LastVertex without true
            let oriented = ch.the_last_edge.is_null();
            self.closing_edge(&x.first_edge, oriented, input.my_tol, ch)?
        };
        self.record(&x.first_edge, &the_edge, ch)
    }

    /// The edges of the form wire from the first to the last one.
    fn several_edges(
        &mut self,
        input: &ProfileInput,
        x: &Extremes,
        en: &Ends,
        ch: &mut Chain,
    ) -> Result<(), Failure> {
        let first = if x.on_first_face {
            &en.false_first_edge
        } else {
            &x.first_edge
        };
        let the_edge = self.chained_edge(first, ch)?;
        self.record(&x.first_edge, &the_edge, ch)?;
        let mut ex = WireExplorer::with_wire(&self.wire)?;
        while ex.more() && !ex.current().is_same(&x.first_edge) {
            ex.next()?;
        }
        ex.next()?;
        while ex.more() && !ex.current().is_same(&x.last_edge) {
            let e = ex.current().clone();
            let eee = self.chained_edge(&e, ch)?;
            self.record(&e, &eee, ch)?;
            ex.next()?;
        }
        if !x.on_last_face {
            // the edge where the walk stopped (the last edge, or null past the end)
            let edg = ex.current().clone();
            let eee = self.closing_edge(&edg, true, input.my_tol, ch)?;
            return self.record(&edg, &eee, ch);
        }
        let eee = self.closing_edge(&en.false_last_edge, true, input.my_tol, ch)?;
        self.record(&x.last_edge, &eee, ch)
    }

    /// The edges of the form wire added to the chain of the profile, each recorded in
    /// the profile map.
    pub(super) fn profile_wire_edges(
        &mut self,
        input: &ProfileInput,
        x: &Extremes,
        en: &Ends,
        ch: &mut Chain,
    ) -> Result<(), Failure> {
        if !self.lf_map.is_bound(&x.first_edge.clone().into_shape()) {
            self.lf_map
                .bind(x.first_edge.clone().into_shape(), List::<Shape>::default());
        }
        if x.first_edge.is_same(&x.last_edge) {
            return self.single_edge(input, x, en, ch);
        }
        self.several_edges(input, x, en, ch)
    }
}
