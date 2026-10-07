//! HLRBRep_Intersector.cxx: Perform on two edges, with the ends moved away from a
//! shared vertex until the intersection domains no longer touch.
use super::{Intersector, edge_bounds};
use crate::hlr_brep::{Curve, EdgeData};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkmath::gp::{Point2, Vector2};

// The shift factors of the four ends (`aDecalagea1` ...), doubled while the ends
// stay too close, -1 once an end falls back to the fraction of its range.
struct Shifts {
    a1: f64,
    a2: f64,
    b1: f64,
    b2: f64,
}

// One edge's domain: its 2D end points and parameters, and the end tolerances.
struct EdgeDomain {
    pa: Point2,
    pb: Point2,
    domain: Domain,
}

// The end `t` moved inwards: by `pdist * shift / |C'(t)|` when that is less than
// 40% of the range `d`, else by the fraction `frac` of the range (and the shift
// becomes -1); `sign` is +1 at the start, -1 at the end.
fn moved_end(
    c: &Curve,
    t: f64,
    (d, frac, sign): (f64, f64, f64),
    pdist: f64,
    shift: &mut f64,
) -> Result<f64, Failure> {
    let (mut p, mut v) = (Point2::default(), Vector2::default());
    c.d1(t, &mut p, &mut v)?;
    let qwe = v.magnitude();
    if qwe > 1e-12 {
        let dd = pdist * *shift / qwe;
        if dd < d * 0.4 {
            return Ok(t + sign * dd);
        }
    }
    *shift = -1.;
    Ok(t + sign * (d * frac))
}

// The domain of one edge with its ends moved by `da`, `db` and the tolerances
// capped by `tol` (-1 at the ends of an edge that ends at the other one).
fn edge_domain(
    e: &EdgeData,
    (da, db): (f64, f64),
    (shift_a, shift_b): (&mut f64, &mut f64),
    (tol, pdist, en_bout): (f64, f64, bool),
) -> Result<EdgeDomain, Failure> {
    let c = e.curve();
    let (mut a, mut ta, mut b, mut tb) = edge_bounds(e); //--   -> Parametres 3d
    let d = b - a;
    if da != 0. {
        a = moved_end(c, a, (d, da, 1.), pdist, shift_a)?;
    }
    if db != 0. {
        b = moved_end(c, b, (d, db, -1.), pdist, shift_b)?;
    }
    let (mut pa, mut pb) = (Point2::default(), Point2::default());
    c.d0(a, &mut pa)?;
    c.d0(b, &mut pb)?;
    let a = c.parameter_2d(a)?;
    let b = c.parameter_2d(b)?;
    if en_bout {
        ta = -1.;
        tb = -1.;
    }
    if f64::from(ta) > tol {
        ta = tol as f32;
    }
    if f64::from(tb) > tol {
        tb = tol as f32;
    }
    let domain = Domain::bounded(pa, a, f64::from(ta), pb, b, f64::from(tb));
    Ok(EdgeDomain { pa, pb, domain })
}

// The closest pair of moved ends among those moved (`cote` 1 to 4), or RealLast.
fn closest_ends(
    d1: &EdgeDomain,
    d2: &EdgeDomain,
    (da1, db1, da2, db2): (f64, f64, f64, f64),
) -> (f64, i32) {
    let dist = |flag: bool, p: Point2, q: Point2| if flag { p.distance(q) } else { real_last() };
    let candidates = [
        dist(da1 != 0. || da2 != 0., d1.pa, d2.pa),
        dist(da1 != 0. || db2 != 0., d1.pa, d2.pb),
        dist(db1 != 0. || da2 != 0., d1.pb, d2.pa),
        dist(db1 != 0. || db2 != 0., d1.pb, d2.pb),
    ];
    let (mut mindist, mut cote) = (candidates[0], 1); //-- cas 1
    for (k, c) in candidates.iter().enumerate().skip(1) {
        if mindist > *c {
            mindist = *c;
            cote = k as i32 + 1;
        }
    }
    (mindist, cote)
}

impl Shifts {
    // Doubles the shifts of the closest ends; false (stop) once one fell back.
    fn widen(&mut self, cote: i32) -> bool {
        match cote {
            1 => (self.a1 *= 2., self.a2 *= 2.),
            2 => (self.a1 *= 2., self.b2 *= 2.),
            3 => (self.b1 *= 2., self.a2 *= 2.),
            _ => (self.b1 *= 2., self.b2 *= 2.),
        };
        !(self.a1 < 0.0 || self.a2 < 0.0 || self.b1 < 0.0 || self.b2 <= 0.0)
    }
}

impl Intersector {
    /// Perform(NA, Edge1, Da1, Db1, NB, Edge2, Da2, Db2, NoBound): the intersection of
    /// two edges in the view plane. The moved ends start 100 tolerances inside;
    /// when the edges end at each other (`en_bout`) and the closest moved ends are
    /// within 1000 tolerances, those shifts double and the domains are rebuilt;
    /// `inter.perform_edges(1, &e1, 0.0, 0.0, 2, &e2, 0.0, 0.0, false)?`.
    #[allow(clippy::too_many_arguments)] // The native signature.
    pub fn perform_edges(
        &mut self,
        _na: i32,
        edge1: &EdgeData,
        da1: f64,
        db1: f64,
        _nb: i32,
        edge2: &EdgeData,
        da2: f64,
        db2: f64,
        en_bout: bool,
    ) -> Result<(), Failure> {
        self.set_type_perform(1);
        // modified by jgv, 18.04.2016 for OCC27341
        let (tol1, tol2) = (precision::confusion(), precision::confusion());
        let tol = if tol1 > tol2 { tol1 } else { tol2 };
        let pdist = if tol < 0.0000001 { 0.0000001 } else { tol };
        let mut s = Shifts {
            a1: 100.0,
            a2: 100.0,
            b1: 100.0,
            b2: 100.0,
        };
        loop {
            let d1 = edge_domain(
                edge1,
                (da1, db1),
                (&mut s.a1, &mut s.b1),
                (tol, pdist, en_bout),
            )?;
            let d2 = edge_domain(
                edge2,
                (da2, db2),
                (&mut s.a2, &mut s.b2),
                (tol, pdist, en_bout),
            )?;
            let mut pas_bon = false;
            if en_bout {
                let (mindist, cote) = closest_ends(&d1, &d2, (da1, db1, da2, db2));
                if mindist < tol * 1000. {
                    pas_bon = s.widen(cote);
                }
            }
            if !pas_bon {
                let (c1, c2) = (edge1.curve(), edge2.curve());
                return self
                    .curve_intersector()
                    .perform(c1, &d1.domain, c2, &d2.domain, tol, tol);
            }
        }
    }
}
