// Rust translation of the two-circle constructor of GccAna_Lin2d2Tan.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Lin2d2Tan(Qualified1, Qualified2, Tolerance): the lines tangent to two
//! circles.
//!
//! Native turns the centre of C2 around the centre of C1 by the angle of sine
//! (+-R1 +- R2) / dist, takes the direction from C1 to the turned point and slides
//! the line to the tangency point. Several branches store the qualifiers of their
//! second solution in slot 1 (so slot 2 stays unset) or copy Qualified1 into
//! qualifier2, and one stores the first tangency point for the second solution;
//! all of this is kept.
use super::{Lin2d2Tan, MEMBER};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ent::{Position, QualifiedCirc, bad_qualifier};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Vector2, Xy};

// The two circles and their derived sizes.
#[derive(Clone, Copy)]
struct Pair {
    c1: Point2,
    c2: Point2,
    r1: f64,
    r2: f64,
    dist: f64,
    tol: f64,
}

// signe * gp_XY(r * D.Y(), -r * D.X()) (`left` false) or signe * gp_XY(-r * D.Y(),
// r * D.X()) (`left` true): a radius normal to D. Negating a factor or multiplying
// by +-1 is exact, so the native spellings of this offset all agree.
fn normal(r: f64, d: Direction2, signe: f64, left: bool) -> Xy {
    if left {
        return Xy::new(signe * (-r * d.y()), signe * (r * d.x()));
    }
    Xy::new(signe * (r * d.y()), signe * (-r * d.x()))
}

impl Pair {
    fn offset(&self, centre: Point2, xy: Xy) -> Point2 {
        Point2::from(centre.xy() + xy)
    }

    // C2 turned around C1 by asin(sine), and the direction from C1 to it.
    fn turned_direction(&self, sine: f64) -> Result<Direction2, Failure> {
        let p1: Point2 = self.c2.rotated(self.c1, sine.asin());
        Direction2::from_vector(Vector2::between(self.c1, p1))
    }

    // The direction of the centre line, from the larger circle to the smaller.
    fn centre_direction(&self) -> Result<Direction2, Failure> {
        if self.r1 < self.r2 {
            return Direction2::from_vector(Vector2::between(self.c2, self.c1));
        }
        Direction2::from_vector(Vector2::between(self.c1, self.c2))
    }

    // |dist + min(R1, R2) - max(R1, R2)|, the internal tangency defect.
    fn inner_defect(&self) -> f64 {
        (self.dist + self.r1.min(self.r2) - self.r1.max(self.r2)).abs()
    }
}

impl Lin2d2Tan {
    /// GccAna_Lin2d2Tan(Qualified1, Qualified2, Tolerance): the lines tangent to
    /// two circles, up to four; GccEnt_BadQualifier when a circle is enclosed;
    /// `Lin2d2Tan::tangent_to_circles(q1, q2, tol)?`.
    pub fn tangent_to_circles(
        qualified1: QualifiedCirc,
        qualified2: QualifiedCirc,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let tol: f64 = tolerance.abs();
        let mut lin = Self::empty(4);
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let (circle1, circle2): (Circle2, Circle2) =
            (qualified1.qualified(), qualified2.qualified());
        if qualified1.is_enclosed() || qualified2.is_enclosed() {
            return Err(bad_qualifier(&format!(
                "GccAna_Lin2d2Tan: qualifiers {:?}, {:?}; expected no GccEnt_enclosed circle",
                qualified1.qualifier(),
                qualified2.qualifier()
            )));
        }
        let pair = Pair {
            c1: circle1.location(),
            c2: circle2.location(),
            r1: circle1.radius(),
            r2: circle2.radius(),
            dist: circle1.location().distance(circle2.location()),
            tol,
        };
        if tol < pair.r1.max(pair.r2) - pair.dist - pair.r1.min(pair.r2) {
            lin.well_done = true;
        } else if !qualified1.is_unqualified() || !qualified2.is_unqualified() {
            lin.qualified_pair(&pair, qualified1, qualified2)?;
        } else {
            lin.unqualified_pair(&pair, qualified1.qualifier())?;
        }
        for i in 1..=lin.nbr_sol {
            lin.parameters(i, circle1, Some(circle2))?;
        }
        Ok(lin)
    }

