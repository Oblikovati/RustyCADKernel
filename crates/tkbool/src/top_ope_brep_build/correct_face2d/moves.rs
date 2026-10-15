//! The moves of the p-curves of `TopOpeBRepBuild_CorrectFace2d` (ConnectWire,
//! TranslateCurve2d, OuterWire, BndBoxWire, MoveWire2d, MoveWires2d, UpdateEdge).
use super::{TopOpeBRepBuildCorrectFace2d, no_map};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Vector2;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
use std::f64::consts::PI;

type C2d = Handle<dyn Geom2dCurve>;

/// The p-curve of the edge on the face, trimmed to its range and translated
/// (`TranslateCurve2d`).
fn translate_curve2d(an_e: &Edge, a_face: &Face, a_tr_v: Vector2) -> Result<C2d, Failure> {
    let pc = BRepTool::curve_on_face(an_e, a_face)?;
    let c2d = pc.pcurve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "TranslateCurve2d: no p-curve",
            "TopOpeBRepBuild_CorrectFace2d",
        )
    })?;
    let a_tr_c: C2d = c2d.copy()?.cast();
    let new_c2d = Geom2dTrimmedCurve::new(&a_tr_c, pc.first, pc.last, true, true)?;
    rustycad_tkg2d::geom2d::Geom2dGeometry::translate(&new_c2d, a_tr_v)?;
    Ok(Handle::new(new_c2d).cast())
}

/// Whether a 2d translation is not a whole number of periods in a periodic direction
/// (`nonPU`/`nonPV` of `ConnectWire`).
fn non_period(t: f64, periodic: bool) -> bool {
    let mut non = t.abs() < 1e-7;
    if !non && periodic {
        let d = t.abs() % (2. * PI);
        non = d > 1e-7 && (2. * PI - d > 1e-7);
    }
    non
}

