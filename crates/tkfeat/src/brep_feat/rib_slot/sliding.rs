// Rust translation of BRepFeat_RibSlot::SlidingProfile of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_RibSlot.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SlidingProfile`: the profile face of a sliding rib: the wire between its end points
//! closed by the normals of the extreme faces when they meet beyond it (concave), else
//! by the edges of the bounding face between the normals, the side holding the check
//! point kept.
use super::RibSlot;
use super::extreme::Extremes;
use super::geometry::{edge_curve, int_par, live};
use crate::errors::null_object;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPICut;
use rustycad_tkbool::brep_algo;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api;
use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakeWire};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// A half line of a profile with its 2d image in the plane.
pub(super) type PlaneLine<'a> = (&'a Handle<dyn GeomCurve>, &'a Handle<dyn Geom2dCurve>);

/// The common inputs of `SlidingProfile` and `NoSlidingProfile`: whether the form is a
/// revolution rib, the tolerance, the plane of the profile, the bound of the basis, the
/// face of the bounding box in the plane, and the point inside the material.
#[derive(Clone)]
pub struct ProfileInput {
    /// Whether the form is a revolution rib.
    pub revol_rib: bool,
    /// The tolerance of the form (the largest vertex tolerance).
    pub my_tol: f64,
    /// The plane of the profile.
    pub pln: Handle<GeomPlane>,
    /// The bound of the box of the basis.
    pub bnd: f64,
    /// The face of the bounding box in the plane.
    pub bnd_face: Face,
    /// A point inside the material beside the wire.
    pub check_pnt: Point3,
}

impl ProfileInput {
    /// The plane (Standard_NullObject when null).
    pub(super) fn plane(&self) -> Result<&GeomPlane, Failure> {
        self.pln
            .get()
            .ok_or_else(|| null_object("BRepFeat_RibSlot", "null plane"))
    }

    /// A curve in the plane as a 2d curve (`GeomAPI::To2d`).
    pub(super) fn to_2d(
        &self,
        c: &Handle<dyn GeomCurve>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        geom_api::to_2d(c, &self.plane()?.pln())
    }

    /// The point of the plane at the 2d point `p`.
    pub(super) fn point(&self, p: Point2) -> Result<Point3, Failure> {
        self.plane()?.d0(p.x(), p.y())
    }

    /// The first intersection of the 2d lines `a` and `b`, None without.
    pub(super) fn meet(
        &self,
        a: &Handle<dyn Geom2dCurve>,
        b: &Handle<dyn Geom2dCurve>,
    ) -> Result<Option<Point3>, Failure> {
        let inter = InterCurveCurve::between(a, b, precision::confusion())?;
        if inter.nb_points() <= 0 {
            return Ok(None);
        }
        Ok(Some(self.point(inter.point(1)?)?))
    }

    /// The bounding wire of the plane (the first wire of the bounding face).
    pub(super) fn bnd_wire(&self) -> Result<Wire, Failure> {
        topods::wire(Explorer::new(&self.bnd_face, ShapeEnum::Wire, ShapeEnum::Shape)?.current())
    }

    /// The edge of the bounding wire cut by the half line `ln` (2d `ln2d`) ahead of its
    /// origin, with the point (the last such edge before the loop stops).
    pub(super) fn bound_hit(
        &self,
        e: &Edge,
        ln: &Handle<dyn GeomCurve>,
        ln2d: &Handle<dyn Geom2dCurve>,
    ) -> Result<Option<Point3>, Failure> {
        let (c, first, last) = edge_curve(e)?;
        let c2d = self.to_2d(&c)?;
        let Some(p) = self.meet(ln2d, &c2d)? else {
            return Ok(None);
        };
        let parl = int_par(ln, p)?;
        let parc = int_par(&c, p)?;
        Ok((parc >= first && parc <= last && parl >= 0.).then_some(p))
    }

    /// The edges of the bounding wire cut by the half lines `ln1` and `ln2` and their
    /// points; None when either is missing.
    pub(super) fn bound_edges(
        &self,
        lines: [PlaneLine; 2],
    ) -> Result<Option<[(Edge, Point3); 2]>, Failure> {
        let mut hits: [Option<(Edge, Point3)>; 2] = [None, None];
        let mut explo = WireExplorer::with_wire(&self.bnd_wire()?)?;
        while explo.more() {
            let e = explo.current().clone();
            for (k, (ln, ln2d)) in lines.iter().enumerate() {
                if let Some(p) = self.bound_hit(&e, ln, ln2d)? {
                    hits[k] = Some((e.clone(), p));
                }
            }
            if hits[0].is_some() && hits[1].is_some() {
                break;
            }
            explo.next()?;
        }
        Ok(match hits {
            [Some(a), Some(b)] => Some([a, b]),
            _ => None,
        })
    }
}