    fn qualified_pair(
        &mut self,
        pair: &Pair,
        q1: QualifiedCirc,
        q2: QualifiedCirc,
    ) -> Result<(), Failure> {
        if q1.is_enclosing() && q2.is_enclosing() {
            return self.both_enclosing(pair, q1.qualifier(), q2.qualifier());
        }
        if (q1.is_enclosing() && q2.is_outside()) || (q2.is_enclosing() && q1.is_outside()) {
            let signe: f64 = if q1.is_enclosing() && q2.is_outside() {
                1.0
            } else {
                -1.0
            };
            return self.enclosing_outside(pair, q1.qualifier(), signe);
        }
        if q1.is_outside() && q2.is_outside() {
            return self.both_outside(pair, q1.qualifier());
        }
        let signe: f64 = if q2.is_unqualified() { 1.0 } else { -1.0 };
        if (q1.is_unqualified() && q2.is_enclosing()) || (q2.is_unqualified() && q1.is_enclosing())
        {
            return self.unqualified_enclosing(pair, q1.qualifier(), signe);
        }
        if (q1.is_unqualified() && q2.is_outside()) || (q2.is_unqualified() && q1.is_outside()) {
            return self.unqualified_outside(pair, q1.qualifier(), signe);
        }
        Ok(())
    }

    // Only one solution: the line through the tangency point of internally tangent
    // circles, of direction D turned left (`left`) or right.
    fn internal_tangent(
        &mut self,
        pair: &Pair,
        qualifiers: (Position, Position),
        left: bool,
    ) -> Result<(), Failure> {
        let d1: Direction2 = pair.centre_direction()?;
        let p1 = Point2::from(pair.c1.xy() + pair.r1 * d1.xy());
        let direction = if left {
            Direction2::new(-d1.y(), d1.x())?
        } else {
            Direction2::new(d1.y(), -d1.x())?
        };
        self.store(1, Line2::new(p1, direction), qualifiers, (p1, p1))?;
        self.well_done = true;
        self.nbr_sol = 1;
        Ok(())
    }

    // Store solution `index` built from C2 turned by asin(sine): tangency offsets
    // (r1 normal on C1, r2 normal on C2) with their own signs and sides.
    fn turned_solution(
        &mut self,
        pair: &Pair,
        index: i32,
        sine: f64,
        side1: (f64, bool),
        side2: (f64, bool),
        q: Position,
    ) -> Result<(Point2, Direction2), Failure> {
        let d1: Direction2 = pair.turned_direction(sine)?;
        let p1: Point2 = pair.offset(pair.c1, normal(pair.r1, d1, side1.0, side1.1));
        let p2: Point2 = pair.offset(pair.c2, normal(pair.r2, d1, side2.0, side2.1));
        self.store(index, Line2::new(p1, d1), (q, q), (p1, p2))?;
        Ok((p1, d1))
    }

    fn both_enclosing(&mut self, pair: &Pair, q1: Position, q2: Position) -> Result<(), Failure> {
        if pair.inner_defect() <= pair.tol && pair.dist >= pair.tol {
            return self.internal_tangent(pair, (q1, q2), true);
        }
        let sine: f64 = (pair.r1 - pair.r2) / pair.dist;
        self.turned_solution(pair, 1, sine, (1.0, false), (1.0, false), q1)?;
        self.well_done = true;
        self.nbr_sol = 1;
        Ok(())
    }

    fn enclosing_outside(&mut self, pair: &Pair, q1: Position, signe: f64) -> Result<(), Failure> {
        let (r1, r2, dist, tol) = (pair.r1, pair.r2, pair.dist, pair.tol);
        if r1 + r2 - dist > tol {
            self.well_done = true;
            return Ok(());
        }
        if (dist - r1 - r2).abs() < tol && dist > tol {
            // External tangency: native does not order the centres here.
            let d1 = Direction2::from_vector(Vector2::between(pair.c1, pair.c2))?;
            let p1 = Point2::from(pair.c1.xy() + r1 * d1.xy());
            self.store(
                1,
                Line2::new(p1, Direction2::new(-d1.y(), d1.x())?),
                (q1, q1),
                (p1, p1),
            )?;
        } else {
            let sine: f64 = signe * (r1 + r2) / dist;
            self.turned_solution(pair, 1, sine, (signe, false), (signe, true), q1)?;
        }
        self.well_done = true;
        self.nbr_sol = 1;
        Ok(())
    }

