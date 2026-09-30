// Rust translation of the enclosing, outside and unqualified cases of the
// two-circle constructor of GccAna_Circ2d2TanRad.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The qualifier pairs without an enclosed circle.
use super::Circ2d2TanRad;
use super::circ_circ::Work;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;

fn along(w: &Work, from: Point2, k: f64) -> Point2 {
    Point2::from(from.xy() + k * w.dir1.xy())
}

impl Circ2d2TanRad {
    pub(super) fn enclosing_enclosing(&mut self, w: &mut Work) -> Result<(), Failure> {
        let (r1, r2, dist, radius, tol) = (w.r1, w.r2, w.dist, w.radius, w.tol);
        if (tol < r1.max(r2) - radius)
            || (tol < r1.max(r2) - dist - r1.min(r2))
            || (dist + r1 + r2 - radius * 2.0 > tol)
        {
            self.well_done = true;
            return Ok(());
        }
        let inner: bool = (dist + r1.min(r2) - r1.max(r2)).abs() <= tol;
        if inner || ((r1 + r2 + dist - 2.0 * radius).abs() <= tol) {
            let signe: f64 = if inner { 1.0 } else { -1.0 };
            self.single(w, along(w, w.center1, signe * (r1 - radius)))?;
            self.tangency1.point.set(1, along(w, w.center1, r1))?;
            self.tangency2
                .point
                .set(1, along(w, w.center2, signe * r2))?;
            self.tangency1.same.set(1, 0)?;
            return self.tangency2.same.set(1, 0);
        }
        if (radius - r1.max(r2)).abs() <= tol {
            if r1 > r2 {
                (w.c[0], w.c[1]) = (w.q1.qualified(), w.c2);
                self.tangency1.same.set(1, 1)?;
                self.tangency2.same.set(1, 0)?;
                self.tangency2
                    .point
                    .set(1, along(w, w.center1, dist + r2))?;
            } else {
                (w.c[0], w.c[1]) = (w.c2, w.q1.qualified());
                self.tangency1.same.set(1, 0)?;
                self.tangency2.same.set(1, 1)?;
                self.tangency1.point.set(1, along(w, w.center1, -r1))?;
            }
            return self.single(w, w.c[0].location());
        }
        w.pairs(&[((radius - r1).abs(), (radius - r2).abs())])
    }

    pub(super) fn enclosing_outside(&mut self, w: &mut Work) -> Result<(), Failure> {
        if w.q1.is_outside() {
            w.swap_arguments();
        }
        let (r1, r2, dist, radius, tol) = (w.r1, w.r2, w.dist, w.radius, w.tol);
        if (r1 - radius > tol) || (tol < r1 + r2 - dist) || (dist - r2 + r1 - radius * 2.0 > tol) {
            self.well_done = true;
            return Ok(());
        }
        let same1: bool = ((r1 - radius).abs() <= tol) || ((r1 + r2 - dist).abs() <= tol);
        if same1 || ((dist - r2 + r1 - radius * 2.0).abs() <= tol) {
            self.tangency1.same.set(1, i32::from(same1))?;
            self.tangency2.same.set(1, 0)?;
            self.single(w, along(w, w.center1, r1 - radius))?;
            self.tangency1.point.set(1, along(w, w.center1, -r1))?;
            return self.tangency2.point.set(1, along(w, w.center1, dist - r2));
        }
        w.pairs(&[((radius - r1).abs(), radius + r2)])
    }

    pub(super) fn enclosing_unqualified(&mut self, w: &mut Work) -> Result<(), Failure> {
        if w.q1.is_unqualified() {
            w.swap_arguments();
        }
        let (r1, r2, dist, radius, tol) = (w.r1, w.r2, w.dist, w.radius, w.tol);
        if (tol < r1 - dist - r2) || (r1 - radius > tol) {
            self.well_done = true;
            return Ok(());
        }
        let same1: bool = (r1 - radius).abs() <= tol;
        if same1 || ((r1 - dist - r2).abs() > 0.0) {
            if same1 {
                self.tangency1.same.set(1, 1)?;
                // Native leaves TheSame2 unset otherwise (0 here, uninitialized natively).
                if ((radius - r2).abs() <= tol) && (w.center1.distance(w.center2) <= tol) {
                    self.tangency2.same.set(1, 1)?;
                }
            } else {
                self.tangency1.same.set(1, 0)?;
                self.tangency2.same.set(1, 0)?;
            }
            self.single(w, along(w, w.center1, radius - r1))?;
            self.tangency1.point.set(1, along(w, w.center1, r1))?;
            return self.tangency2.point.set(1, along(w, w.center1, dist + r2));
        }
        let (l1, l2) = (w.c[0].location(), w.c[1].location());
        w.ring(3, l1, (radius - r1).abs())?;
        w.ring(4, l2, radius + r2)?;
        w.ring(1, l1, (radius - r1).abs())?;
        w.ring(2, l2, (radius - r2).abs())?;
        w.nbsol = 2;
        Ok(())
    }

