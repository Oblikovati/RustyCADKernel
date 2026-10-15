// Rust translation of ChFi3d_Builder::PerformMoreSurfdata (ChFi3d_Builder_C1.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of one contour whose two last surface data meet the face closing it: the
//! last one is removed and the hole closed by the intersections of the face with the
//! two fillets and the cut of the last fillet.
use super::ChFi3dBuilder;
use crate::chfi_ds::{CommonPoint, FaceInterference, StripeHandle, SurfDataHandle, lock};
use crate::chfi3d::{Arete, compute_arete, eval_tol_reached, index_of_surf_data};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::gp::Point3;

/// The tolerance of the comparisons of `PerformMoreSurfdata`.
pub(super) const TOL3D: f64 = 1.0e-4;

/// One intersection curve of two fillet surfaces with its pcurves and tolerance.
pub(super) struct SurfCut {
    pub c: Handle<dyn GeomCurve>,
    pub pc1: Handle<dyn Geom2dCurve>,
    pub pc2: Handle<dyn Geom2dCurve>,
    pub tol: f64,
}

/// The ends of a curve.
pub(super) fn ends(c: &Handle<dyn GeomCurve>) -> Result<(Point3, Point3), Failure> {
    let g = c
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "PerformMoreSurfdata"))?;
    Ok((g.value(g.first_parameter())?, g.value(g.last_parameter())?))
}

/// The intersection line of the two surfaces with an end at `p` (`GeomInt_IntSS` with
/// approximations), the last one tried when none: None when the intersection fails or
/// no line ends there.
fn cut_ending_at(
    s1: &Handle<dyn GeomSurface>,
    s2: &Handle<dyn GeomSurface>,
    p: Point3,
) -> Result<Option<SurfCut>, Failure> {
    let mut inter = GeomIntIntSS::new();
    inter.perform(
        s1,
        s2,
        1.0e-7,
        GeomIntIntSSApprox {
            approx: true,
            approx_s1: true,
            approx_s2: true,
        },
    )?;
    if !inter.is_done() {
        return Ok(None);
    }
    let h1: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(s1)?).cast();
    let h2: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(s2)?).cast();
    for i in 1..=inter.nb_lines() {
        let (c, pc1, pc2) = (inter.line(i)?, inter.line_on_s1(i)?, inter.line_on_s2(i)?);
        let tol = eval_tol_reached((&h1, &pc1), (&h2, &pc2), &c)?;
        let (e1, e2) = ends(&c)?;
        if e1.distance(&p) <= TOL3D || e2.distance(&p) <= TOL3D {
            return Ok(Some(SurfCut { c, pc1, pc2, tol }));
        }
    }
    Ok(None)
}

/// The analysis of the corner: the contour, the surface data, the face, the common
/// points on the arcs and the curves to write.
pub(super) struct MoreSurfdata {
    pub stripe: StripeHandle,
    pub seq: crate::chfi_ds::SurfDataSequenceHandle,
    pub ind: i32,
    pub ind_prev: i32,
    pub isfirst: bool,
    pub face: Face,
    pub neighbor: Face,
    pub cp1_on_arc: CommonPoint,
    pub cp2_on_arc: CommonPoint,
    pub cp2_not_on_arc: CommonPoint,
    pub is2nd_cp1_on_arc: bool,
    pub cracc: Arete,
    pub fi_line: FaceInterference,
    pub cint1: SurfCut,
    pub second: Option<(Point3, SurfCut)>,
    pub tr_cracc: Handle<dyn GeomCurve>,
}

/// The surface data `i` of the sequence.
pub(super) fn sd_at(
    seq: &crate::chfi_ds::SurfDataSequenceHandle,
    i: i32,
) -> Result<SurfDataHandle, Failure> {
    lock(seq)
        .get((i - 1) as usize)
        .cloned()
        .ok_or_else(|| Failure::out_of_range("PerformMoreSurfdata: surface data index"))
}

