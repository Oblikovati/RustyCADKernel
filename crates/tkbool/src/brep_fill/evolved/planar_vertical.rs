// Rust translation of BRepFill_Evolved::PlanarPerform and VerticalPerform of
// ModelingAlgorithms/TKBool/BRepFill/BRepFill_Evolved.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The volevos of planar profile parts (faces between the parallels of the spine at the
//! distances of the edge ends) and of vertical parts (prisms of a parallel).
use super::statics::{altitud, confusion, distance_to_oz, edge_vertices};
use super::{BRepFillEvolved, ProfileGenerated, ShapeOrigins};
use crate::brep_algo::BRepAlgoFaceRestrictor;
use crate::brep_fill::{BRepFillOffsetAncestors, BRepFillOffsetWire};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkprim::brep_sweep::BRepSweepPrism;
use rustycad_tktopalgo::brep_mat2d::{BRepMAT2dBisectingLocus, BRepMAT2dLinkTopoBilo};

/// The bisecting locus and links of the spine.
pub(super) type Bilo<'a> = (&'a BRepMAT2dBisectingLocus, &'a mut BRepMAT2dLinkTopoBilo);

impl BRepFillEvolved {
    /// The work data of a volevo: the spine (FORWARD), the profile, an empty map and
    /// an empty compound.
    pub(super) fn start_volevo(&mut self, sp: &Face, pr: &Wire) -> Result<(), Failure> {
        self.my_spine = topods::face(&sp.as_ref().oriented(Orientation::Forward))?;
        self.my_profile = pr.clone();
        self.my_map.clear(false);
        let mut c = Compound::default();
        BRepBuilder::default().make_compound(&mut c);
        self.my_shape = c.into_shape();
        Ok(())
    }

    /// `l` appended to the generated shapes of (`a`, `b`), bound empty when absent.
    pub(super) fn append_generated(
        &mut self,
        a: &Shape,
        b: &Shape,
        l: impl IntoIterator<Item = Shape>,
    ) {
        let inner = self
            .my_map
            .try_emplaced(a.clone(), ProfileGenerated::default);
        let list = inner.try_emplaced(b.clone(), List::default);
        for s in l {
            list.append(s);
        }
    }