    pub(super) fn outside_outside(&mut self, w: &mut Work) -> Result<(), Failure> {
        let (r1, r2, dist, radius, tol) = (w.r1, w.r2, w.dist, w.radius, w.tol);
        if (tol < r1.max(r2) - dist - r1.min(r2)) || (dist - r1 - r2 - radius * 2.0 > tol) {
            self.well_done = true;
            return Ok(());
        }
        if (dist + r1.min(r2) - r1.max(r2)).abs() <= tol {
            let signe: f64 = if r1 < r2 { -1.0 } else { 1.0 };
            self.single(w, along(w, w.center1, signe * (radius + r1)))?;
            let p1: Point2 = along(w, w.center1, signe * r1);
            self.tangency1.point.set(1, p1)?;
            self.tangency2.point.set(1, p1)?;
        } else if (dist - r1 - r2 - radius * 2.0).abs() <= tol {
            self.single(w, along(w, w.center1, r1 + radius))?;
            self.tangency1.point.set(1, along(w, w.center1, r1))?;
            self.tangency2.point.set(1, along(w, w.center2, -r2))?;
        } else {
            return w.pairs(&[(radius + r1, radius + r2)]);
        }
        self.tangency1.same.set(1, 0)?;
        self.tangency2.same.set(1, 0)
    }

    pub(super) fn outside_unqualified(&mut self, w: &mut Work) -> Result<(), Failure> {
        if w.q1.is_unqualified() {
            w.swap_arguments();
        }
        let (r1, r2, dist, radius, tol) = (w.r1, w.r2, w.dist, w.radius, w.tol);
        if (tol < r1 - dist - r2)
            || ((tol < r2 - dist - r1) && (tol < radius * 2.0 - r2 - dist + r1))
            || ((dist - r1 - r2 > tol) && (tol < dist - r1 - r2 - radius * 2.0))
        {
            self.well_done = true;
            return Ok(());
        }
        if ((r1 - r2 - dist).abs() <= tol)
            || (((dist - r1 - r2).abs() <= tol) && ((radius * 2.0 - dist + r1 + r2).abs() <= tol))
            || (((dist + r1 - r2).abs() <= tol) && ((r2 + dist - r1 - radius * 2.0).abs() <= tol))
        {
            self.single(w, along(w, w.center1, radius + r1))?;
            self.tangency1.point.set(1, along(w, w.center1, r1))?;
            self.tangency2
                .point
                .set(1, along(w, w.center1, dist + r2))?;
            self.tangency1.same.set(1, 0)?;
            return self.tangency2.same.set(1, 0);
        }
        // Native centres the ring C(2) on C(1), not on C(2); kept.
        let (l1, l2) = (w.c[0].location(), w.c[1].location());
        w.ring(3, l1, radius + r1)?;
        w.ring(4, l2, radius + r2)?;
        w.ring(1, l1, radius + r1)?;
        w.ring(2, l1, (radius - r2).abs())?;
        w.nbsol = 2;
        Ok(())
    }

    pub(super) fn unqualified_unqualified(&mut self, w: &mut Work) -> Result<(), Failure> {
        let (r1, r2, radius, tol) = (w.r1, w.r2, w.radius, w.tol);
        let dist: f64 = w.dist;
        if (dist - r1 - r2 > tol) && (tol < (dist - r1 - r2 - radius * 2.0)) {
            self.well_done = true;
            return Ok(());
        }
        let nested: f64 = r1.max(r2) - dist - r1.min(r2);
        if (nested > tol) && (nested - radius * 2.0 > tol) {
            self.well_done = true;
            return Ok(());
        }
        let p3: f64 = r1.max(r2) - r1.min(r2) - dist - radius * 2.0;
        let p4: f64 = dist - r1 - r2;
        let p5: f64 = radius * 2.0 - dist + r1 + r2;
        if p3 > 0.0 {
            w.dist = r1.max(r2) - r1.min(r2) - radius * 2.0;
        } else if p4 > 0.0 && p5 < 0.0 {
            w.r1 = dist - r2 - radius * 2.0;
        }
        let (c1, c2, r1) = (w.center1, w.center2, w.r1);
        let radii: [(Point2, f64); 8] = [
            (c1, (radius - r1).abs()),
            (c2, (radius - r2).abs()),
            (c1, (radius - r1).abs()),
            (c2, radius + r2),
            (c1, radius + r1),
            (c2, (radius - r2).abs()),
            (c1, radius + r1),
            (c2, radius + r2),
        ];
        for (k, (centre, ring_radius)) in radii.into_iter().enumerate() {
            w.ring(k + 1, centre, ring_radius)?;
        }
        w.nbsol = 4;
        Ok(())
    }
}
