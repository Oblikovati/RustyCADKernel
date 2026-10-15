// Rust translation of BRepFeat_RibSlot::NoSlidingProfile (the closing part) of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_RibSlot.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `NoSlidingProfile`: the profile face of a rib not sliding: the wire closed by the
//! tangents at its ends when they meet beyond it (concave), else by the edges of the
//! bounding face between the tangents (the end edges lengthened when they stop on the
//! faces), the side holding the check point kept. The edges are chained vertex to
//! vertex in a wire, the edges of the form wire recorded in the profile map.
use super::RibSlot;
use super::extreme::Extremes;
use super::geometry::{edge_curve, live};
use super::sliding::{ProfileInput, concavity, next_cyclic};
use rustycad_tkbool::brep_algo;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakeVertex};

/// The wire of the profile being chained: its last edge, its first vertex and point.
pub(super) struct Chain {
    pub(super) b: BRepBuilder,
    pub(super) w: Wire,
    pub(super) the_fv: Vertex,
    pub(super) the_firstpoint: Point3,
    pub(super) the_last_edge: Edge,
}

impl Chain {
    fn new() -> Self {
        let b = BRepBuilder::default();
        let mut w = Wire::default();
        b.make_wire(&mut w);
        Self {
            b,
            w,
            the_fv: Vertex::default(),
            the_firstpoint: Point3::default(),
            the_last_edge: Edge::default(),
        }
    }

    /// The edge added at the end of the wire, its first vertex (and point when
    /// `with_point`) kept when it is the first edge.
    pub(super) fn add(&mut self, e: &Edge, with_point: bool) -> Result<(), Failure> {
        self.b.add(&mut self.w, e)?;
        if self.the_fv.is_null() {
            self.the_fv = topexp::first_vertex(e, true);
            if with_point {
                self.the_firstpoint = BRepTool::pnt(&self.the_fv)?;
            }
        }
        self.the_last_edge = e.clone();
        Ok(())
    }

    /// The segment to `to`: from `from` for the first edge, else from the end of the
    /// last edge to a new vertex.
    fn segment(&mut self, from: Point3, to: Point3) -> Result<(), Failure> {
        let e = if self.the_last_edge.is_null() {
            BRepLibMakeEdge::from_points(from, to)?.edge()?
        } else {
            let v1 = topexp::last_vertex(&self.the_last_edge, true);
            let v2 = BRepLibMakeVertex::new(to)?.vertex()?;
            BRepLibMakeEdge::from_vertices(&v1, &v2)?.edge()?
        };
        self.add(&e, true)
    }

    /// The first vertex of the wire when `pp` is on it, else `v`.
    pub(super) fn closing(&self, pp: Point3, v: Vertex, my_tol: f64) -> Vertex {
        if !self.the_fv.is_null() && self.the_firstpoint.distance(&pp) <= my_tol {
            return self.the_fv.clone();
        }
        v
    }
}

/// The ends of the profile: the end points and tangent lines, the lengthened edges.
pub(super) struct Ends {
    pub(super) firstpoint: Point3,
    pub(super) lastpoint: Point3,
    pub(super) firstln: Handle<GeomLine>,
    pub(super) lastln: Handle<GeomLine>,
    pub(super) false_first_edge: Edge,
    pub(super) false_last_edge: Edge,
    pub(super) false_only_one: Edge,
}

/// The line through `p` along `v` (`new Geom_Line(p, v)`).
fn line_of(p: Point3, v: Vector3) -> Result<Handle<GeomLine>, Failure> {
    Ok(Handle::new(GeomLine::from_point(
        p,
        Direction3::from_vector(v)?,
    )))
}

impl RibSlot {
    /// The tangent lines at the start of the first edge (backward) and at the end of the
    /// last edge.
    fn ends(&self, x: &Extremes) -> Result<Ends, Failure> {
        let (first_curve, f1, _) = edge_curve(&x.first_edge)?;
        let (last_curve, _, l2) = edge_curve(&x.last_edge)?;
        let (firstpoint, firstvect) = live(&first_curve)?.d1(f1)?;
        let lastln = line_of(firstpoint, -firstvect)?;
        let (lastpoint, lastvect) = live(&last_curve)?.d1(l2)?;
        let firstln = line_of(lastpoint, lastvect)?;
        Ok(Ends {
            firstpoint,
            lastpoint,
            firstln,
            lastln,
            false_first_edge: Edge::default(),
            false_last_edge: Edge::default(),
            false_only_one: Edge::default(),
        })
    }

    /// The end edges lengthened when they stop on the faces, the tangent lines moved to
    /// their new ends (concavity 3).
    fn lengthen_ends(
        &self,
        input: &ProfileInput,
        x: &Extremes,
        en: &mut Ends,
    ) -> Result<(), Failure> {
        let same = x.first_edge.is_same(&x.last_edge);
        if x.on_first_face {
            en.false_first_edge = x.first_edge.clone();
            self.edge_extention(&mut en.false_first_edge, input.bnd, true)?;
            let (cc, f, _) = edge_curve(&en.false_first_edge)?;
            let (p, v) = live(&cc)?.d1(f)?;
            en.firstpoint = p;
            en.lastln = line_of(p, -v)?;
            if same {
                en.false_only_one = en.false_first_edge.clone();
            }
        }
        if x.on_last_face {
            if !same {
                en.false_last_edge = x.last_edge.clone();
            } else {
                if en.false_only_one.is_null() {
                    en.false_only_one = x.last_edge.clone();
                }
                en.false_last_edge = en.false_only_one.clone();
            }
            self.edge_extention(&mut en.false_last_edge, input.bnd, false)?;
            if same {
                en.false_only_one = en.false_last_edge.clone();
            }
            let vv2 = topexp::last_vertex(&en.false_last_edge, true);
            let (cc, _, l) = edge_curve(&en.false_last_edge)?;
            let (_, v) = live(&cc)?.d1(l)?;
            en.lastpoint = BRepTool::pnt(&vv2)?;
            en.firstln = line_of(en.lastpoint, v)?;
        }
        Ok(())
    }

