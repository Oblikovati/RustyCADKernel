// Rust translation of ChFi3d_Builder::PerformOneCorner (ChFi3d_Builder_C1.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cork curve of a corner of one contour and the driver `PerformOneCorner`.
use super::ChFi3dBuilder;
use super::compute::spine_of;
use super::corner_tools::{recale, select_stripe};
use super::one_corner::Cork;
use super::one_corner::OneCorner;
use super::one_corner::pcurve_of;
use super::one_corner::value_2d;
use crate::chfi_ds::{State, lock};
use crate::chfi3d::{IntersectionCurves, boite, bound_fac, bound_surf, compute_curves, couture};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSPoint};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::extrema::ExtCC;
use rustycad_tkmath::gp::Point2;

impl ChFi3dBuilder {
    /// The cork: the intersection of the fillet with the face at the end between the ends
    /// of the fillet, split where it crosses the seam of the face.
    pub(super) fn cork_curve(&mut self, oc: &OneCorner) -> Result<Cork, Failure> {
        let hgs: Handle<dyn Adaptor3dSurface> = bound_surf(&self.ds, &oc.fd, 1, 2)?.cast();
        let (cv1, cv2) = {
            let g = lock(&oc.fd);
            (
                g.vertex(oc.isfirst, 1).clone(),
                g.vertex(oc.isfirst, 2).clone(),
            )
        };
        let mut pfac1 = if oc.onsame && oc.ifop == 1 {
            oc.p2dbout
        } else {
            value_2d(&pcurve_of(cv1.arc()?, &oc.fv)?, cv1.parameter_on_arc()?)?
        };
        let mut pfac2 = if oc.onsame && oc.ifop == 2 {
            oc.p2dbout
        } else {
            value_2d(&pcurve_of(cv2.arc()?, &oc.fv)?, cv2.parameter_on_arc()?)?
        };
        let pfil1 = Self::fillet_end_point(&oc.fd, 1, oc.isfirst)?;
        let pfil2 = Self::fillet_end_point(&oc.fd, 2, oc.isfirst)?;
        if oc.onsame {
            recale(&oc.hbs, &mut pfac1, &mut pfac2, oc.ifad == 1)?;
        }
        let pardeb = [pfil1.x(), pfil1.y(), pfac1.x(), pfac1.y()];
        let parfin = [pfil2.x(), pfil2.y(), pfac2.x(), pfac2.y()];
        bound_fac(&oc.hbs, boite(pfac1, pfac2), true)?;
        let hbs: Handle<dyn Adaptor3dSurface> = oc.hbs.clone().cast();
        let mut out = IntersectionCurves::default();
        if !compute_curves(
            (&hgs, &hbs),
            (pardeb, parfin),
            (self.tolapp3d, self.tol2d),
            // The native default wholeCurv = true.
            true,
            &mut out,
        )? {
            return Err(Failure::new("OneCorner : echec calcul intersection"));
        }
        let c = out.c3d.get().ok_or_else(|| Failure::new("OneCorner"))?;
        let mut cork = Cork {
            udeb: c.first_parameter(),
            ufin: c.last_parameter(),
            cc: out.c3d.clone(),
            ps: out.pc1,
            pc: out.pc2,
            tolreached: out.tolreached,
            ..Cork::default()
        };
        if let Some(edgecouture) = couture(&oc.fv)? {
            cork.edgecouture = edgecouture.clone();
            if !BRepTool::degenerated(&edgecouture)? {
                self.split_at_seam(&mut cork, &edgecouture)?;
            }
        }
        Ok(cork)
    }

