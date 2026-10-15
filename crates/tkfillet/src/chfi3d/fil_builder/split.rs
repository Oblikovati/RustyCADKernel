// Rust translation of ChFi3d_FilBuilder::SplitSurf (ChFi3d_FilBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The split of a fillet where its sections degenerate to a point.
use super::ChFi3dFilBuilder;
use crate::brep_blend::BRepBlendLineHandle;
use crate::chfi_ds::{SurfData, SurfDataHandle, lock, shared};
use crate::chfi3d::ChFi3dSearchSing;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{Function, FunctionRoot};

/// The parameter and the gap between the two sides of the section `i` of the line.
fn section_gap(lin: &BRepBlendLineHandle, i: i32) -> Result<(f64, f64), Failure> {
    let l = lock(lin);
    let p = l.point(i)?;
    Ok((p.parameter(), p.point_on_s1().distance(&p.point_on_s2())))
}

/// The parameter of the section `i` of the line.
fn section_param(lin: &BRepBlendLineHandle, i: i32) -> Result<f64, Failure> {
    Ok(lock(lin).point(i)?.parameter())
}

/// The isos of the two sides of the surface (`UIso(UFirst)`, `UIso(ULast)`).
type SideIsos = (Handle<dyn GeomCurve>, Handle<dyn GeomCurve>);

impl ChFi3dFilBuilder {
    /// `SplitSurf(SeqData, Line)`: the parameters where the iso of the fillet degenerates
    /// (a minimum of the gap between the sides of the sections where `V . V'` vanishes),
    /// and a copy of the surface data inserted before the reference for each of them.
    pub(super) fn split_surf(
        &mut self,
        seq: &mut Vec<SurfDataHandle>,
        lin: &BRepBlendLineHandle,
    ) -> Result<(), Failure> {
        let nbpnt = lock(lin).nb_points();
        if nbpnt < 3 {
            return Ok(());
        }
        let reference = seq
            .first()
            .cloned()
            .ok_or_else(|| Failure::out_of_range("SplitSurf: no surface data"))?;
        let surf = self
            .base
            .ds
            .surface(lock(&reference).surf())
            .surface()
            .clone();
        let s = surf
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "SplitSurf"))?;
        let (ufirst, ulast, _, _) = s.bounds();
        let isos: SideIsos = (s.u_iso(ufirst)?, s.u_iso(ulast)?);
        let (vfirst, vlast) = {
            let r = lock(&reference);
            let (i1, i2) = (r.interference_on_s1(), r.interference_on_s2());
            (
                i1.first_parameter().min(i2.first_parameter()),
                i1.last_parameter().max(i2.last_parameter()),
            )
        };
        let les_vi = self.singular_parameters(lin, nbpnt, &isos, (vfirst, vlast))?;
        for (ii, t) in les_vi.into_iter().enumerate() {
            self.insert_split(seq, ii, &reference, &isos, t)?;
        }
        Ok(())
    }

    /// The parameters `vi` where the iso `v = vi` is punctual, found by the minima of
    /// the gap along the line, then refined by the root of `ChFi3d_SearchSing`.
    fn singular_parameters(
        &self,
        lin: &BRepBlendLineHandle,
        nbpnt: i32,
        isos: &SideIsos,
        (vfirst, vlast): (f64, f64),
    ) -> Result<Vec<f64>, Failure> {
        let mut fonc = ChFi3dSearchSing::new(&isos.0, &isos.1);
        let mut les_vi: Vec<f64> = Vec::new();
        let mut ii = 1;
        while ii <= nbpnt && section_param(lin, ii)? < vfirst {
            ii += 1;
        }
        if ii == 1 {
            ii += 1;
        }
        let (mut b, mut courant) = section_gap(lin, ii)?;
        let (mut a, mut precedant) = section_gap(lin, ii - 1)?;
        ii += 1;
        while ii <= nbpnt && section_param(lin, ii)? <= vlast {
            while ii <= nbpnt
                && section_param(lin, ii)? < vlast
                && section_param(lin, ii)? - b < precision::p_confusion()
            {
                ii += 1;
            }
            let (c, suivant) = section_gap(lin, ii)?;
            if courant < precedant && courant < suivant {
                self.refine_minimum(&mut fonc, (a, c), (vfirst, vlast), &mut les_vi)?;
            }
            a = b;
            precedant = courant;
            b = c;
            courant = suivant;
            ii += 1;
        }
        Ok(les_vi)
    }

    /// The exact minimum between `a` and `c`, kept when `V . V'` is below the
    /// approximation tolerance there and it is away from the ends and the last one.
    fn refine_minimum(
        &self,
        fonc: &mut ChFi3dSearchSing,
        (a, c): (f64, f64),
        (vfirst, vlast): (f64, f64),
        les_vi: &mut Vec<f64>,
    ) -> Result<(), Failure> {
        let tol2d = self.base.tol2d;
        let resol = FunctionRoot::bounded(fonc, (a + c) / 2., tol2d, a, c, 50)?;
        if let Some(failure) = fonc.take_failure() {
            return Err(failure);
        }
        if !resol.is_done() {
            return Ok(());
        }
        let racine = resol.root()?;
        let mut val = 0.;
        fonc.value(racine, &mut val);
        if let Some(failure) = fonc.take_failure() {
            return Err(failure);
        }
        if val >= self.base.tolapp3d {
            return Ok(());
        }
        let lower = match les_vi.last() {
            None => vfirst,
            Some(last) => *last,
        };
        if racine > lower + tol2d && racine < vlast - tol2d {
            les_vi.push(racine);
        }
        Ok(())
    }

    /// The copy of the reference inserted at `ii` (0-based), ending at `t` where the
    /// reference now starts, with its own surface and curves in the data structure.
    fn insert_split(
        &mut self,
        seq: &mut Vec<SurfDataHandle>,
        ii: usize,
        reference: &SurfDataHandle,
        isos: &SideIsos,
        t: f64,
    ) -> Result<(), Failure> {
        let sd = shared(SurfData::new());
        lock(&sd).copy_from(&lock(reference));
        seq.insert(ii, sd.clone());
        let dstru = &mut self.base.ds;
        let s = dstru.surface(lock(reference).surf()).clone();
        let mut g = lock(&sd);
        g.change_surf(dstru.add_surface(&s));
        let mut c1 = dstru.curve(g.interference_on_s1().line_index()).clone();
        let i1 = dstru.add_curve(&mut c1);
        g.change_interference_on_s1().set_line_index(i1);
        let mut c2 = dstru.curve(g.interference_on_s2().line_index()).clone();
        let i2 = dstru.add_curve(&mut c2);
        g.change_interference_on_s2().set_line_index(i2);
        let p1: Point3 = isos.0.d0(t)?;
        let p2: Point3 = isos.1.d0(t)?;
        let p3d = Point3::from((p1.xyz() + p2.xyz()) / 2.);
        let vertex_tol = p1.distance(&p2) + c1.tolerance().max(c2.tolerance());
        let mut r = lock(reference);
        for on_s in [1, 2] {
            let cp = g.change_vertex(false, on_s);
            cp.reset();
            cp.set_point(p3d);
            cp.set_tolerance(vertex_tol);
            let cr = r.change_vertex(true, on_s);
            cr.reset();
            cr.set_point(p3d);
            cr.set_tolerance(vertex_tol);
            g.change_interference(on_s).set_last_parameter(t);
            r.change_interference(on_s).set_first_parameter(t);
        }
        g.set_last_spine_param(t);
        r.set_first_spine_param(t);
        Ok(())
    }
}