    /// The parallel of the spine at the distance of `v` (computed once per vertex),
    /// recorded in the map by ancestor and with its ancestors in `off_anc`.
    fn parallel_at(
        &mut self,
        v: &Vertex,
        (offset, alt): (f64, f64),
        bilo: &mut Bilo<'_>,
        join: JoinType,
        (paral, off_anc): (&mut BRepFillOffsetWire, &mut BRepFillOffsetAncestors),
        map_vp: &mut ShapeOrigins,
    ) -> Result<(), Failure> {
        if map_vp.is_bound(v.as_ref()) {
            return Ok(());
        }
        let spine = self.my_spine.clone();
        paral.perform_with_bilo(&spine, offset, bilo.0, bilo.1, join, alt)?;
        off_anc.perform(paral)?;
        let shape = paral.shape().clone();
        map_vp.bind(v.as_ref().clone(), shape.clone());
        for wc in Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let gs = off_anc.ancestor(&topods::edge(&wc)?)?.clone();
            let bound = self
                .my_map
                .seek(&gs)
                .is_some_and(|m| m.is_bound(v.as_ref()));
            if !bound {
                let generated = paral.generated_shapes(&gs)?.clone();
                self.my_map
                    .try_emplaced(gs.clone(), ProfileGenerated::default)
                    .bind(v.as_ref().clone(), generated);
            }
        }
        Ok(())
    }

    /// The wires of a parallel added to the face restrictor (reversed when asked).
    fn add_parallel_wires(
        rest: &Shape,
        to_reverse: bool,
        fr: &mut BRepAlgoFaceRestrictor,
    ) -> Result<(), Failure> {
        if rest.is_null() {
            return Ok(());
        }
        let wires: Vec<Shape> = if rest.shape_type()? == ShapeEnum::Wire {
            vec![rest.clone()]
        } else {
            Explorer::new(rest, ShapeEnum::Wire, ShapeEnum::Shape)?.collect()
        };
        for w in wires {
            fr.add(&topods::wire(&if to_reverse { w.reversed() } else { w })?);
        }
        Ok(())
    }

    /// The faces of one planar profile edge: bounded by the parallels at its ends, set
    /// at its altitude, recorded by the ancestors of their edges.
    fn planar_edge(
        &mut self,
        e: &Edge,
        bilo: &mut Bilo<'_>,
        join: JoinType,
        paral: &mut BRepFillOffsetWire,
        map_vp: &mut ShapeOrigins,
    ) -> Result<(), Failure> {
        let mut fr = BRepAlgoFaceRestrictor::new();
        let mut off_anc = BRepFillOffsetAncestors::new();
        let (v0, v1) = edge_vertices(e);
        let alt = altitud(&v0)?;
        let offset = [distance_to_oz(&v0)?, distance_to_oz(&v1)?];
        let is_min_v1 = offset[0] < offset[1];
        for (i, v) in [&v0, &v1].into_iter().enumerate() {
            self.parallel_at(
                v,
                (offset[i], alt),
                bilo,
                join,
                (paral, &mut off_anc),
                map_vp,
            )?;
            let rest = map_vp.seek(v.as_ref()).cloned().unwrap_or_default();
            let to_reverse = (is_min_v1 && i == 1) || (!is_min_v1 && i == 0);
            Self::add_parallel_wires(&rest, to_reverse, &mut fr)?;
        }
        let mut t = Transform3::default();
        t.set_translation(Vector3::new(0., 0., alt));
        let moved = self.my_spine.as_ref().moved(&Location::new(t), false)?;
        fr.init(&topods::face(&moved)?, false, false);
        fr.perform()?;
        while fr.more() {
            let f = fr.current()?;
            BRepBuilder::default().add(&mut self.my_shape, f.as_ref())?;
            for ce in Explorer::new(f.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
                let ce = topods::edge(&ce)?;
                if off_anc.has_ancestor(&ce) {
                    let init_e = off_anc.ancestor(&ce)?.clone();
                    self.append_generated(&init_e, e.as_ref(), [f.as_ref().clone()]);
                }
            }
            fr.next()?;
        }
        Ok(())
    }

    /// `PlanarPerform(Spine, Profile, Locus, Link, Join)` (private): the faces between
    /// the parallels of the spine for each edge of a planar profile part;
    /// natively one offset tool serves every edge.
    pub(super) fn planar_perform(
        &mut self,
        sp: &Face,
        pr: &Wire,
        locus: &BRepMAT2dBisectingLocus,
        link: &mut BRepMAT2dLinkTopoBilo,
        join: JoinType,
    ) -> Result<(), Failure> {
        self.start_volevo(sp, pr)?;
        let mut map_vp = ShapeOrigins::default();
        let mut paral = BRepFillOffsetWire::default();
        let mut bilo: Bilo<'_> = (locus, link);
        let mut prof_exp = WireExplorer::with_wire(&self.my_profile)?;
        while prof_exp.more() {
            let e = prof_exp.current().clone();
            self.planar_edge(&e, &mut bilo, join, &mut paral, &mut map_vp)?;
            prof_exp.next()?;
        }
        Ok(())
    }

    /// The parallel at the first vertex of a vertical part, recorded at that vertex by
    /// ancestor.
    fn vertical_base(
        &mut self,
        v1: &Vertex,
        bilo: &mut Bilo<'_>,
        join: JoinType,
        paral: &mut BRepFillOffsetWire,
        off_anc: &mut BRepFillOffsetAncestors,
    ) -> Result<Shape, Failure> {
        let mut offset = distance_to_oz(v1)?;
        if offset.abs() < confusion() {
            offset = 0.;
        }
        let spine = self.my_spine.clone();
        paral.perform_with_bilo(&spine, offset, bilo.0, bilo.1, join, altitud(v1)?)?;
        off_anc.perform(paral)?;
        let base = paral.shape().clone();
        for an_edge in Explorer::new(&base, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let ae = off_anc.ancestor(&topods::edge(&an_edge)?)?.clone();
            self.append_generated(&ae, v1.as_ref(), [an_edge]);
        }
        Ok(base)
    }

    /// The prism images of the shapes generated at `v1` recorded at `v2` (last shapes)
    /// and at `e` (swept shapes), for every spine shape (the list at `v1` is read
    /// before the appends; natively the same list when `v1` is `v2`).
    fn record_prism(
        &mut self,
        ps: &mut BRepSweepPrism,
        v1: &Vertex,
        v2: &Vertex,
        e: &Edge,
    ) -> Result<(), Failure> {
        let keys: Vec<Shape> = self.my_map.items().map(|(k, _)| k.clone()).collect();
        for key in keys {
            let lof = super::find2(&self.my_map, &key, v1.as_ref())?.clone();
            let (mut last, mut swept) = (Vec::new(), Vec::new());
            for os in lof.iter() {
                last.push(ps.last_shape_of(os)?);
                swept.push(ps.shape_of(os)?);
            }
            self.append_generated(&key, v2.as_ref(), []);
            self.append_generated(&key, e.as_ref(), []);
            self.append_generated(&key, v2.as_ref(), last);
            self.append_generated(&key, e.as_ref(), swept);
        }
        Ok(())
    }

    /// `VerticalPerform(Spine, Profile, Locus, Link, Join)` (private): the parallel of
    /// the spine at the first vertex swept edge by edge along Oz.
    pub(super) fn vertical_perform(
        &mut self,
        sp: &Face,
        pr: &Wire,
        locus: &BRepMAT2dBisectingLocus,
        link: &mut BRepMAT2dLinkTopoBilo,
        join: JoinType,
    ) -> Result<(), Failure> {
        self.start_volevo(sp, pr)?;
        let mut paral = BRepFillOffsetWire::default();
        let mut off_anc = BRepFillOffsetAncestors::new();
        let mut bilo: Bilo<'_> = (locus, link);
        let (mut first, mut base) = (true, Shape::default());
        let mut prof_exp = WireExplorer::with_wire(&self.my_profile)?;
        while prof_exp.more() {
            let e = prof_exp.current().clone();
            let (v1, v2) = edge_vertices(&e);
            let (alt1, alt2) = (altitud(&v1)?, altitud(&v2)?);
            if first {
                base = self.vertical_base(&v1, &mut bilo, join, &mut paral, &mut off_anc)?;
                first = false;
            }
            let mut ps =
                BRepSweepPrism::new(&base, Vector3::new(0., 0., alt2 - alt1), false, true)?;
            base = ps.last_shape()?;
            let swept = ps.shape()?;
            for f in Explorer::new(&swept, ShapeEnum::Face, ShapeEnum::Shape)? {
                BRepBuilder::default().add(&mut self.my_shape, &f)?;
            }
            self.record_prism(&mut ps, &v1, &v2, &e)?;
            prof_exp.next()?;
        }
        Ok(())
    }
}