/// `explo.Next()`, restarting the bounding wire at its end (`Current()` null).
pub(super) fn next_cyclic(explo: &mut WireExplorer, bnd_wire: &Wire) -> Result<(), Failure> {
    if explo.more() {
        explo.next()?;
        if explo.current().is_null() {
            explo.init(bnd_wire)?;
        }
    } else {
        explo.init(bnd_wire)?;
    }
    Ok(())
}

/// The concavity test of both profiles: for a revolution rib, parallel opposite normals
/// ahead of the end points are concave (2) and equal ones skip the test; else concave
/// (1) when the normals meet ahead of the last one. The meeting point is returned, with
/// whether the parallel concave case was met.
pub(super) fn concavity(
    input: &ProfileInput,
    ln1: &Handle<GeomLine>,
    ln2: &Handle<GeomLine>,
    pnts: (Point3, Point3),
    concavite: &mut i32,
) -> Result<(Point3, bool), Failure> {
    let (l1, l2) = (
        ln1.get().ok_or_else(|| null_object("concavity", "null"))?,
        ln2.get().ok_or_else(|| null_object("concavity", "null"))?,
    );
    let mut test_ok = true;
    let mut parallel = false;
    if input.revol_rib {
        let (d1, d2) = (l1.position().direction(), l2.position().direction());
        if d1.is_opposite(d2, input.my_tol) {
            let par1 = elclib::parameter(&l1.lin(), pnts.0)?;
            let par2 = elclib::parameter(&l2.lin(), pnts.1)?;
            if par1 >= input.my_tol || par2 >= input.my_tol {
                *concavite = 2; // parallel and concave
                parallel = true;
            }
        }
        if d1.is_equal(&d2, input.my_tol) && *concavite == 3 {
            test_ok = false;
        }
    }
    let mut pt = Point3::default();
    if test_ok {
        let c1: Handle<dyn GeomCurve> = ln1.clone().cast();
        let c2: Handle<dyn GeomCurve> = ln2.clone().cast();
        if let Some(p) = input.meet(&input.to_2d(&c1)?, &input.to_2d(&c2)?)? {
            pt = p;
            if int_par(&c1, pt)? > 0. {
                *concavite = 1; // concave
            }
        }
    }
    Ok((pt, parallel))
}

/// The edge between two points added to the wire.
fn add_segment(ww: &mut BRepLibMakeWire, a: Point3, b: Point3) -> Result<(), Failure> {
    ww.add_edge(&BRepLibMakeEdge::from_points(a, b)?.edge()?)
}

impl RibSlot {
    /// The part of the bounding wire between the normals (concavity 3).
    fn sliding_bound(
        &self,
        input: &ProfileInput,
        ln1: &Handle<dyn GeomCurve>,
        ln2: &Handle<dyn GeomCurve>,
        ww: &mut BRepLibMakeWire,
    ) -> Result<bool, Failure> {
        let (l2d1, l2d2) = (input.to_2d(ln1)?, input.to_2d(ln2)?);
        let Some([(bnd_edge1, bnd_pnt1), (bnd_edge2, bnd_pnt2)]) =
            input.bound_edges([(ln1, &l2d1), (ln2, &l2d2)])?
        else {
            return Ok(false);
        };
        add_segment(ww, self.last_pnt, bnd_pnt1)?;
        if bnd_edge1.is_same(&bnd_edge2) {
            // Particular case : same edge -> simply determined path
            add_segment(ww, bnd_pnt1, bnd_pnt2)?;
            add_segment(ww, bnd_pnt2, self.first_pnt)?;
            return Ok(true);
        }
        let bnd_wire = input.bnd_wire()?;
        let mut explo = WireExplorer::with_wire(&bnd_wire)?;
        let mut last_pnt = Point3::default();
        while explo.more() {
            let e = explo.current().clone();
            if e.is_same(&bnd_edge1) {
                let pp = BRepTool::pnt(&topexp::last_vertex(&e, true))?;
                if pp.distance(&bnd_pnt1) >= BRepTool::edge_tolerance(&e)? {
                    last_pnt = pp;
                }
                add_segment(ww, bnd_pnt1, last_pnt)?;
                break;
            }
            explo.next()?;
        }
        next_cyclic(&mut explo, &bnd_wire)?;
        // Check if this is BndEdge2 -> if yes : it is required to turn to join FirstPnt
        loop {
            let e = explo.current().clone();
            let fin = e.is_same(&bnd_edge2);
            let pp = if fin {
                bnd_pnt2
            } else {
                BRepTool::pnt(&topexp::last_vertex(&e, true))?
            };
            add_segment(ww, last_pnt, pp)?;
            last_pnt = pp;
            next_cyclic(&mut explo, &bnd_wire)?;
            if fin {
                break;
            }
        }
        add_segment(ww, bnd_pnt2, self.first_pnt)?;
        Ok(true)
    }

