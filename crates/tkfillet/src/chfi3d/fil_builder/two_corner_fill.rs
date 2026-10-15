// Rust translation of the corner surface of ChFi3d_FilBuilder::PerformTwoCorner and the
// file static ToricRotule (ChFi3d_FilBuilder_C2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner surface of two fillets on the same side of the common face: a torus
//! between three planes, or a filling of the fillets, the common face and the pivot.
use super::two_corner::{CornerFaces, Pivot, TwoCorner, face_adaptor};
use super::{ChFi3dFilBuilder, constant_radius};
use crate::chfi_ds::{Stripe, StripeHandle, SurfData, SurfDataHandle, lock, shared};
use crate::chfi3d::builder::spine_of;
use crate::chfi3d::{concave_side, mkbound_geom, mkbound_tangents_2d, next_side};
use crate::chfikpart::ChFiKPartComputeData;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, reverse};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillBoundary, GeomFillConstrainedFilling, GeomFillSimpleBound,
};
use rustycad_tkgeombase::geom2d_convert;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Point2, Vector2};

/// `ToricRotule(fac, s1, s2, c1, c2)`: three planes, the face perpendicular to the two
/// others, and two constant fillets of the same radius.
fn toric_rotule(
    fac: &BRepAdaptorSurface,
    s1: &BRepAdaptorSurface,
    s2: &BRepAdaptorSurface,
    c: [&StripeHandle; 2],
) -> Result<bool, Failure> {
    let tolesp = 1.0e-7;
    let r = |st: &StripeHandle| -> Result<Option<f64>, Failure> {
        let sp = spine_of(st)?;
        let g = lock(&sp);
        match g.as_fil_spine() {
            Some(f) if f.is_constant() => Ok(Some(f.radius()?)),
            _ => Ok(None),
        }
    };
    let (Some(r1), Some(r2)) = (r(c[0])?, r(c[1])?) else {
        return Ok(false);
    };
    if fac.get_type()? != SurfaceType::Plane
        || s1.get_type()? != SurfaceType::Plane
        || s2.get_type()? != SurfaceType::Plane
    {
        return Ok(false);
    }
    let df = fac.plane()?.position().direction();
    let ds1 = s1.plane()?.position().direction();
    let ds2 = s2.plane()?.position().direction();
    if df.dot(ds1).abs() >= tolesp || df.dot(ds2).abs() >= tolesp {
        return Ok(false);
    }
    Ok((r1 - r2).abs() < tolesp)
}

/// The corner surface and how it meets the pivot: the face of the pivot and the pcurve
/// of the pivot on it (none when the fillets meet the pivot at the same point).
pub(super) struct CornerSurface {
    pub coin: SurfDataHandle,
    pub corner: StripeHandle,
    pub fapiv: Face,
    pub pctrans: Orientation,
    pub resetcp: [bool; 2],
}

impl ChFi3dFilBuilder {
    /// The corner surface between the two fillets (the "cornerdata"), written with its
    /// edges when it is computed; added to the contours in every case.
    pub(super) fn corner_surface(
        &mut self,
        tc: &TwoCorner,
        piv: &Pivot,
        ctx: &CornerFaces,
    ) -> Result<(), Failure> {
        let coin = shared(SurfData::new());
        let mut stripe = Stripe::new();
        *stripe.change_set_of_surf_data() = Some(shared(vec![coin.clone()]));
        let mut cs = CornerSurface {
            coin,
            corner: shared(stripe),
            fapiv: Face::default(),
            pctrans: Orientation::Forward,
            resetcp: [false, false],
        };
        if tc.res.sameside {
            let hfaco = face_adaptor(&ctx.faco)?;
            let hbrs = [face_adaptor(&ctx.ff[0])?, face_adaptor(&ctx.ff[1])?];
            self.base.done = if toric_rotule(&hfaco, &hbrs[0], &hbrs[1], [&tc.st[0], &tc.st[1]])? {
                self.toric_corner(tc, piv, ctx, &cs, (&hfaco, &hbrs))?
            } else {
                self.filled_corner(tc, piv, ctx, &mut cs, (&hfaco, &hbrs))?
            };
            if self.base.done {
                self.write_same_side_corner(tc, piv, ctx, &cs)?;
            }
        } else {
            self.on_diff_corner(tc, ctx, &cs)?;
        }
        let isurf = lock(&cs.coin).surf();
        self.append_evi(&tc.vtx, isurf);
        self.base.my_list_stripe.push(cs.corner);
        Ok(())
    }

