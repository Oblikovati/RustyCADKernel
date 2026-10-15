// Rust translation of the inverted concavities of ChFi3d_FilBuilder::PerformTwoCorner
// (ChFi3d_FilBuilder_C2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of two fillets on opposite sides of the common face (one end OnSame, the
//! other OnDiff): a filling of the two fillets and of the face at the end of the OnSame
//! one, closed on the arc of the OnDiff one.
use super::ChFi3dFilBuilder;
use super::two_corner::{CornerFaces, TwoCorner};
use super::two_corner_fill::CornerSurface;
use crate::chfi_ds::{CommonPoint, Regul, State, lock};
use crate::chfi3d::{
    Arete, build_pcurve_3d, compute_arete, fil_curve_in_ds, index_point_in_ds, mkbound,
    mkbound_segment, same_parameter,
};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSCurve;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::{Orientation, reverse};
use rustycad_tkgeomalgo::geom_fill::GeomFillConstrainedFilling;
use rustycad_tkmath::gp::Point2;

/// The fillets of the inverted concavities: `sam` OnSame, `dif` OnDiff (indices in the
/// corner).
struct SamDif {
    sam: usize,
    dif: usize,
}

/// The filling of the inverted corner and what its edges need: the fillet surfaces with
/// the pcurves of the boundaries, and the pcurve on the face at the end.
struct DiffFill {
    hsurfdif: Handle<dyn Adaptor3dSurface>,
    pcsurfdif: Handle<dyn Geom2dCurve>,
    hbrfopsam: Handle<dyn Adaptor3dSurface>,
    pcfopsam: Handle<dyn Geom2dCurve>,
    cpopdif: CommonPoint,
}

/// The surface of a fillet in the data structure as an adaptor.
fn fillet_adaptor(s: &Handle<dyn GeomSurface>) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    Ok(Handle::new(GeomAdaptorSurface::with_surface(s)?).cast())
}

impl ChFi3dFilBuilder {
    /// Which fillet is OnSame and which OnDiff (Standard_Failure for another
    /// configuration).
    fn sam_dif(ctx: &CornerFaces) -> Result<SamDif, Failure> {
        match (ctx.stat[0], ctx.stat[1]) {
            (State::OnSame, State::OnDiff) => Ok(SamDif { sam: 0, dif: 1 }),
            (State::OnDiff, State::OnSame) => Ok(SamDif { sam: 1, dif: 0 }),
            _ => Err(Failure::new("TwoCorner : Config unknown")),
        }
    }

    /// The corner of inverted concavities: the filling, then its two edges and the ends
    /// of the contours (Standard_Failure when it cannot be built).
    pub(super) fn on_diff_corner(
        &mut self,
        tc: &TwoCorner,
        ctx: &CornerFaces,
        cs: &CornerSurface,
    ) -> Result<(), Failure> {
        if !tc.okinter {
            return Err(Failure::new("TwoCorner : No intersection pc pc"));
        }
        let sd = Self::sam_dif(ctx)?;
        let fill = self.diff_filling(tc, &sd, cs)?;
        self.base.done = true;
        self.write_diff_corner(tc, &sd, cs, fill)
    }