impl ChFi3dBuilder {
    /// `PerformMoreSurfdata(Index)`: the corner of one contour ending with two surface
    /// data on the face at the vertex; nothing when the configuration is not met.
    pub(crate) fn perform_more_surfdata(&mut self, index: i32) -> Result<(), Failure> {
        let Some(stripe) = self.my_vdata_map.find_from_index(index)?.first().cloned() else {
            return Ok(());
        };
        let vtx = self.my_vdata_map.find_key(index)?;
        let Some(ms) = self.more_surfdata_analysis(&stripe, &vtx)? else {
            return Ok(());
        };
        self.write_more_surfdata(ms)
    }

    /// The corner analysed: None when the native routine returns early.
    fn more_surfdata_analysis(
        &self,
        stripe: &StripeHandle,
        vtx: &Vertex,
    ) -> Result<Option<MoreSurfdata>, Failure> {
        let seq = lock(stripe)
            .set_of_surf_data()
            .cloned()
            .ok_or_else(|| Failure::new("PerformMoreSurfdata: no surface data"))?;
        let (ind, sens) = index_of_surf_data(vtx, stripe)?;
        let isfirst = sens == 1;
        let (cp1, cp2) = {
            let sd = sd_at(&seq, ind)?;
            let g = lock(&sd);
            (g.vertex(isfirst, 1).clone(), g.vertex(isfirst, 2).clone())
        };
        let mut face = Face::default();
        self.find_face(vtx, (&cp1, &cp2), &mut face)?;
        let surf_prev = BRepTool::surface(&face)?;
        let ind_prev = if sens == 1 { ind + 1 } else { ind - 1 };
        let (arc1, arc2) = self.edges_on_face(vtx, &face)?;
        let on_arcs = |cp: &CommonPoint| -> Result<bool, Failure> {
            Ok(cp.is_on_arc() && (cp.arc()?.is_same(&arc1) || cp.arc()?.is_same(&arc2)))
        };
        let (pcp1, pcp2) = {
            let sd = sd_at(&seq, ind_prev)?;
            let g = lock(&sd);
            (g.vertex(isfirst, 1).clone(), g.vertex(isfirst, 2).clone())
        };
        let (cp2_on_arc, cp2_not_on_arc, is2nd_cp1_on_arc) = if on_arcs(&pcp1)? {
            (pcp1, pcp2, true)
        } else if on_arcs(&pcp2)? {
            (pcp2, pcp1, false)
        } else {
            return Ok(None);
        };
        let (cp1_on_arc, is1st_cp1_on_arc) = if cp1.point().distance(&cp2_on_arc.point()) <= TOL3D {
            (cp2, false)
        } else {
            (cp1, true)
        };
        if !cp1_on_arc.is_on_arc() {
            return Ok(None);
        }
        let sd = sd_at(&seq, ind)?;
        let ind_surface = if is1st_cp1_on_arc {
            lock(&sd).index_of_s1()
        } else {
            lock(&sd).index_of_s2()
        };
        let neighbor = rustycad_tkbrep::topods::face(self.my_ds_shape(ind_surface))?;
        let cracc = self.end_arete(&sd, isfirst)?;
        let fi_line = lock(&sd)
            .interference(if is1st_cp1_on_arc { 1 } else { 2 })
            .clone();
        let surf = self.ds.surface(lock(&sd).surf()).surface().clone();
        let Some(cint1) = cut_ending_at(&surf_prev, &surf, cp1_on_arc.point())? else {
            return Ok(None);
        };
        let pext = Self::free_end(&cint1.c, (&cp1_on_arc, &cp2_on_arc))?;
        let Some((second, tr_cracc)) = self.second_section(
            &seq,
            ind_prev,
            (&surf_prev, &cracc),
            pext,
            (&cp2_on_arc, &cp2_not_on_arc),
        )?
        else {
            return Ok(None);
        };
        Ok(Some(MoreSurfdata {
            stripe: stripe.clone(),
            seq,
            ind,
            ind_prev,
            isfirst,
            face,
            neighbor,
            cp1_on_arc,
            cp2_on_arc,
            cp2_not_on_arc,
            is2nd_cp1_on_arc,
            cracc,
            fi_line,
            cint1,
            second,
            tr_cracc,
        }))
    }

    /// The shape `i` of the data structure (`myDS->Shape(i)`).
    fn my_ds_shape(&self, i: i32) -> &rustycad_tkbrep::topods::Shape {
        self.ds.shape(i, true)
    }