    /// The edges of the wire of the form between its end points.
    fn sliding_wire_edges(&self, x: &Extremes, ww: &mut BRepLibMakeWire) -> Result<(), Failure> {
        let (first_curve, _, _) = edge_curve(&x.first_edge)?;
        if x.first_edge.is_same(&x.last_edge) {
            // only one edge : particular processing
            let fpar = int_par(&first_curve, self.first_pnt)?;
            let lpar = int_par(&first_curve, self.last_pnt)?;
            let c = if fpar > lpar {
                live(&first_curve)?.reversed()?
            } else {
                first_curve
            };
            return ww.add_edge(
                &BRepLibMakeEdge::from_curve_points(&c, self.first_pnt, self.last_pnt)?.edge()?,
            );
        }
        let fl_pnt = BRepTool::pnt(&topexp::last_vertex(&x.first_edge, true))?;
        ww.add_edge(
            &BRepLibMakeEdge::from_curve_points(&first_curve, self.first_pnt, fl_pnt)?.edge()?,
        )?;
        let mut ex = WireExplorer::with_wire(&self.wire)?;
        while ex.more() && !ex.current().is_same(&x.first_edge) {
            ex.next()?;
        }
        ex.next()?;
        while ex.more() && !ex.current().is_same(&x.last_edge) {
            ww.add_edge(ex.current())?;
            ex.next()?;
        }
        let (last_curve, _, _) = edge_curve(&x.last_edge)?;
        let lf_pnt = BRepTool::pnt(&topexp::first_vertex(&x.last_edge, true))?;
        ww.add_edge(
            &BRepLibMakeEdge::from_curve_points(&last_curve, lf_pnt, self.last_pnt)?.edge()?,
        )
    }

    /// `SlidingProfile(Prof, RevolRib, myTol, Concavite, myPln, BndFace, CheckPnt, ...)`:
    /// the profile face of a sliding form (false when it cannot be built valid);
    /// `rib.sliding_profile(&mut prof, &input, &mut concavite, &ext)?`.
    pub(crate) fn sliding_profile(
        &mut self,
        prof: &mut Face,
        input: &ProfileInput,
        concavite: &mut i32,
        x: &Extremes,
    ) -> Result<bool, Failure> {
        let mut ww = BRepLibMakeWire::new();
        let (mut f_n, mut l_n) = (
            self.normal(&x.first_face, self.first_pnt)?,
            self.normal(&x.last_face, self.last_pnt)?,
        );
        // Case of the groove (cut) <> rib (fuse) -> we are in the material
        if !self.fuse {
            f_n.reverse();
            l_n.reverse();
        }
        let ln2 = Handle::new(GeomLine::from_point(self.first_pnt, f_n));
        let ln1 = Handle::new(GeomLine::from_point(self.last_pnt, l_n));
        let (pt, parallel) = concavity(
            input,
            &ln1,
            &ln2,
            (self.first_pnt, self.last_pnt),
            concavite,
        )?;
        if parallel {
            add_segment(&mut ww, self.last_pnt, self.first_pnt)?;
        }
        let (c1, c2): (Handle<dyn GeomCurve>, Handle<dyn GeomCurve>) = (ln1.cast(), ln2.cast());
        if *concavite == 1 {
            // if concave : it is possible to extend first and last edges of the wire to
            // the bounding box
            add_segment(&mut ww, self.last_pnt, pt)?;
            add_segment(&mut ww, pt, self.first_pnt)?;
        } else if *concavite == 3 && !self.sliding_bound(input, &c1, &c2, &mut ww)? {
            return Ok(false);
        }
        // ---Construction of the profile
        self.sliding_wire_edges(x, &mut ww)?;
        let fac =
            BRepLibMakeFace::from_plane_wire(input.plane()?.pln(), &ww.wire()?, true)?.face()?;
        if !brep_algo::is_valid(&fac)? {
            return Ok(false);
        }
        *prof = if *concavite != 3 {
            fac
        } else {
            self.material_side(input, &fac, false)?
        };
        brep_algo::is_valid(prof)
    }

    /// The face holding the check point: `fac`, or the rest of the bounding face when
    /// the point is out of it (the history of that cut followed when `update`).
    pub(super) fn material_side(
        &mut self,
        input: &ProfileInput,
        fac: &Face,
        update: bool,
    ) -> Result<Face, Failure> {
        let cl = BRepTopAdaptorFClass2d::new(fac, BRepTool::face_tolerance(fac)?)?;
        let (u, v) = elslib::parameters(&input.plane()?.pln(), input.check_pnt);
        if cl.perform(Point2::new(u, v), true)? != State::Out {
            return Ok(fac.clone());
        }
        // If face is not the correct part of BndFace take the complementary
        let c = BRepAlgoAPICut::from_shapes(&input.bnd_face, fac)?;
        if update {
            self.update_descendants_bop(&c, c.shape(), false)?;
        }
        let w =
            topods::wire(Explorer::new(c.shape(), ShapeEnum::Wire, ShapeEnum::Shape)?.current())?;
        BRepLibMakeFace::from_wire(&w, false)?.face()
    }
}