impl TopOpeBRepBuildCorrectFace2d<'_> {
    /// The p-curves of the edge translated and updated, both of a closed edge
    /// (by the builder on a trial, by `UpdateEdge` otherwise).
    fn translate_edge(
        &mut self,
        an_edge: &Edge,
        a_copy_face: &Face,
        a_tr_v: Vector2,
        try_flag: bool,
        an_ed_map: &mut IndexedMap<Shape, ShapeMapHasher>,
    ) -> Result<(), Failure> {
        let bb = BRepBuilder::default();
        if !BRepTool::is_closed_on_face(an_edge, a_copy_face)? {
            let a_tr_c = translate_curve2d(an_edge, a_copy_face, a_tr_v)?;
            if try_flag {
                return bb.update_edge_pcurve_on_face(
                    an_edge,
                    &a_tr_c,
                    a_copy_face,
                    self.my_face_tolerance,
                );
            }
            return self.update_edge(an_edge, &a_tr_c, a_copy_face, self.my_face_tolerance);
        }
        if an_ed_map.contains(an_edge.as_ref()) {
            return Ok(());
        }
        an_ed_map.add(an_edge.clone().into_shape());
        let reversed = topods::edge(&an_edge.reversed())?;
        let (an_ef, an_er) = if an_edge.orientation() == Orientation::Forward {
            (an_edge.clone(), reversed)
        } else {
            (reversed, an_edge.clone())
        };
        let a_tr_cf = translate_curve2d(&an_ef, a_copy_face, a_tr_v)?;
        let a_tr_cr = translate_curve2d(&an_er, a_copy_face, a_tr_v)?;
        if try_flag {
            return bb.update_edge_pcurves_on_face(
                an_edge,
                (&a_tr_cf, &a_tr_cr),
                a_copy_face,
                self.my_face_tolerance,
            );
        }
        self.update_edge2(
            an_edge,
            (&a_tr_cf, &a_tr_cr),
            a_copy_face,
            self.my_face_tolerance,
        )
    }

    /// The edge to start the connection from: the first non-degenerated edge to
    /// avoid, else the degenerated one of the longest 2d extent.
    fn first_avoid_edge(
        a_copy_face: &Face,
        an_avoid_map: &IndexedMap<Shape>,
    ) -> Result<Shape, Failure> {
        for e in an_avoid_map.iter() {
            if !BRepTool::degenerated(&topods::edge(e)?)? {
                return Ok(e.clone());
            }
        }
        let (mut a_dmax, mut a_first_edge) = (0., Shape::default());
        for e in an_avoid_map.iter() {
            let (pf, pl) = Self::get_p2d_fl(a_copy_face, &topods::edge(e)?)?;
            let a_r = pf.distance(pl);
            if (a_r - a_dmax) > 1e-7 {
                (a_dmax, a_first_edge) = (a_r, e.clone());
            }
        }
        Ok(a_first_edge)
    }

    /// `ConnectWire(aCopyFace, aCopyAvoidMap, aTryToConnectFlag)` (private): the edges
    /// of the current wire, from an edge to avoid, translated to follow each other in
    /// 2d; 1 when a translation is not a whole number of periods (or, on a trial, when
    /// the wire does not close).
    pub(super) fn connect_wire(
        &mut self,
        a_copy_face: &Face,
        a_copy_avoid_map: &IndexedMap<Shape>,
        try_flag: bool,
    ) -> Result<i32, Failure> {
        let priz = self.make_right_wire()?;
        if priz != 0 {
            return Ok(priz);
        }
        let (mut nb_edges, mut an_avoid_map) = (0, IndexedMap::<Shape>::default());
        for an_edge in Explorer::new(&self.my_current_wire, ShapeEnum::Edge, ShapeEnum::Shape)? {
            nb_edges += 1;
            if a_copy_avoid_map.contains(&an_edge) {
                an_avoid_map.add(an_edge);
            }
        }
        if nb_edges == 1 || an_avoid_map.extent() == nb_edges {
            return Ok(0);
        }
        let a_first_edge = Self::first_avoid_edge(a_copy_face, &an_avoid_map)?;
        let seq: Vec<Shape> = self
            .make_head_list(&a_first_edge)?
            .iter()
            .cloned()
            .collect();
        let tol = precision::confusion();
        let (pa1, mut pb) = Self::get_p2d_fl(a_copy_face, &topods::edge(&seq[0])?)?;
        let mut an_ed_map: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        let bas = BRepAdaptorSurface::with_face(a_copy_face, true)?;
        for (k, s) in seq.iter().enumerate().skip(1) {
            let an_edge = topods::edge(s)?;
            let (pf, pl) = Self::get_p2d_fl(a_copy_face, &an_edge)?;
            if pf.distance(pb) < tol {
                pb = pl;
                continue;
            }
            let a_tr_v = Vector2::between(pf, pb);
            let non_pu = non_period(a_tr_v.x(), bas.is_u_periodic()?);
            let non_pv = non_period(a_tr_v.y(), bas.is_v_periodic()?);
            if non_pu && non_pv && !BRepTool::degenerated(&an_edge)? {
                return Ok(1);
            }
            if BRepTool::is_closed_on_face(&an_edge, a_copy_face)?
                && an_ed_map.contains(an_edge.as_ref())
            {
                continue;
            }
            self.translate_edge(&an_edge, a_copy_face, a_tr_v, try_flag, &mut an_ed_map)?;
            pb = Self::get_p2d_fl(a_copy_face, &an_edge)?.1;
            if try_flag && k + 1 == seq.len() && pa1.distance(pb) > tol {
                return Ok(1);
            }
        }
        Ok(0)
    }

    /// `OuterWire(anOuterWire)` (private): the first wire of the corrected face whose
    /// face alone classifies the infinite point OUT.
    pub(super) fn outer_wire(&self) -> Result<Option<Wire>, Failure> {
        let (surf, loc) = BRepTool::surface_with_location(&self.my_corrected_face)?;
        for w in Explorer::new(&self.my_corrected_face, ShapeEnum::Wire, ShapeEnum::Shape)? {
            let a_wire = topods::wire(&w)?;
            let bb = BRepBuilder::default();
            let mut new_face = Face::default();
            bb.make_face_on_located_surface(&mut new_face, &surf, &loc, self.my_face_tolerance)?;
            bb.add(&mut new_face, &a_wire)?;
            let class2d = BRepTopAdaptorFClass2d::new(&new_face, self.my_face_tolerance)?;
            if class2d.perform_infinite_point()? == State::Out {
                return Ok(Some(a_wire));
            }
        }
        Ok(None)
    }

    /// `BndBoxWire(aWire, aB2d)` (private): the 2d box of the p-curves of the wire.
    fn bnd_box_wire(&self, a_wire: &Wire) -> Result<BoundingBox2, Failure> {
        let mut b2d = BoundingBox2::default();
        for e in Explorer::new(a_wire, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let an_edge = topods::edge(&e)?;
            let tol_e = BRepTool::edge_tolerance(&an_edge)?;
            let bac2d = BRepAdaptorCurve2d::with_edge_on_face(&an_edge, &self.my_corrected_face)?;
            Add2dCurve::add_adaptor(&bac2d, tol_e, &mut b2d)?;
        }
        Ok(b2d)
    }

    /// `MoveWire2d(aWire, aTrV)` (private): the p-curves of the wire translated.
    fn move_wire2d(&mut self, a_wire: &Wire, a_tr_v: Vector2) -> Result<(), Failure> {
        if a_tr_v.magnitude() < precision::confusion() {
            return Ok(());
        }
        let face = self.my_corrected_face.clone();
        let mut an_ed_map: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
        let edges: Vec<Shape> = Explorer::new(a_wire, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
        for e in edges {
            self.translate_edge(&topods::edge(&e)?, &face, a_tr_v, false, &mut an_ed_map)?;
        }
        Ok(())
    }

    /// The translation by whole periods in U bringing `a` into [0, 2 PI).
    fn period_shift(a: f64) -> Vector2 {
        let two_pi = 2. * PI;
        let b = (-a / two_pi) as i32;
        let k = if a < 0. { 1 } else { 0 };
        Vector2::new(f64::from(b + k) * two_pi, 0.)
    }

    /// `MoveWires2d(anOuterWire)` (private): the outer wire moved into the first period
    /// in U, then each wire whose box is out of its box moved by periods after it.
    pub(super) fn move_wires2d(&mut self, an_outer_wire: &Wire) -> Result<(), Failure> {
        let [ou1, _, ou2, _] = self.bnd_box_wire(an_outer_wire)?.get()?;
        self.move_wire2d(an_outer_wire, Self::period_shift(0.5 * (ou1 + ou2)))?;
        let b2d_outer = self.bnd_box_wire(an_outer_wire)?;
        let [ou1, _, _, _] = b2d_outer.get()?;
        let wires: Vec<Shape> =
            Explorer::new(&self.my_corrected_face, ShapeEnum::Wire, ShapeEnum::Shape)?.collect();
        for w in wires {
            self.my_current_wire = topods::wire(&w)?;
            if w.is_equal(an_outer_wire.as_ref()) {
                continue;
            }
            let b2d = self.bnd_box_wire(&self.my_current_wire.clone())?;
            let [u1, _, _, _] = b2d.get()?;
            if b2d.is_out_box(&b2d_outer) {
                let cw = self.my_current_wire.clone();
                self.move_wire2d(&cw, Self::period_shift(u1 - ou1))?;
            }
        }
        Ok(())
    }

    /// The copy in the map of the original of `e_copy` (made, with the vertices, when
    /// missing): the edge to update with it.
    fn mapped_original(&mut self, e_copy: &Edge) -> Result<Option<Edge>, Failure> {
        let Ok(e) = self
            .my_ed_map_inversed
            .find_from_key(e_copy.as_ref())
            .cloned()
        else {
            return Ok(None);
        };
        let pmap = self.my_map.as_deref_mut().ok_or_else(no_map)?;
        if let Ok(an_edge) = pmap.find_from_key(&e) {
            return topods::edge(an_edge).map(Some);
        }
        let bb = BRepBuilder::default();
        let mut an_edge = e.empty_copied()?;
        for v in Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            bb.add(&mut an_edge, &v)?;
        }
        pmap.add(e, an_edge.clone());
        Ok(Some(topods::edge(&an_edge)?))
    }

    /// `UpdateEdge(E, C, F, Tol)` (private): the p-curve on the copy and on the copy of
    /// the original edge in the map.
    fn update_edge(&mut self, e_copy: &Edge, c: &C2d, f: &Face, tol: f64) -> Result<(), Failure> {
        let bb = BRepBuilder::default();
        if let Some(an_edge) = self.mapped_original(e_copy)? {
            bb.update_edge_pcurve_on_face(&an_edge, c, f, tol)?;
        }
        bb.update_edge_pcurve_on_face(e_copy, c, f, tol)
    }

    /// `UpdateEdge(E, C1, C2, F, Tol)` (private): the seam p-curves likewise.
    fn update_edge2(
        &mut self,
        e_copy: &Edge,
        c: (&C2d, &C2d),
        f: &Face,
        tol: f64,
    ) -> Result<(), Failure> {
        let bb = BRepBuilder::default();
        if let Some(an_edge) = self.mapped_original(e_copy)? {
            bb.update_edge_pcurves_on_face(&an_edge, c, f, tol)?;
        }
        bb.update_edge_pcurves_on_face(e_copy, c, f, tol)
    }
}