    /// The cork split at the seam of the face when it crosses it inside.
    pub(super) fn split_at_seam(
        &mut self,
        cork: &mut Cork,
        edgecouture: &Edge,
    ) -> Result<(), Failure> {
        let (c, ubid, vbid) = BRepTool::curve(edgecouture)?;
        let ctrim = GeomTrimmedCurve::new(&c, ubid, vbid, true, true)?;
        let cur1 = GeomAdaptorCurve::with_curve(&ctrim.basis_curve())?;
        let cur2 = GeomAdaptorCurve::with_curve(&cork.cc)?;
        let ext = ExtCC::with_curves(&cur1, &cur2, 1.0e-10, 1.0e-10)?;
        if !ext.is_done() || ext.nb_ext()? == 0 {
            return Ok(());
        }
        let (mut imin, mut distmin2) = (0, f64::MAX);
        for i in 1..=ext.nb_ext()? {
            if ext.square_distance(i)? < distmin2 {
                distmin2 = ext.square_distance(i)?;
                imin = i;
            }
        }
        if distmin2 > precision::square_confusion() {
            return Ok(());
        }
        let (ponc1, ponc2) = ext.points(imin)?;
        cork.par1 = ponc1.parameter();
        cork.par2 = ponc2.parameter();
        let tol = 1.0e-4;
        if (cork.par2 - cork.udeb).abs() > tol && (cork.ufin - cork.par2).abs() > tol {
            cork.indpt = self
                .ds
                .add_point(&TopOpeBRepDSPoint::new(*ponc1.value(), tol));
            cork.intcouture = true;
            self.add_split_curves(cork, (cork.udeb, cork.par2, cork.ufin))?;
        }
        Ok(())
    }

    /// The two parts of the cork curve at `par2`, added to the data structure.
    pub(super) fn add_split_curves(
        &mut self,
        cork: &mut Cork,
        (udeb, par2, ufin): (f64, f64, f64),
    ) -> Result<(), Failure> {
        let c1: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&cork.cc, udeb, par2, true, true)?).cast();
        let c2: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&cork.cc, par2, ufin, true, true)?).cast();
        cork.icurv = [
            self.ds
                .add_curve(&mut TopOpeBRepDSCurve::new(&c1, cork.tolreached, false)),
            self.ds
                .add_curve(&mut TopOpeBRepDSCurve::new(&c2, cork.tolreached, false)),
        ];
        cork.curv = [c1, c2];
        Ok(())
    }

    /// `PerformOneCorner(Index, thePrepareOnSame)`: the corner of one contour at the
    /// `index`-th vertex; only the end of the fillet on the opposite face moved when
    /// preparing OnSame.
    pub(crate) fn perform_one_corner(
        &mut self,
        index: i32,
        prepare_on_same: bool,
    ) -> Result<(), Failure> {
        let vtx = self.my_vdata_map.find_key(index)?;
        let list = self.my_vdata_map.find_from_index(index)?.to_vec();
        let Some(k) = select_stripe(&list, &vtx, prepare_on_same)? else {
            return Ok(());
        };
        let stripe = list[k].clone();
        let spine = spine_of(&stripe)?;
        let (fd, isfirst) = self.corner_surf_data(&stripe, &vtx)?;
        let stat = lock(&spine).status(isfirst);
        let arcspine = {
            let sp = lock(&spine);
            if isfirst {
                sp.edges(1)?.clone()
            } else {
                sp.edges(sp.nb_edges())?.clone()
            }
        };
        let fop = topods::face(self.ds.shape(lock(&fd).index(2), true))?;
        let mut oc = OneCorner {
            stripe,
            fd,
            isfirst,
            onsame: stat == State::OnSame,
            fv: Face::default(),
            fad: Face::default(),
            fop,
            arcprol: Edge::default(),
            arcspine,
            oarcprolv: Orientation::Forward,
            oarcprolop: Orientation::Forward,
            hbs: Handle::new(BRepAdaptorSurface::default()),
            hbop: Handle::new(BRepAdaptorSurface::default()),
            ifad: 1,
            ifop: 2,
            p2dbout: Point2::default(),
        };
        if oc.onsame && !self.on_same_faces(&mut oc, &vtx)? {
            return self.perform_intersection_at_end(index);
        }
        let inters = match self.update_corner_ends(&mut oc, &vtx, prepare_on_same)? {
            None => return Ok(()),
            Some(false) if !oc.onsame => return self.perform_intersection_at_end(index),
            Some(i) => i,
        };
        if !inters {
            return Err(Failure::with_message(
                "Standard_NotImplemented",
                "OneCorner : bouchon non ecrit",
            ));
        }
        let mut cork = self.cork_curve(&oc)?;
        self.write_cork(&oc, &vtx, &mut cork)?;
        Ok(())
    }
}
