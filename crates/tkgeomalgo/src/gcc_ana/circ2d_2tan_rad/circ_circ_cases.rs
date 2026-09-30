// Rust translation of the enclosed-circle cases of the two-circle constructor of
// GccAna_Circ2d2TanRad.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The qualifier pairs with an enclosed circle.
use super::Circ2d2TanRad;
use super::circ_circ::Work;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;

// center + k * dir1 (gp_XY + double * gp_XY).
fn along(w: &Work, from: Point2, k: f64) -> Point2 {
    Point2::from(from.xy() + k * w.dir1.xy())
}

impl Circ2d2TanRad {
    pub(super) fn enclosed_enclosed(&mut self, w: &mut Work) -> Result<(), Failure> {
        let (r1, r2, dist, radius, tol) = (w.r1, w.r2, w.dist, w.radius, w.tol);
        if radius * 2.0 - (r1 + r2 - dist).abs() > tol {
            self.well_done = true;
            return Ok(());
        }
        if (dist - r1 - r2 > tol) || (tol < r1.max(r2) - dist - r1.min(r2)) {
            self.well_done = true;
            return Ok(());
        }
        let secant: bool = (dist - r1 - r2).abs() <= tol;
        let inner: bool = (r1.max(r2) - dist - r1.min(r2)).abs() <= tol;
        if !(secant || inner) {
            let (l1, l2) = (w.c[0].location(), w.c[1].location());
            w.ring(1, l1, (radius - r1).abs())?;
            w.ring(2, l2, (radius - r2).abs())?;
            w.nbsol = 1;
            return Ok(());
        }
        let (rbid, signe): (f64, f64) = if secant {
            (r2 + (dist - r1 - r2) / 2.0, -1.0)
        } else {
            let r3: f64 = if r1 > r2 {
                r2
            } else {
                // Native replaces the variable C2 by C1 here.
                w.c2 = w.q1.qualified();
                r1
            };
            (-r3 + (r1.min(r2) + dist - r1.max(r2)) / 2.0, 1.0)
        };
        self.single(w, along(w, w.center2, -rbid))?;
        self.tangency1
            .point
            .set(1, along(w, w.center2, signe * r2))?;
        self.tangency2.point.set(1, along(w, w.center1, r1))?;
        self.tangency1.same.set(1, 0)?;
        self.tangency2.same.set(1, 0)
    }

    pub(super) fn enclosed_enclosing(&mut self, w: &mut Work) -> Result<(), Failure> {
        if w.q1.is_enclosing() {
            w.swap_arguments();
        }
        let (r1, r2, dist, radius, tol) = (w.r1, w.r2, w.dist, w.radius, w.tol);
        if (r2 - radius > tol)
            || (tol < radius - r1)
            || (tol > r1 - dist - r2)
            || ((tol < radius * 2.0 - r1 - dist - r2) || (tol < r1 + r2 - dist - radius * 2.0))
        {
            self.well_done = true;
            return Ok(());
        }
        if ((r2 - radius).abs() <= tol) || ((r1 - radius).abs() <= tol) {
            return self.enclosed_enclosing_same(w);
        }
        let touching: bool = (r2 + dist - r1).abs() <= tol;
        if touching || ((radius * 2.0 - r1 - dist - r2).abs() < tol) {
            let signe: f64 = if touching { 1.0 } else { -1.0 };
            self.single(w, along(w, w.center1, signe * (r1 - radius)))?;
            self.tangency1
                .point
                .set(1, along(w, w.center1, signe * r1))?;
            self.tangency2.point.set(1, along(w, w.center2, r2))?;
            self.tangency1.same.set(1, 0)?;
            return self.tangency2.same.set(1, 0);
        }
        let (l1, l2) = (w.c[0].location(), w.c[1].location());
        w.ring(1, l1, (radius - r1).abs())?;
        w.ring(2, l2, (radius - r2).abs())?;
        w.nbsol = 1;
        Ok(())
    }