    /// `myEVIMap.ChangeFind(Vtx).Append(isurf)`, the list bound first when needed.
    pub(super) fn append_evi(&mut self, vtx: &Vertex, isurf: i32) {
        let key: rustycad_tkbrep::topods::Shape = vtx.clone().into();
        if !self.base.my_evi_map.is_bound(&key) {
            self.base.my_evi_map.bind(key.clone(), Vec::new());
        }
        if let Some(l) = self.base.my_evi_map.change_seek(&key) {
            l.push(isurf);
        }
    }

    /// The direct construction: the torus joint of `ChFiKPart_ComputeData::ComputeCorner`.
    fn toric_corner(
        &mut self,
        tc: &TwoCorner,
        piv: &Pivot,
        ctx: &CornerFaces,
        cs: &CornerSurface,
        (hfaco, hbrs): (
            &Handle<BRepAdaptorSurface>,
            &[Handle<BRepAdaptorSurface>; 2],
        ),
    ) -> Result<bool, Failure> {
        let ofaco = ctx.faco.orientation();
        let mut ori = ofaco;
        let (oris, orisff1) = {
            let s = lock(&tc.st[0]);
            (
                s.orientation_on_face(tc.ifaco(0)),
                s.orientation_on_face(tc.ifaarc(0)),
            )
        };
        let mut off1 = ctx.ff[0].orientation();
        next_side(&mut ori, &mut off1, oris, orisff1, 1);
        let (mut op1, mut op2) = (Orientation::Forward, Orientation::Forward);
        if piv.yapiv {
            let (_, o1, o2) = concave_side(&hbrs[0], &hbrs[1], &piv.edge)?;
            (op1, op2) = (o1, o2);
        }
        let (op1, op2) = (reverse(op1), reverse(op2));
        let radius = constant_radius(&spine_of(&tc.st[0])?, "PerformTwoCorner")?.unwrap_or(0.);
        let s: Handle<dyn Adaptor3dSurface> = hfaco.clone().cast();
        let s1: Handle<dyn Adaptor3dSurface> = hbrs[0].clone().cast();
        let s2: Handle<dyn Adaptor3dSurface> = hbrs[1].clone().cast();
        ChFiKPartComputeData::compute_corner_rotule(
            &mut self.base.ds,
            &cs.coin,
            (&s, &s1, &s2),
            (ofaco, ori, op1, op2),
            radius,
        )
    }