    /// The edges of the bounding face between the tangent lines (concavity 3); false
    /// when a line misses the bounding wire.
    fn no_sliding_bound(
        &self,
        input: &ProfileInput,
        en: &Ends,
        ch: &mut Chain,
    ) -> Result<bool, Failure> {
        let (c1, c2): (Handle<dyn GeomCurve>, Handle<dyn GeomCurve>) =
            (en.firstln.clone().cast(), en.lastln.clone().cast());
        let (l2d1, l2d2) = (input.to_2d(&c1)?, input.to_2d(&c2)?);
        let Some([(bnd_edge1, bnd_pnt1), (bnd_edge2, bnd_pnt2)]) =
            input.bound_edges([(&c1, &l2d1), (&c2, &l2d2)])?
        else {
            return Ok(false);
        };
        ch.segment(en.lastpoint, bnd_pnt1)?;
        if bnd_edge1.is_same(&bnd_edge2) {
            ch.segment(bnd_pnt1, bnd_pnt2)?;
            ch.segment(bnd_pnt2, en.firstpoint)?;
            return Ok(true);
        }
        let bnd_wire = input.bnd_wire()?;
        let mut explo = WireExplorer::with_wire(&bnd_wire)?;
        let mut last_pnt = Point3::default();
        while explo.more() {
            let e = explo.current().clone();
            if e.is_same(&bnd_edge1) {
                let pp = BRepTool::pnt(&topexp::last_vertex(&e, true))?;
                if pp.distance(&bnd_pnt1) > BRepTool::edge_tolerance(&e)? {
                    last_pnt = pp;
                }
                ch.segment(bnd_pnt1, last_pnt)?;
                break;
            }
            explo.next()?;
        }
        next_cyclic(&mut explo, &bnd_wire)?;
        loop {
            let e = explo.current().clone();
            let fin = e.is_same(&bnd_edge2);
            let pp = if fin {
                bnd_pnt2
            } else {
                BRepTool::pnt(&topexp::last_vertex(&e, true))?
            };
            ch.segment(last_pnt, pp)?;
            last_pnt = pp;
            next_cyclic(&mut explo, &bnd_wire)?;
            if fin {
                break;
            }
        }
        ch.segment(bnd_pnt2, en.firstpoint)?;
        Ok(true)
    }

    /// `NoSlidingProfile(Prof, RevolRib, myTol, Concavite, myPln, bnd, BndFace, CheckPnt,
    /// ...)`: the profile face of a form not sliding (false when it cannot be built
    /// valid); `rib.no_sliding_profile(&mut prof, &input, &mut concavite, &ext)?`.
    pub(crate) fn no_sliding_profile(
        &mut self,
        prof: &mut Face,
        input: &ProfileInput,
        concavite: &mut i32,
        x: &Extremes,
    ) -> Result<bool, Failure> {
        let mut ch = Chain::new();
        let mut en = self.ends(x)?;
        let (pt, _) = concavity(
            input,
            &en.firstln,
            &en.lastln,
            (self.first_pnt, self.last_pnt),
            concavite,
        )?;
        // ---Construction of the face profile
        if *concavite == 3 {
            self.lengthen_ends(input, x, &mut en)?;
            if !self.no_sliding_bound(input, &en, &mut ch)? {
                return Ok(false);
            }
        }
        if *concavite == 1 {
            ch.segment(pt, en.firstpoint)?;
        }
        self.profile_wire_edges(input, x, &en, &mut ch)?;
        self.close_profile(input, *concavite, pt, &en, &mut ch)?;
        ch.w.set_closed(BRepTool::is_closed(&ch.w)?)?;
        let fac = BRepLibMakeFace::from_plane_wire(input.plane()?.pln(), &ch.w, true)?.face()?;
        if !brep_algo::is_valid(&fac)? {
            return Ok(false);
        }
        *prof = if *concavite == 3 {
            self.material_side(input, &fac, true)?
        } else {
            fac
        };
        brep_algo::is_valid(prof)
    }

    /// The closing edges after the form wire: to the meeting point of the tangents
    /// (concave), or between the end points (parallel concave).
    fn close_profile(
        &self,
        input: &ProfileInput,
        concavite: i32,
        pt: Point3,
        en: &Ends,
        ch: &mut Chain,
    ) -> Result<(), Failure> {
        if concavite == 1 {
            let eef = if ch.the_last_edge.is_null() {
                BRepLibMakeEdge::from_points(en.lastpoint, pt)?.edge()?
            } else {
                let v1 = topexp::last_vertex(&ch.the_last_edge, true);
                let v2 = BRepLibMakeVertex::new(pt)?.vertex()?;
                let v2 = ch.closing(pt, v2, input.my_tol);
                BRepLibMakeEdge::from_vertices(&v1, &v2)?.edge()?
            };
            ch.add(&eef, false)?;
        }
        if concavite == 2 {
            let e = BRepLibMakeEdge::from_points(en.lastpoint, en.firstpoint)?.edge()?;
            ch.b.add(&mut ch.w, &e)?;
        }
        Ok(())
    }
}