    fn both_outside(&mut self, pair: &Pair, q1: Position) -> Result<(), Failure> {
        if pair.inner_defect() < pair.tol && pair.dist > pair.tol {
            return self.internal_tangent(pair, (q1, q1), false);
        }
        let sine: f64 = (pair.r2 - pair.r1) / pair.dist;
        self.turned_solution(pair, 1, sine, (1.0, true), (1.0, true), q1)?;
        self.well_done = true;
        self.nbr_sol = 1;
        Ok(())
    }

    fn unqualified_enclosing(
        &mut self,
        pair: &Pair,
        q1: Position,
        signe: f64,
    ) -> Result<(), Failure> {
        let (r1, r2, dist) = (pair.r1, pair.r2, pair.dist);
        if pair.inner_defect() < pair.tol && dist > pair.tol {
            return self.internal_tangent(pair, (q1, q1), true);
        }
        let (p1, _) =
            self.turned_solution(pair, 1, (r1 - r2) / dist, (1.0, false), (signe, false), q1)?;
        self.well_done = true;
        self.nbr_sol = 1;
        if r1.min(r2) + r1.max(r2) < dist {
            let sine: f64 = signe * (r1 + r2) / dist;
            self.turned_solution(pair, 2, sine, (signe, false), (1.0, true), q1)?;
            // Native stores the qualifiers in slot 1 again and P1, not P2, as the
            // first tangency point of this solution.
            self.qualifier1.set(2, Position::default())?;
            self.qualifier2.set(2, Position::default())?;
            self.tangency1.point.set(2, p1)?;
            self.nbr_sol = 2;
        }
        Ok(())
    }

    fn unqualified_outside(
        &mut self,
        pair: &Pair,
        q1: Position,
        signe: f64,
    ) -> Result<(), Failure> {
        let (r1, r2, dist) = (pair.r1, pair.r2, pair.dist);
        if pair.inner_defect() <= pair.tol && dist >= pair.tol {
            return self.internal_tangent(pair, (q1, q1), false);
        }
        let sine: f64 = signe * (r2 - r1) / dist;
        self.turned_solution(pair, 1, sine, (1.0, true), (signe, true), q1)?;
        self.well_done = true;
        self.nbr_sol = 1;
        if r1.min(r2) + r1.max(r2) < dist {
            let sine2: f64 = signe * (-r2 - r1) / dist;
            self.turned_solution(pair, 2, sine2, (signe, true), (signe, false), q1)?;
            // Native stores the qualifiers of this solution in slot 1 again.
            self.qualifier1.set(2, Position::default())?;
            self.qualifier2.set(2, Position::default())?;
            self.nbr_sol = 2;
        }
        Ok(())
    }

    // Both circles unqualified: for each sign, the tangent of sine
    // signe (R2 - R1) / dist, then the one of sine signe (R2 + R1) / dist when the
    // circles are apart.
    fn unqualified_pair(&mut self, pair: &Pair, q1: Position) -> Result<(), Failure> {
        let (r1, r2, dist) = (pair.r1, pair.r2, pair.dist);
        let mut signe: f64 = 1.0;
        for _ in 1..=2 {
            signe = -signe;
            self.nbr_sol += 1;
            let sine: f64 = signe * (r2 - r1) / dist;
            self.turned_solution(pair, self.nbr_sol, sine, (signe, true), (signe, true), q1)?;
            self.well_done = true;
            if r1.min(r2) + r1.max(r2) < dist {
                let sine2: f64 = signe * (r2 + r1) / dist;
                self.nbr_sol += 1;
                self.turned_solution(pair, self.nbr_sol, sine2, (signe, false), (signe, true), q1)?;
            }
        }
        Ok(())
    }
}
