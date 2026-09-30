// Rust translation of the two Init members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ConstrainedFilling.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_ConstrainedFilling::Init` for three and four boundaries.
use super::corners::{coons_cnd, kill_corners, sort_bounds};
use super::{GeomFillConstrainedFilling, live};
use crate::geom_fill::{
    GeomFillBoundary, GeomFillCoonsAlgPatch, GeomFillDegeneratedBound, GeomFillTgtField,
    GeomFillTgtOnCoons,
};
use crate::law::{self, LawFunction, LawLinear};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;

type Bound = Handle<dyn GeomFillBoundary>;

/// A tangent field on side `i` of the patch.
fn tangent_field(patch: &Handle<GeomFillCoonsAlgPatch>, i: i32) -> Handle<dyn GeomFillTgtField> {
    Handle::new(GeomFillTgtOnCoons::new(patch, i)).cast()
}

/// Every boundary reparametrized on [0, 1], reversed as sorted.
fn reparametrize_all(bound: &[Bound], rev: &[bool]) -> Result<(), Failure> {
    for (b, &r) in bound.iter().zip(rev) {
        live(b)?.reparametrize(0., 1., false, false, 1., 1., r)?;
    }
    Ok(())
}

impl GeomFillConstrainedFilling {
    /// `Init(B1, B2, B3, NoCheck)`: the boundaries are sorted into a chain (reversed
    /// and reparametrized on [0, 1]), the fourth side is the point halfway between
    /// the free ends of the second and third, the corners are checked (unless
    /// `no_check`) and the surface is built. The tangent fields and corner states of
    /// an earlier `Init` stay where this one sets none, as natively;
    /// `f.init_three([&b1, &b2, &b3], false)?`.
    pub fn init_three(&mut self, bounds: [&Bound; 3], no_check: bool) -> Result<(), Failure> {
        let mut rev = [false; 3];
        let mut bound: [Bound; 3] = bounds.map(Handle::clone);
        sort_bounds(&mut bound, &mut rev, &mut self.stcor)?;
        rev[2] = !rev[2];
        reparametrize_all(&bound, &rev)?;
        let db: Bound = self.apex(&bound)?;
        self.ptch = Handle::new(GeomFillCoonsAlgPatch::new([
            &bound[0], &bound[1], &db, &bound[2],
        ])?);
        let mut ttgalg: [Handle<dyn GeomFillTgtField>; 3] = Default::default();
        for (slot, side) in [(0, 0), (1, 1), (2, 3)] {
            if live(&bound[slot])?.has_normals() {
                ttgalg[slot] = tangent_field(&self.ptch, side);
                self.tgalg[side as usize] = ttgalg[slot].clone();
            }
        }
        self.min_tangents()?;
        if !no_check {
            let nrev = [false, false, true];
            self.mig[2] = self.mig[3];
            coons_cnd(&bound, &nrev, &mut self.stcor, &self.mig)?;
            kill_corners(&bound, (&rev, &nrev), &self.stcor, &ttgalg)?;
        }
        self.stcor[3] = self.stcor[2];
        self.mix_if_twisted()?;
        self.build()
    }
    /// The degenerated fourth side of `Init(B1, B2, B3)`: the mean of the ends of the
    /// second and third boundaries at 1, with their larger 3d tolerance.
    fn apex(&self, bound: &[Bound]) -> Result<Bound, Failure> {
        let (b1, b2) = (live(&bound[1])?, live(&bound[2])?);
        let (p1, p2) = (b1.value(1.)?, b2.value(1.)?);
        let ppp = Point3::from((p1.xyz() + p2.xyz()) * 0.5);
        let t3: f64 = b1.tol3d().max(b2.tol3d());
        Ok(Handle::new(GeomFillDegeneratedBound::new(ppp, 0., 1., t3, 10.)).cast())
    }
    /// `Init(B1, B2, B3, B4, NoCheck)`: the boundaries are sorted into a loop
    /// (reversed and reparametrized on [0, 1]), the corners are checked (unless
    /// `no_check`) and the surface is built. The tangent fields and corner states of
    /// an earlier `Init` stay where this one sets none, as natively;
    /// `f.init_four([&b1, &b2, &b3, &b4], false)?`.
    pub fn init_four(&mut self, bounds: [&Bound; 4], no_check: bool) -> Result<(), Failure> {
        let mut rev = [false; 4];
        let mut bound: [Bound; 4] = bounds.map(Handle::clone);
        sort_bounds(&mut bound, &mut rev, &mut self.stcor)?;
        rev[2] = !rev[2];
        rev[3] = !rev[3];
        reparametrize_all(&bound, &rev)?;
        self.ptch = Handle::new(GeomFillCoonsAlgPatch::new([
            &bound[0], &bound[1], &bound[2], &bound[3],
        ])?);
        for (i, b) in bound.iter().enumerate() {
            if live(b)?.has_normals() {
                self.tgalg[i] = tangent_field(&self.ptch, i as i32);
            }
        }
        self.min_tangents()?;
        if !no_check {
            let nrev = [false, false, true, true];
            coons_cnd(&bound, &nrev, &mut self.stcor, &self.mig)?;
            kill_corners(&bound, (&rev, &nrev), &self.stcor, &self.tgalg)?;
        }
        self.mix_if_twisted()?;
        self.build()
    }
    /// `mig[i] = 1`, then the minimal magnitude of each tangent field.
    fn min_tangents(&mut self) -> Result<(), Failure> {
        for i in 0..4 {
            self.mig[i] = 1.;
            if self.tgalg[i].is_some() {
                self.min_tgte(i as i32)?;
            }
        }
        Ok(())
    }
    /// The check that the tangent fields keep their side: at the first field that
    /// turns over, both blending laws become `Law::MixBnd` of the linear ones.
    fn mix_if_twisted(&mut self) -> Result<(), Failure> {
        for i in 0..4 {
            self.mig[i] = 1.;
            if self.tgalg[i].is_null() || self.check_tgte(i as i32)? {
                continue;
            }
            let patch = self.patch()?;
            let (fu1, fu2) = patch.func_pair();
            let ffu1: Handle<dyn LawFunction> = law::mix_bnd_law(&fu1.cast::<LawLinear>())?.cast();
            let ffu2: Handle<dyn LawFunction> = law::mix_bnd_law(&fu2.cast::<LawLinear>())?.cast();
            patch.set_func(&ffu1, &ffu2);
            break;
        }
        Ok(())
    }
}