    // Radius equal to one of the radii: the solution is centred on that argument.
    fn enclosed_enclosing_same(&mut self, w: &mut Work) -> Result<(), Failure> {
        let (r1, r2, radius, tol) = (w.r1, w.r2, w.radius, w.tol);
        if (r2 - radius).abs() <= tol {
            w.c[1] = w.c2;
            self.tangency2.same.set(1, 1)?;
            self.tangency1.same.set(1, 0)?;
            self.tangency1
                .point
                .set(1, along(w, w.c[1].location(), r2))?;
        } else if (radius - r1).abs() <= tol {
            w.c[1] = w.q1.qualified();
            self.tangency1.same.set(1, 1)?;
            self.tangency2.same.set(1, 0)?;
            self.tangency2
                .point
                .set(1, along(w, w.c[1].location(), r1))?;
        }
        self.single(w, w.c[1].location())
    }

    pub(super) fn enclosed_outside(&mut self, w: &mut Work) -> Result<(), Failure> {
        if w.q1.is_outside() {
            w.swap_arguments();
        }
        let (r1, r2, dist, radius, tol) = (w.r1, w.r2, w.dist, w.radius, w.tol);
        if (radius - r1 > tol)
            || (dist - r2 - r1 > tol)
            || ((r2 - dist - r1 + radius * 2.0 > tol) || (r1 - r2 - dist - radius * 2.0 > tol))
        {
            self.well_done = true;
            return Ok(());
        }
        let bigger_touching: bool = (radius - r1 > 0.0) && ((dist - r1 - r2).abs() <= tol);
        let near_side: bool = (r1 - r2 + dist - radius * 2.0).abs() <= tol;
        let far_side: bool = (r1 - r2 - dist - radius * 2.0).abs() <= tol;
        let touching: bool = (dist - r1 - r2).abs() <= tol;
        if !(bigger_touching || near_side || far_side || touching) {
            let (l1, l2) = (w.c[0].location(), w.c[1].location());
            w.ring(1, l1, (radius - r1).abs())?;
            w.ring(2, l2, radius + r2)?;
            w.nbsol = 1;
            return Ok(());
        }
        let signe: f64 = if near_side { -1.0 } else { 1.0 };
        if !near_side && (bigger_touching || touching) {
            // R2 takes R1; the TheSame1 flag native sets here is reset below.
            w.r2 = w.r1;
        }
        let r2: f64 = w.r2;
        self.single(w, along(w, w.center1, signe * (r1 - radius)))?;
        self.tangency1
            .point
            .set(1, along(w, w.center1, signe * r1))?;
        self.tangency2
            .point
            .set(1, along(w, w.center2, signe * r2))?;
        self.tangency1.same.set(1, 0)?;
        self.tangency2.same.set(1, 0)
    }

    pub(super) fn enclosed_unqualified(&mut self, w: &mut Work) -> Result<(), Failure> {
        if w.q1.is_unqualified() {
            w.swap_arguments();
        }
        let (r1, r2, dist, radius, tol) = (w.r1, w.r2, w.dist, w.radius, w.tol);
        if (radius - r1 > tol) || (dist - r2 - r1 > tol) {
            self.well_done = true;
            return Ok(());
        }
        if ((dist - r2 - r1).abs() <= tol) || ((radius - r1).abs() <= tol) {
            self.single(w, along(w, w.center1, r1 - radius))?;
            self.tangency2
                .point
                .set(1, along(w, w.center1, dist - r2))?;
            self.tangency2.same.set(1, 0)?;
            if (radius - r1).abs() > 0.0 {
                return self.tangency1.same.set(1, 1);
            }
            self.tangency1.same.set(1, 0)?;
            return self.tangency1.point.set(1, along(w, w.center1, r1));
        }
        let (l1, l2) = (w.c[0].location(), w.c[1].location());
        w.ring(3, l1, (radius - r1).abs())?;
        w.ring(4, l2, radius + r2)?;
        w.ring(1, l1, (radius - r1).abs())?;
        w.ring(2, l2, (radius - r2).abs())?;
        w.nbsol = 2;
        Ok(())
    }
}