    /// The construction by filling: the boundaries on the two fillets, on the common
    /// face and on the pivot (unless the fillets meet it at the same point).
    fn filled_corner(
        &mut self,
        tc: &TwoCorner,
        piv: &Pivot,
        ctx: &CornerFaces,
        cs: &mut CornerSurface,
        (hfaco, hbrs): (
            &Handle<BRepAdaptorSurface>,
            &[Handle<BRepAdaptorSurface>; 2],
        ),
    ) -> Result<bool, Failure> {
        let (t3d, ta) = (self.base.tolapp3d, 2.0e-4);
        let mut b: Vec<Handle<dyn GeomFillBoundary>> = Vec::new();
        let mut fac_ends: Vec<(i32, Point2, Vector2)> = Vec::new();
        for k in 0..2 {
            let fiarc = tc.interference(k, tc.ifaarc(k));
            let fico = tc.interference(k, tc.ifaco(k));
            let upc = fiarc.parameter(tc.isfirst(k));
            let p2da = fiarc.pcurve_on_surf().value(upc)?;
            let p2df = fico.pcurve_on_surf().value(upc)?;
            let (p2dfac, v2dfac) = fico.pcurve_on_face().d1(upc)?;
            let surf = self
                .base
                .ds
                .surface(lock(&tc.sd[k]).surf())
                .surface()
                .clone();
            b.push(mkbound_geom(&surf, p2df, p2da, (t3d, ta), false)?);
            fac_ends.push((tc.sens[k], p2dfac, v2dfac));
        }
        let gfaco: Handle<dyn Adaptor3dSurface> = hfaco.clone().cast();
        let (bfac, pcurve_on_face) =
            mkbound_tangents_2d(&gfaco, fac_ends[0], fac_ends[1], (t3d, ta))?;
        let mut fil = GeomFillConstrainedFilling::new(8, 20);
        let mut hfapiv: Handle<dyn Adaptor3dSurface> = Handle::default();
        let mut pcurve_on_piv: Handle<dyn Geom2dCurve> = Handle::default();
        if piv.sameparam {
            fil.init_three([&bfac, &b[1], &b[0]], true)?;
        } else {
            let (lo, hi) = (piv.par.0.min(piv.par.1), piv.par.0.max(piv.par.1));
            let hpiv = piv.hpivot.get().ok_or_else(|| {
                Failure::with_message("Standard_NullObject", "PerformTwoCorner: no pivot")
            })?;
            let trim = hpiv.trim(lo, hi, self.base.tolesp)?;
            let bpiv: Handle<dyn GeomFillBoundary> =
                Handle::new(GeomFillSimpleBound::new(&trim, t3d, ta)).cast();
            fil.init_four([&bfac, &b[1], &bpiv, &b[0]], true)?;
            let (_, darc) = hpiv.d1(piv.par.0)?;
            let b1 = b[0]
                .get()
                .ok_or_else(|| Failure::new("PerformTwoCorner: null bound"))?;
            let (_, lb1) = b1.bounds()?;
            let (_, dcf) = b1.d1(lb1)?;
            let pivotverslebas = darc.dot(dcf) <= 0.;
            let pcfalenvers = piv.par.0 > piv.par.1;
            let k = if pivotverslebas != pcfalenvers { 1 } else { 0 };
            cs.resetcp[k] = true;
            let mut fapiv = ctx.ff[k].clone();
            hfapiv = hbrs[k].clone().cast();
            fapiv.set_orientation(Orientation::Forward);
            let (pc, pctrans) = self.pcurve_on_pivot(&piv.edge, &fapiv, (lo, hi), pcfalenvers)?;
            pcurve_on_piv = pc;
            cs.pctrans = pctrans;
            cs.fapiv = fapiv;
        }
        let surfcoin = fil.surface().cast();
        self.base.complete_data_coin(
            &cs.coin,
            &surfcoin,
            (&gfaco, &pcurve_on_face),
            (&hfapiv, &pcurve_on_piv),
            ctx.faco.orientation(),
            true,
            [false; 4],
        )
    }

    /// The pcurve of the pivot on its face between the parameters of the fillets,
    /// reparameterized on [0, 1] and reversed when the fillets reach it backwards, and its
    /// transition.
    fn pcurve_on_pivot(
        &self,
        pivot: &rustycad_tkbrep::topods::Edge,
        fapiv: &Face,
        (lo, hi): (f64, f64),
        pcfalenvers: bool,
    ) -> Result<(Handle<dyn Geom2dCurve>, Orientation), Failure> {
        let pcpivot = BRepAdaptorCurve2d::with_edge_on_face(pivot, fapiv)?;
        let mut pctrans = Orientation::Forward;
        let mut ex = Explorer::new(fapiv.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
        while ex.more() {
            if ex.current().is_same(pivot) {
                pctrans = ex.current().orientation();
                break;
            }
            ex.next()?;
        }
        let pc: Handle<Geom2dBSplineCurve> = if pcpivot.get_type()? != CurveType::BSplineCurve {
            let trc: Handle<dyn Geom2dCurve> = Handle::new(Geom2dTrimmedCurve::new(
                &pcpivot.geom2d_adaptor().curve(),
                lo,
                hi,
                true,
                true,
            )?)
            .cast();
            geom2d_convert::curve_to_bspline_curve(&trc, ParameterisationType::TgtThetaOver2)?
        } else {
            geom2d_convert::split_bspline_curve(
                &pcpivot.geom2d_adaptor().curve().cast(),
                lo,
                hi,
                self.base.tol2d,
                true,
            )?
        };
        let curve = pc
            .get()
            .ok_or_else(|| Failure::new("PerformTwoCorner: null pcurve on the pivot"))?;
        let mut kk = curve.knots();
        bsplclib::reparametrize(0., 1., &mut kk)?;
        curve.set_knots(&kk)?;
        if pcfalenvers {
            curve.reverse()?;
            pctrans = reverse(pctrans);
        }
        Ok((pc.cast(), pctrans))
    }
}