    /// The edge of the fillet at its end at the vertex (`ChFi3d_ComputeArete` with the
    /// tolerance of the routine).
    fn end_arete(&self, sd: &SurfDataHandle, isfirst: bool) -> Result<Arete, Failure> {
        let last = isfirst;
        let (p1, p2, pp1, pp2, isurf) = {
            let g = lock(sd);
            let at = |on_s: i32| -> Result<rustycad_tkmath::gp::Point2, Failure> {
                let fi = g.interference(on_s);
                let u = if last {
                    fi.last_parameter()
                } else {
                    fi.first_parameter()
                };
                fi.pcurve_on_surf().value(u)
            };
            (
                g.vertex(!last, 1).clone(),
                g.vertex(!last, 2).clone(),
                at(1)?,
                at(2)?,
                g.surf(),
            )
        };
        let surf = self.ds.surface(isurf).surface().clone();
        let mut arete = Arete::default();
        compute_arete(
            (&p1, pp1),
            (&p2, pp2),
            &surf,
            &mut arete,
            (TOL3D, self.tol2d),
            0,
        )?;
        Ok(arete)
    }

    /// The end of the intersection away from the two points on the arcs, if any.
    fn free_end(
        c: &Handle<dyn GeomCurve>,
        (cp1, cp2): (&CommonPoint, &CommonPoint),
    ) -> Result<Option<Point3>, Failure> {
        let (e1, e2) = ends(c)?;
        let away = |p: Point3| p.distance(&cp2.point()) > TOL3D && p.distance(&cp1.point()) > TOL3D;
        if away(e1) {
            return Ok(Some(e1));
        }
        if away(e2) {
            return Ok(Some(e2));
        }
        Ok(None)
    }

    /// The cut of the last fillet trimmed at the projection of the free end (and the
    /// intersection of the face with the previous fillet ending at the point on its arc),
    /// or whole without second section; None when the native routine returns early.
    #[allow(clippy::type_complexity)]
    fn second_section(
        &self,
        seq: &crate::chfi_ds::SurfDataSequenceHandle,
        ind_prev: i32,
        (surf_prev, cracc): (&Handle<dyn GeomSurface>, &Arete),
        pext: Option<Point3>,
        (cp2_on_arc, cp2_not_on_arc): (&CommonPoint, &CommonPoint),
    ) -> Result<Option<(Option<(Point3, SurfCut)>, Handle<dyn GeomCurve>)>, Failure> {
        let c = cracc
            .c3d
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "PerformMoreSurfdata"))?;
        let mut par = None;
        if let Some(pext) = pext {
            let cad = GeomAdaptorCurve::with_curve(&cracc.c3d)?;
            let ext = ExtPC::with_curve(pext, &cad as &dyn Adaptor3dCurve, TOL3D)?;
            if !ext.is_done() {
                return Ok(None);
            }
            for i in 1..=ext.nb_ext()? {
                if ext.is_min(i)? {
                    let pt = ext.point(i)?;
                    if pext.distance(pt.value()) <= TOL3D {
                        par = Some(pt.parameter());
                    }
                }
            }
        }
        let Some(apar) = par else {
            let tr: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
                &cracc.c3d,
                c.first_parameter(),
                c.last_parameter(),
                true,
                true,
            )?)
            .cast();
            return Ok(Some((None, tr)));
        };
        let (e1, _) = ends(&cracc.c3d)?;
        let apar1 = if e1.distance(&cp2_not_on_arc.point()) <= TOL3D {
            c.first_parameter()
        } else {
            c.last_parameter()
        };
        let (lo, hi) = if apar1 < apar {
            (apar1, apar)
        } else {
            (apar, apar1)
        };
        let tr: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&cracc.c3d, lo, hi, true, true)?).cast();
        let sd_prev = sd_at(seq, ind_prev)?;
        let surf = self.ds.surface(lock(&sd_prev).surf()).surface().clone();
        let Some(cint2) = cut_ending_at(surf_prev, &surf, cp2_on_arc.point())? else {
            return Ok(None);
        };
        let pext = pext.ok_or_else(|| Failure::new("PerformMoreSurfdata: no free end"))?;
        Ok(Some((Some((pext, cint2)), tr)))
    }
}