    /// The filling of the fillets and of the face at the end of the OnSame one.
    fn diff_filling(
        &mut self,
        tc: &TwoCorner,
        sd: &SamDif,
        cs: &CornerSurface,
    ) -> Result<DiffFill, Failure> {
        let (sam, dif) = (sd.sam, sd.dif);
        let (ifacosam, ifaopsam) = (tc.ifaco(sam), tc.ifaarc(sam));
        let (ifacodif, ifaopdif) = (tc.ifaco(dif), tc.ifaarc(dif));
        let (usam, udif) = (tc.uintpc(sam), tc.uintpc(dif));
        let cpopdif = tc.vertex(dif, ifaopdif);
        if !cpopdif.is_on_arc() {
            return Err(Failure::new(
                "TwoCorner : No point on restriction on surface OnDiff",
            ));
        }
        let arcopdif = cpopdif.arc()?.clone();
        let fopsam = topods::face(self.base.ds.shape(lock(&tc.sd[sam]).index(ifaopsam), true))?;
        let (t3d, ta) = (self.base.tolapp3d, 2.0e-4);
        let fisam_op = tc.interference(sam, ifaopsam);
        let ppopsam = fisam_op.pcurve_on_surf().value(usam)?;
        let ppcosam = tc
            .interference(sam, ifacosam)
            .pcurve_on_surf()
            .value(usam)?;
        let hsurfsam = fillet_adaptor(self.base.ds.surface(lock(&tc.sd[sam]).surf()).surface())?;
        let (bsam, pcsurfsam) = mkbound_segment(&hsurfsam, ppopsam, ppcosam, (t3d, ta), false)?;
        let upcopdif = tc.interference(dif, ifaopdif).parameter(tc.isfirst(dif));
        let ppopdif = tc
            .interference(dif, ifaopdif)
            .pcurve_on_surf()
            .value(upcopdif)?;
        let ppcodif = tc
            .interference(dif, ifacodif)
            .pcurve_on_surf()
            .value(udif)?;
        let hsurfdif = fillet_adaptor(self.base.ds.surface(lock(&tc.sd[dif]).surf()).surface())?;
        let (bdif, pcsurfdif) = mkbound_segment(&hsurfdif, ppcodif, ppopdif, (t3d, ta), false)?;
        let ppfacsam = fisam_op.pcurve_on_face().value(usam)?;
        let curvopsam = self.base.ds.curve(fisam_op.line_index()).curve().clone();
        let (_, vvfacsam) = curvopsam.d1(usam)?;
        let mut ppfacdif: Point2 = BRepAdaptorCurve2d::with_edge_on_face(&arcopdif, &fopsam)?
            .value(cpopdif.parameter_on_arc()?)?;
        let surfopsam = BRepAdaptorSurface::with_face(&fopsam, true)?;
        if surfopsam.is_u_closed()? {
            // jgv for OCC26173
            let uperiod = surfopsam.last_u_parameter()? - surfopsam.first_u_parameter()?;
            if (ppfacsam.x() - ppfacdif.x()).abs() > uperiod / 2. {
                let shift = if ppfacdif.x() < ppfacsam.x() {
                    uperiod
                } else {
                    -uperiod
                };
                ppfacdif.set_x(ppfacdif.x() + shift);
            }
        }
        let (_, vvfacdif) =
            BRepAdaptorCurve::with_edge(&arcopdif)?.d1(cpopdif.parameter_on_arc()?)?;
        let hbrfopsam: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&fopsam, false)?).cast();
        let pcfopsam = build_pcurve_3d(&hbrfopsam, ppfacsam, vvfacsam, ppfacdif, vvfacdif, true)?;
        let bfac = mkbound(&hbrfopsam, &pcfopsam, t3d, ta, false)?;
        let mut fil = GeomFillConstrainedFilling::new(8, 20);
        fil.init_three([&bsam, &bdif, &bfac], true)?;
        let surfcoin = fil.surface().cast();
        let osurfsam = lock(&tc.sd[sam]).orientation();
        let done = self.base.complete_data_coin(
            &cs.coin,
            &surfcoin,
            (&hsurfsam, &pcsurfsam),
            (&hbrfopsam, &Handle::default()),
            osurfsam,
            true,
            [false; 4],
        )?;
        if !done {
            return Err(Failure::new("concavites inverted : fail"));
        }
        Ok(DiffFill {
            hsurfdif,
            pcsurfdif,
            hbrfopsam,
            pcfopsam,
            cpopdif,
        })
    }

    /// The edge of the corner surface at one end: its curve and pcurve on the corner.
    fn diff_arete(&self, cs: &CornerSurface, first: bool) -> Result<Arete, Failure> {
        let (p1, p2, pp1, pp2) = {
            let g = lock(&cs.coin);
            let at = |on_s: i32| -> Result<Point2, Failure> {
                let fi = g.interference(on_s);
                let u = if first {
                    fi.first_parameter()
                } else {
                    fi.last_parameter()
                };
                fi.pcurve_on_surf().value(u)
            };
            (
                g.vertex(first, 1).clone(),
                g.vertex(first, 2).clone(),
                at(1)?,
                at(2)?,
            )
        };
        let surf = self
            .base
            .ds
            .surface(lock(&cs.coin).surf())
            .surface()
            .clone();
        let mut arete = Arete::default();
        compute_arete(
            (&p1, pp1),
            (&p2, pp2),
            &surf,
            &mut arete,
            (self.base.tolapp3d, self.base.tol2d),
            0,
        )?;
        Ok(arete)
    }

    /// The edges of the inverted corner and the ends of the two contours.
    fn write_diff_corner(
        &mut self,
        tc: &TwoCorner,
        sd: &SamDif,
        cs: &CornerSurface,
        mut fill: DiffFill,
    ) -> Result<(), Failure> {
        let (sam, dif) = (sd.sam, sd.dif);
        let (ifacosam, ifaopsam) = (tc.ifaco(sam), tc.ifaarc(sam));
        let (ifacodif, ifaopdif) = (tc.ifaco(dif), tc.ifaarc(dif));
        {
            let mut g = lock(&cs.coin);
            *g.change_vertex(true, 2) = fill.cpopdif.clone();
            *g.change_vertex(false, 2) = fill.cpopdif.clone();
        }
        let (pf1, pl1) = {
            let g = lock(&cs.coin);
            (g.vertex(true, 1).clone(), g.vertex(false, 1).clone())
        };
        let ds = &mut self.base.ds;
        let if1 = index_point_in_ds(&pf1, ds)?;
        let if2 = index_point_in_ds(&fill.cpopdif, ds)?;
        let il1 = index_point_in_ds(&pl1, ds)?;
        let il2 = if2;
        let adeb = self.diff_arete(cs, true)?;
        let hc3d: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_curve(&adeb.c3d)?).cast();
        let mut tolr1 = 0.;
        same_parameter(
            &hc3d,
            &mut fill.pcfopsam,
            &fill.hbrfopsam,
            self.base.tolapp3d,
            &mut tolr1,
        )?;
        let icf = self.base.ds.add_curve(&mut TopOpeBRepDSCurve::new(
            &adeb.c3d,
            adeb.tolreached.max(tolr1),
            false,
        ));
        let (mut opcfopsam, ifopsam) = {
            let g = lock(&tc.sd[sam]);
            (g.interference(ifaopsam).transition(), g.index(ifaopsam))
        };
        if tc.isfirst(sam) {
            opcfopsam = reverse(opcfopsam);
        }
        let interf = fil_curve_in_ds(icf, ifopsam, &fill.pcfopsam, opcfopsam);
        self.base
            .ds
            .change_shape_interferences_index(ifopsam)?
            .append(interf);
        let coin_surf = lock(&cs.coin).surf();
        let mut regdeb = Regul::new();
        regdeb.set_curve(icf);
        regdeb.set_s1(coin_surf, false);
        regdeb.set_s2(ifopsam, true);
        self.base.my_regul.push(regdeb);
        {
            let mut c = lock(&cs.corner);
            *c.change_first_pcurve() = adeb.pcurv.clone();
            c.change_first_curve(icf);
            c.change_first_parameters(adeb.pardeb, adeb.parfin);
            c.change_index_first_point_on_s1(if1);
            c.change_index_first_point_on_s2(if2);
        }
        let afin = self.diff_arete(cs, false)?;
        let hc3d: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_curve(&afin.c3d)?).cast();
        let mut tolr2 = 0.;
        same_parameter(
            &hc3d,
            &mut fill.pcsurfdif,
            &fill.hsurfdif,
            self.base.tolapp3d,
            &mut tolr2,
        )?;
        let icl = self.base.ds.add_curve(&mut TopOpeBRepDSCurve::new(
            &afin.c3d,
            afin.tolreached.max(tolr2),
            false,
        ));
        let mut regfin = Regul::new();
        regfin.set_curve(icl);
        regfin.set_s1(coin_surf, false);
        regfin.set_s2(lock(&tc.sd[dif]).surf(), false);
        self.base.my_regul.push(regfin);
        {
            let mut c = lock(&cs.corner);
            *c.change_last_pcurve() = afin.pcurv.clone();
            c.change_last_curve(icl);
            c.change_last_parameters(afin.pardeb, afin.parfin);
            c.change_index_last_point_on_s1(il1);
            c.change_index_last_point_on_s2(il2);
        }
        let sdsam_surf = lock(&tc.sd[sam]).surf();
        {
            let mut g = lock(&cs.coin);
            g.change_index_of_s1(-sdsam_surf);
            g.change_index_of_s2(0);
        }
        let solid = lock(&tc.st[sam]).solid_index();
        lock(&cs.corner).set_solid_index(solid);
        let intcoin1 = lock(&cs.coin).interference_on_s1().clone();
        let fsam = tc.isfirst(sam);
        {
            let mut s = lock(&tc.st[sam]);
            s.set_curve(intcoin1.line_index(), fsam);
            s.in_ds(fsam, 1);
            *s.change_pcurve(fsam) = intcoin1.pcurve_on_face().clone();
            s.set_index_point(if1, fsam, ifaopsam);
            s.set_index_point(il1, fsam, ifacosam);
            s.set_parameters(fsam, intcoin1.first_parameter(), intcoin1.last_parameter());
            if ifaopsam == 2 {
                s.set_orientation_at_end(Orientation::Reversed, fsam);
            }
        }
        {
            let mut g = lock(&tc.sd[sam]);
            *g.change_vertex(fsam, ifaopsam) = pf1.clone();
            *g.change_vertex(fsam, ifacosam) = pl1.clone();
            g.change_interference_on_s1()
                .set_parameter(tc.uintpc(sam), fsam);
            g.change_interference_on_s2()
                .set_parameter(tc.uintpc(sam), fsam);
        }
        let fdif = tc.isfirst(dif);
        {
            let mut s = lock(&tc.st[dif]);
            s.set_curve(icl, fdif);
            *s.change_pcurve(fdif) = fill.pcsurfdif.clone();
            s.set_index_point(il2, fdif, ifaopdif);
            s.set_index_point(il1, fdif, ifacodif);
            s.set_parameters(fdif, afin.pardeb, afin.parfin);
            if ifaopdif == 1 {
                s.set_orientation_at_end(Orientation::Reversed, fdif);
            }
        }
        let mut g = lock(&tc.sd[dif]);
        *g.change_vertex(fdif, ifaopdif) = fill.cpopdif.clone();
        *g.change_vertex(fdif, ifacodif) = pl1;
        g.change_interference(ifacodif)
            .set_parameter(tc.uintpc(dif), fdif);
        Ok(())
    }
}
