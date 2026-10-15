// Rust translation of the loop over the faces of ChFi3d_Builder::PerformIntersectionAtEnd
// (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillet cut by each face of the chain: the ends of each piece of the cut on the
//! edges between the faces (extended when needed), and the intersection curve.
use super::ChFi3dBuilder;
use super::at_end::{AtEnd, NN};
use super::corner_tools::{extend_surface, inters_update_on_same};
use super::one_corner::{pcurve_of, value_2d};
use crate::chfi_ds::lock;
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSInterference, TopOpeBRepDSPoint, TopOpeBRepDSSurface,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomBoundedSurface, GeomCurve, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::int_curve_surface::HInter;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS};
use rustycad_tkgeombase::geom_lib::extend_surf_by_length;
use rustycad_tkmath::gp::Point2;

/// The state of the loop over the faces cut by the fillet.
pub(super) struct CutLoop {
    pub proledge: [bool; NN + 1],
    pub prolface: [i32; NN + 1],
    pub shrink: [bool; NN],
    pub faceprol: Vec<Face>,
    pub indcurve: [i32; NN],
    pub indpoint1: i32,
    pub indpoint2: i32,
    pub interfedge: Vec<Handle<TopOpeBRepDSInterference>>,
    pub interf_pc: Vec<Handle<TopOpeBRepDSInterference>>,
    pub interf_ps: Vec<Handle<TopOpeBRepDSInterference>>,
    pub paredge1: f64,
    pub paredge2: f64,
    pub tolex: f64,
    pub extend: bool,
    pub sfacemoins1: Handle<dyn GeomSurface>,
    pub sface: Handle<dyn GeomSurface>,
    pub pfil1: Point2,
    pub pfil2: Point2,
    pub pfac1: Point2,
    pub pfac2: Point2,
    pub pint: Point2,
    pub cc: Handle<dyn GeomCurve>,
    pub ps: Handle<dyn Geom2dCurve>,
    pub pc: Handle<dyn Geom2dCurve>,
    pub p1: f64,
    pub p2: f64,
    pub p2d1: Point2,
    pub p2d2: Point2,
    pub cint: Handle<dyn GeomCurve>,
    pub c2dint1: Handle<dyn Geom2dCurve>,
    pub c2dint2: Handle<dyn Geom2dCurve>,
    pub inters: HInter,
    pub tolreached: f64,
    pub inters_on_same_failed: bool,
    pub hc1: Handle<dyn Geom2dCurve>,
}

impl CutLoop {
    /// The loop before the first face.
    pub(super) fn new() -> Self {
        Self {
            proledge: [false; NN + 1],
            prolface: [0; NN + 1],
            shrink: [false; NN],
            faceprol: vec![Face::default(); NN],
            indcurve: [0; NN],
            indpoint1: 0,
            indpoint2: 0,
            interfedge: vec![Handle::default(); NN],
            interf_pc: vec![Handle::default(); NN],
            interf_ps: vec![Handle::default(); NN],
            paredge1: 0.,
            paredge2: 0.,
            tolex: 1.0e-4,
            extend: false,
            sfacemoins1: Handle::default(),
            sface: Handle::default(),
            pfil1: Point2::default(),
            pfil2: Point2::default(),
            pfac1: Point2::default(),
            pfac2: Point2::default(),
            pint: Point2::default(),
            cc: Handle::default(),
            ps: Handle::default(),
            pc: Handle::default(),
            p1: 0.,
            p2: 0.,
            p2d1: Point2::default(),
            p2d2: Point2::default(),
            cint: Handle::default(),
            c2dint1: Handle::default(),
            c2dint2: Handle::default(),
            inters: HInter::new(),
            tolreached: 0.,
            inters_on_same_failed: false,
            hc1: Handle::default(),
        }
    }
}

/// The point of the face surface nearest to `p` (the parameters left when it fails).
pub(super) fn surface_parameters(
    s: &Handle<dyn GeomSurface>,
    p: rustycad_tkmath::gp::Point3,
    tol: f64,
    uv: &mut Point2,
) -> Result<(), Failure> {
    let asurf = GeomAdaptorSurface::with_surface(s)?;
    let ext = ExtPS::with_surface(p, &asurf, tol, tol, ExtFlag::Min, ExtAlgo::Grad)?;
    if ext.is_done() {
        let (u, v) = ext.point(1)?.parameter();
        *uv = Point2::new(u, v);
    }
    Ok(())
}

impl ChFi3dBuilder {
    /// The face of the chain to cut by at `nb` (extended when it was), as the adaptor of
    /// the cut.
    pub(super) fn init_cut_face(
        &self,
        ae: &AtEnd,
        cl: &CutLoop,
        nb: usize,
        restrict: bool,
    ) -> Result<(), Failure> {
        let f = if cl.prolface[nb - 1] != 0 {
            &cl.faceprol[nb - 1]
        } else {
            &ae.faces[nb - 1]
        };
        ae.hbs.initialize(f, restrict)
    }

    /// The first end of the cut on the side 1 of the fillet (the first face).
    pub(super) fn first_end(
        &mut self,
        ae: &mut AtEnd,
        cl: &mut CutLoop,
        f: &Face,
    ) -> Result<(), Failure> {
        cl.hc1 = BRepTool::curve_on_face(&ae.edges[0], &ae.faces[0])?.pcurve;
        if ae.is_on_same[0] {
            self.init_cut_face(ae, cl, 1, false)?;
            let c3df = self.ds.curve(ae.fi[0].line_index()).curve().clone();
            let mut ufi = ae.fi[1].parameter(ae.isfirst);
            let (hgs, hbs): (Handle<dyn Adaptor3dSurface>, Handle<dyn Adaptor3dSurface>) =
                (ae.hgs.clone().cast(), ae.hbs.clone().cast());
            let ok = {
                let mut g = lock(&ae.fd);
                let (mut fi, mut cp) = (g.interference(1).clone(), g.vertex(ae.isfirst, 1).clone());
                let r = inters_update_on_same(
                    &hgs,
                    &hbs,
                    &c3df,
                    (&ae.f1, &ae.faces[0], &ae.edges[0], &ae.vtx),
                    ae.isfirst,
                    10. * self.tolapp3d,
                    (&mut fi, &mut cp),
                    &mut cl.pfac1,
                    &mut ufi,
                )?;
                *g.change_interference(1) = fi.clone();
                *g.change_vertex(ae.isfirst, 1) = cp;
                ae.fi[0] = fi;
                r
            };
            if !ok {
                return Err(Failure::new("IntersectionAtEnd: pb intersection Face - Fi"));
            }
            if cl.inters_on_same_failed {
                let c = if cl.c2dint2.is_null() {
                    cl.hc1.clone()
                } else {
                    cl.c2dint2.clone()
                };
                cl.paredge2 =
                    ProjectPointOnCurve::with_curve(cl.pfac1, &c)?.lower_distance_parameter()?;
            }
            let cv1 = ae.cv(1);
            cl.indpoint1 = self
                .ds
                .add_point(&TopOpeBRepDSPoint::new(cv1.point(), self.tolapp3d));
            lock(&ae.stripe).set_index_point(cl.indpoint1, ae.isfirst, 1);
            let (vert1, _) = topexp::vertices(&ae.edges[0], false);
            let arc_ori = if ae.vtx.is_same(&vert1) {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            let (tol, e0, pe) = (self.tolapp3d, ae.edges[0].clone(), cl.paredge2);
            ae.change_cv(1, |c| c.set_arc(tol, &e0, pe, arc_ori));
        } else if cl.hc1.is_null() {
            extend_surface(&mut cl.sfacemoins1, &mut cl.prolface[0])?;
            if cl.prolface[0] != 0 {
                cl.extend = true;
                let tol = BRepTool::face_tolerance(f)?;
                BRepBuilder::default().make_face_on_located_surface(
                    &mut cl.faceprol[0],
                    &cl.sfacemoins1,
                    f.location(),
                    tol,
                )?;
                if !ae.is_on_same[0] {
                    surface_parameters(&cl.sfacemoins1, ae.cv(1).point(), tol, &mut cl.pfac1)?;
                }
            }
        } else {
            cl.pfac1 = value_2d(&cl.hc1, ae.cv(1).parameter_on_arc()?)?;
        }
        cl.paredge1 = ae.cv(1).parameter_on_arc()?;
        let fi1 = &ae.fi[0];
        let t = if fi1.line_index() != 0 {
            fi1.parameter(ae.isfirst)
        } else {
            fi1.parameter(!ae.isfirst)
        };
        cl.pfil1 = value_2d(fi1.pcurve_on_surf(), t)?;
        Ok(())
    }

    /// The curve of the edge `e2` restricted to its basis (to half of it on a periodic
    /// one, the half without the start of the edge).
    pub(super) fn edge_curve(
        e2: &Edge,
    ) -> Result<(Handle<dyn GeomCurve>, Handle<GeomAdaptorCurve>), Failure> {
        let (c, mut ubid, mut vbid) = BRepTool::curve(e2)?;
        let ctrim = GeomTrimmedCurve::new(&c, ubid, vbid, true, true)?;
        let basis = ctrim.basis_curve();
        let b = basis
            .get()
            .ok_or_else(|| Failure::new("IntersectionAtEnd"))?;
        let (utrim, vtrim) = (b.first_parameter(), b.last_parameter());
        if ctrim.is_periodic() {
            if ubid > ctrim.period()? {
                ubid = (utrim + vtrim) / 2.;
                vbid = vtrim;
            } else {
                ubid = utrim;
                vbid = (utrim + vtrim) / 2.;
            }
        } else {
            ubid = utrim;
            vbid = vtrim;
        }
        Ok((
            c.clone(),
            Handle::new(GeomAdaptorCurve::with_range(&c, ubid, vbid)?),
        ))
    }

    /// The fillet surface extended at its end when the edge does not cut it (once), and
    /// the surface data moved on the extended surface when the stripe end is not in the
    /// structure yet.
    pub(super) fn extend_fillet(
        &mut self,
        ae: &mut AtEnd,
        cl: &mut CutLoop,
        hc: &Handle<dyn Adaptor3dCurve>,
    ) -> Result<(), Failure> {
        let surf = self.ds.surface(lock(&ae.fd).surf()).surface().clone();
        let mut s1: Handle<dyn GeomBoundedSurface> = surf.cast();
        if s1.is_null() {
            return Ok(());
        }
        let length = 0.5 * ae.fi_length[0].max(ae.fi_length[1]);
        extend_surf_by_length(&mut s1, length, 1, false, !ae.isfirst)?;
        cl.prolface[NN] = 1;
        if lock(&ae.stripe).is_in_ds(!ae.isfirst) != 0 {
            return Ok(());
        }
        let gs1: Handle<dyn GeomSurface> = s1.clone().cast();
        ae.hgs.load(&gs1)?;
        let hgs: Handle<dyn Adaptor3dSurface> = ae.hgs.clone().cast();
        cl.inters.perform(hc, &hgs)?;
        if !cl.inters.is_done() || cl.inters.nb_points()? == 0 {
            return Ok(());
        }
        let tol = self.ds.surface(ae.isurf).tolerance();
        let ns = self.ds.add_surface(&TopOpeBRepDSSurface::new(&gs1, tol));
        lock(&ae.fd).change_surf(ns);
        let key = ae.edge_spine.clone().into();
        if self.my_evi_map.is_bound(&key) {
            let l = self.my_evi_map.change_find(&key)?;
            if let Some(pos) = l.iter().position(|i| *i == ae.isurf) {
                l.remove(pos);
            }
            l.push(ns);
        } else {
            self.my_evi_map.bind(key, vec![ns]);
        }
        ae.isurf = ns;
        Ok(())
    }

    /// The faces around the edge extended when it does not cut the fillet, and the curve
    /// of their intersection cut instead.
    pub(super) fn extend_faces(
        &self,
        ae: &AtEnd,
        cl: &mut CutLoop,
        nb: usize,
        f: &Face,
        (c, hc): (&Handle<dyn GeomCurve>, &Handle<GeomAdaptorCurve>),
    ) -> Result<(), Failure> {
        let is_spline = !c.clone().cast::<GeomBSplineCurve>().is_null()
            || !c.clone().cast::<GeomBezierCurve>().is_null();
        if !is_spline {
            return Ok(());
        }
        cl.sface = BRepTool::surface(&ae.faces[nb])?;
        extend_surface(&mut cl.sface, &mut cl.prolface[nb])?;
        let tol = BRepTool::face_tolerance(f)?;
        BRepBuilder::default().make_face_on_located_surface(
            &mut cl.faceprol[nb],
            &cl.sface,
            ae.faces[nb].location(),
            tol,
        )?;
        if nb != 0 && cl.prolface[nb - 1] == 0 {
            extend_surface(&mut cl.sfacemoins1, &mut cl.prolface[nb - 1])?;
            if cl.prolface[nb - 1] != 0 {
                let tol = BRepTool::face_tolerance(f)?;
                BRepBuilder::default().make_face_on_located_surface(
                    &mut cl.faceprol[nb - 1],
                    &cl.sfacemoins1,
                    f.location(),
                    tol,
                )?;
            }
        } else {
            let mut prol = 0;
            extend_surface(&mut cl.sfacemoins1, &mut prol)?;
        }
        let approx = GeomIntIntSSApprox {
            approx: true,
            approx_s1: true,
            approx_s2: true,
        };
        let inter_ss = GeomIntIntSS::with_surfaces(&cl.sfacemoins1, &cl.sface, 1.0e-7, approx)?;
        if inter_ss.is_done() {
            let hgs: Handle<dyn Adaptor3dSurface> = ae.hgs.clone().cast();
            let hca: Handle<dyn Adaptor3dCurve> = hc.clone().cast();
            let mut trouve = false;
            let mut i = 1;
            while i <= inter_ss.nb_lines() && !trouve {
                cl.extend = true;
                cl.cint = inter_ss.line(i)?;
                cl.c2dint1 = inter_ss.line_on_s1(i)?;
                cl.c2dint2 = inter_ss.line_on_s2(i)?;
                hc.load(&cl.cint)?;
                cl.inters.perform(&hca, &hgs)?;
                trouve = cl.inters.is_done() && cl.inters.nb_points()? != 0;
                cl.tolex = inter_ss.tol_reached3d();
                i += 1;
            }
        }
        Ok(())
    }

    /// The end of the piece on the edge `nb` between the faces: the intersection of the
    /// edge (or of the extended faces) with the fillet nearest to the vertex; false when
    /// the corner must be left to `PerformMoreThreeCorner`.
    pub(super) fn edge_end(
        &mut self,
        ae: &mut AtEnd,
        cl: &mut CutLoop,
        nb: usize,
        f: &Face,
    ) -> Result<bool, Failure> {
        let e2 = ae.edges[nb].clone();
        let (c, hc) = Self::edge_curve(&e2)?;
        let hca: Handle<dyn Adaptor3dCurve> = hc.clone().cast();
        let hgs: Handle<dyn Adaptor3dSurface> = ae.hgs.clone().cast();
        cl.inters.perform(&hca, &hgs)?;
        if cl.prolface[NN] == 0 && (!cl.inters.is_done() || cl.inters.nb_points()? == 0) {
            self.extend_fillet(ae, cl, &hca)?;
        }
        if !cl.inters.is_done() || cl.inters.nb_points()? == 0 {
            self.extend_faces(ae, cl, nb, f, (&c, &hc))?;
        }
        if !cl.inters.is_done() {
            return Err(Failure::new("IntersectionAtEnd: pb intersection Face cb"));
        }
        if cl.inters.nb_points()? == 0 {
            if nb == 0 || nb == ae.nbface {
                cl.inters_on_same_failed = true;
                return Ok(true);
            }
            return Ok(false);
        }
        let p = BRepTool::pnt(&ae.vtx)?;
        let (mut nbp, mut distmin) = (1, p.distance(&cl.inters.point(1)?.pnt()));
        for i in 2..=cl.inters.nb_points()? {
            let d = p.distance(&cl.inters.point(i)?.pnt());
            if d < distmin {
                distmin = d;
                nbp = i;
            }
        }
        let ip = cl.inters.point(nbp)?;
        cl.pfil2 = Point2::new(ip.u(), ip.v());
        cl.paredge2 = ip.w();
        if !cl.extend {
            cl.pfac2 = value_2d(&pcurve_of(&e2, f)?, cl.paredge2)?;
            cl.pint = value_2d(&pcurve_of(&e2, &ae.faces[nb])?, cl.paredge2)?;
        } else if cl.c2dint1.is_null() || cl.c2dint2.is_null() {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "Failed to get p-curve of edge",
            ));
        } else {
            cl.pfac2 = value_2d(&cl.c2dint1, cl.paredge2)?;
            cl.pint = value_2d(&cl.c2dint2, cl.paredge2)?;
        }
        Ok(true)
    }
}
