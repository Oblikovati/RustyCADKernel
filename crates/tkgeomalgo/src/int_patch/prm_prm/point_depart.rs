// Rust translation of IntPatch_PrmPrmIntersection::PointDepart and the class
// IntPatch_InfoPD (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PointDepart`: the start points of the walkings from a grid of the common box of
//! the surfaces: the cells crossed by the sampled triangles of both surfaces.
use super::{BASE, IntPatchPrmPrmIntersection};
use crate::int_patch::IntPatchPrmPrmIntersectionT3Bits;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use crate::intf::native_int_cast;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;

/// `IntPatch_InfoPD`: the sampled points of both surfaces, their grid cells and the
/// codes of their positions against the box of the other surface, in flat arrays
/// indexed `i * NBI + j`.
struct InfoPd {
    nbi: i32,
    p1ds2: Vec<i32>,
    p2ds1: Vec<i32>,
    ip1: Vec<i32>,
    ip2: Vec<i32>,
    p1: Vec<Point3>,
    p2: Vec<Point3>,
}

impl InfoPd {
    fn new(nbi: i32) -> Self {
        let n: usize = (nbi * nbi) as usize;
        let origin = Point3::new(0.0, 0.0, 0.0);
        Self {
            nbi,
            p1ds2: vec![0; n],
            p2ds1: vec![0; n],
            ip1: vec![0; n],
            ip2: vec![0; n],
            p1: vec![origin; n],
            p2: vec![origin; n],
        }
    }

    /// `Index(i, j)`.
    fn index(&self, i: i32, j: i32) -> usize {
        (i * self.nbi + j) as usize
    }
}

/// The sampling of one surface: its bounds, its steps and counts, and the largest
/// diagonal step of its grid.
struct Sampling {
    u0: f64,
    v0: f64,
    du: f64,
    dv: f64,
    su: i32,
    sv: i32,
    dmax: f64,
}

/// Samples `s` on `su x sv` points into `points`, adding them to `bbox`.
fn sample(
    s: &Adaptor3dSurfaceHandle,
    (su, sv): (i32, i32),
    pd: &InfoPd,
    points: &mut [Point3],
    bbox: &mut BoundingBox3,
) -> Result<Sampling, Failure> {
    let (u0, u1) = (s.first_u_parameter()?, s.last_u_parameter()?);
    let (v0, v1) = (s.first_v_parameter()?, s.last_v_parameter()?);
    let du: f64 = (u1 - u0) / f64::from(su - 1);
    let dv: f64 = (v1 - v0) / f64::from(sv - 1);
    let mut dmax: f64 = 0.0;
    let mut u: f64 = u0;
    for i in 0..su {
        let mut v: f64 = v0;
        for j in 0..sv {
            let p: Point3 = s.value(u, v)?;
            points[pd.index(i, j)] = p;
            bbox.add_point(p);
            if i > 0 && j > 0 {
                let q: Point3 = points[pd.index(i - 1, j - 1)];
                let d: f64 = (q.x() - p.x()).abs() + (q.y() - p.y()).abs() + (q.z() - p.z()).abs();
                if d > dmax {
                    dmax = d;
                }
            }
            v += dv;
        }
        u += du;
    }
    bbox.enlarge(1.0e-8);
    Ok(Sampling {
        u0,
        v0,
        du,
        dv,
        su,
        sv,
        dmax,
    })
}

/// The grid of the common box: its origin and steps, and the enlarged boxes of both
/// surfaces (`x10..z11`, `x20..z21`).
struct GridFrame {
    origin: [f64; 3],
    step: [f64; 3],
    box1: ([f64; 3], [f64; 3]),
    box2: ([f64; 3], [f64; 3]),
}

fn limits_of(bbox: &BoundingBox3) -> ([f64; 3], [f64; 3]) {
    let l = bbox.limits();
    ([l.x_min, l.y_min, l.z_min], [l.x_max, l.y_max, l.z_max])
}

impl GridFrame {
    /// The frame of the intersection of both boxes, enlarged by twice the larger
    /// diagonal step.
    fn new(box1: &BoundingBox3, box2: &BoundingBox3, dmax_on: (f64, f64)) -> Self {
        let (mut min1, mut max1) = limits_of(box1);
        let (mut min2, mut max2) = limits_of(box2);
        let mut lo: [f64; 3] = [0, 1, 2].map(|k| if min1[k] > min2[k] { min1[k] } else { min2[k] });
        let mut hi: [f64; 3] = [0, 1, 2].map(|k| if max1[k] < max2[k] { max1[k] } else { max2[k] });
        let mut dmax_on1: f64 = dmax_on.0;
        if dmax_on.1 > dmax_on1 {
            dmax_on1 = dmax_on.1;
        }
        dmax_on1 += dmax_on1;
        for k in 0..3 {
            lo[k] -= dmax_on1;
            hi[k] += dmax_on1;
        }
        for k in 0..3 {
            min1[k] -= dmax_on1;
            max1[k] += dmax_on1;
        }
        for k in 0..3 {
            min2[k] -= dmax_on1;
            max2[k] += dmax_on1;
        }
        let step: [f64; 3] = grid_steps(lo, hi);
        Self {
            origin: lo,
            step,
            box1: (min1, max1),
            box2: (min2, max2),
        }
    }
}

/// The steps of the grid of 128 cells in each direction, none smaller than a hundredth
/// of the largest.
fn grid_steps(lo: [f64; 3], hi: [f64; 3]) -> [f64; 3] {
    let nb_pg: f64 = f64::from(BASE);
    let mut d: [f64; 3] = [0, 1, 2].map(|k| (hi[k] - lo[k]) / nb_pg);
    let mut dmax: f64 = d[0];
    if d[1] > dmax {
        dmax = d[1];
    }
    if d[2] > dmax {
        dmax = d[2];
    }
    for dk in &mut d {
        if *dk < dmax * 0.01 {
            *dk = dmax * 0.01;
        }
    }
    d
}

impl IntPatchPrmPrmIntersection {
    /// The grid cell of a point (-1 when out of the grid).
    fn cell_of(&self, frame: &GridFrame, p: Point3) -> i32 {
        let coords: [f64; 3] = [p.x(), p.y(), p.z()];
        let mut cell: [i32; 3] = [0; 3];
        for k in 0..3 {
            let half: f64 = frame.step[k] * 0.5;
            cell[k] = native_int_cast((coords[k] - frame.origin[k] + half) / frame.step[k]);
            if self.dans_grille(cell[k]) == 0 {
                return -1;
            }
        }
        self.grille_integer(cell[0], cell[1], cell[2])
    }

    /// The cells and the rejection codes (against the box of the other surface) of the
    /// sampled points of one surface.
    fn code_points(
        &self,
        frame: &GridFrame,
        (points, other_box): (&[Point3], ([f64; 3], [f64; 3])),
        (su, sv, nbi): (i32, i32, i32),
        (cells, codes): (&mut [i32], &mut [i32]),
    ) {
        for i in 0..su {
            for j in 0..sv {
                let at: usize = (i * nbi + j) as usize;
                let p: Point3 = points[at];
                codes[at] = self.code_reject(other_box.0, other_box.1, [p.x(), p.y(), p.z()]);
                cells[at] = self.cell_of(frame, p);
            }
        }
    }

    /// Fills `map` with the cells of the triangles of the sampled grid of one surface
    /// whose corners are not all on one side of the box of the other surface.
    fn fill_map(
        &self,
        (cells, codes): (&[i32], &[i32]),
        (su, sv, nbi): (i32, i32, i32),
        map: &mut IntPatchPrmPrmIntersectionT3Bits,
    ) {
        let at = |i: i32, j: i32| (i * nbi + j) as usize;
        for i in 0..su - 1 {
            for j in 0..sv - 1 {
                let (c00, c10, c11, c01) = (
                    codes[at(i, j)],
                    codes[at(i + 1, j)],
                    codes[at(i + 1, j + 1)],
                    codes[at(i, j + 1)],
                );
                if !((c00 & c10) != 0 || (c00 & c11) != 0) {
                    self.remplit(
                        cells[at(i, j)],
                        cells[at(i + 1, j)],
                        cells[at(i + 1, j + 1)],
                        map,
                    );
                }
                if !((c00 & c01) != 0 || (c00 & c11) != 0) {
                    self.remplit(
                        cells[at(i, j)],
                        cells[at(i, j + 1)],
                        cells[at(i + 1, j + 1)],
                        map,
                    );
                }
            }
        }
    }
}

/// The first sample point of a surface in the cell `newind`, marked as treated.
fn find_sample(
    cells: &mut [i32],
    (su, sv, nbi): (i32, i32, i32),
    newind: i32,
    treated: i32,
) -> Option<(i32, i32)> {
    for nu in 0..su {
        for nv in 0..sv {
            let at: usize = (nu * nbi + nv) as usize;
            if cells[at] == newind {
                cells[at] = treated;
                return Some((nu, nv));
            }
        }
    }
    None
}

/// The mean of the parameters of the 3 sample points of a surface nearest to `p` (the
/// running maximum kept first, as natively).
fn nearest_three(points: &[Point3], sampling: &Sampling, nbi: i32, p: Point3) -> (f64, f64) {
    let mut dist3: [f64; 3] = [f64::MAX; 3];
    let (mut u3, mut v3) = ([0.0; 3], [0.0; 3]);
    let mut u: f64 = sampling.u0;
    for i in 0..sampling.su {
        let mut v: f64 = sampling.v0;
        for j in 0..sampling.sv {
            let t: f64 = points[(i * nbi + j) as usize].square_distance(&p);
            // The largest value goes to Dist3[0].
            for (a, b) in [(0, 1), (1, 2), (0, 1)] {
                if dist3[a] < dist3[b] {
                    dist3.swap(a, b);
                    u3.swap(a, b);
                    v3.swap(a, b);
                }
            }
            if t < dist3[0] {
                (dist3[0], u3[0], v3[0]) = (t, u, v);
            }
            v += sampling.dv;
        }
        u += sampling.du;
    }
    ((u3[0] + u3[1] + u3[2]) / 3.0, (v3[0] + v3[1] + v3[2]) / 3.0)
}

/// The state of the extraction of `PointDepart`.
struct Extraction<'a> {
    pd: InfoPd,
    frame: GridFrame,
    samplings: (Sampling, Sampling),
    surfaces: (&'a Adaptor3dSurfaceHandle, &'a Adaptor3dSurfaceHandle),
}

impl IntPatchPrmPrmIntersection {
    /// Clears the neighbours of the cell `(i, j, k)` in `m1` when at least 3 of the
    /// cells around it (itself included) are in both maps.
    fn clear_neighbours(
        &self,
        [i, j, k]: [i32; 3],
        (m1, m2): (
            &mut IntPatchPrmPrmIntersectionT3Bits,
            &IntPatchPrmPrmIntersectionT3Bits,
        ),
    ) {
        let inside = |t: i32| self.dans_grille(t - 1) != 0 && self.dans_grille(t + 1) != 0;
        if !(inside(i) && inside(j) && inside(k)) {
            return;
        }
        const LIM: i32 = 3;
        let mut nb: i32 = 0;
        for si in -1..=1 {
            for sj in -1..=1 {
                for sk in -1..=1 {
                    let lu: i32 = self.grille_integer(i + si, j + sj, k + sk);
                    if nb < LIM && m1.val(lu) != 0 && m2.val(lu) != 0 {
                        nb += 1;
                    }
                }
            }
        }
        if nb < LIM {
            return;
        }
        for si in -1..=1 {
            for sj in -1..=1 {
                for sk in -1..=1 {
                    if si != 0 || sj != 0 || sk != 0 {
                        m1.raz(self.grille_integer(i + si, j + sj, k + sk));
                    }
                }
            }
        }
    }

    /// The start point of the cell `newind`: the sample points of both surfaces in it,
    /// else the means of the three nearest sample points of each surface.
    fn cell_point(
        &self,
        ex: &mut Extraction<'_>,
        newind: i32,
        treated: i32,
        p: Point3,
    ) -> Result<IntSurfPntOn2S, Failure> {
        let (s1, s2) = &ex.samplings;
        let nbi: i32 = ex.pd.nbi;
        let found1 = find_sample(&mut ex.pd.ip1, (s1.su, s1.sv, nbi), newind, treated);
        let found2 = match found1 {
            Some(_) => find_sample(&mut ex.pd.ip2, (s2.su, s2.sv, nbi), newind, treated),
            None => None,
        };
        let mut pon2s = IntSurfPntOn2S::new();
        if let (Some((nu1, nv1)), Some((nu2, nv2))) = (found1, found2) {
            let (surf1, surf2) = ex.surfaces;
            pon2s.set_value_all(
                p,
                surf1.first_u_parameter()? + f64::from(nu1) * s1.du,
                surf1.first_v_parameter()? + f64::from(nv1) * s1.dv,
                surf2.first_u_parameter()? + f64::from(nu2) * s2.du,
                surf2.first_v_parameter()? + f64::from(nv2) * s2.dv,
            );
            return Ok(pon2s);
        }
        // No point of the triangle is close enough: the 3 nearest points of each array.
        let (u1_3, v1_3) = nearest_three(&ex.pd.p1, s1, nbi, p);
        let (u2_3, v2_3) = nearest_three(&ex.pd.p2, s2, nbi, p);
        pon2s.set_value_all(p, u1_3, v1_3, u2_3, v2_3);
        Ok(pon2s)
    }

    /// `PointDepart(LineOn2S, S1, SU1, SV1, S2, SU2, SV2)`: appends to `line_on_2s` a
    /// start point for each grid cell (of 128 x 128 x 128 over the common box) crossed
    /// by the sampled grids of both surfaces (15 times the given counts, at most 200).
    ///
    /// ```ignore
    /// inter.point_depart(&line, &s1, (10, 10), &s2, (10, 10))?;
    /// ```
    pub fn point_depart(
        &self,
        line_on_2s: &Handle<IntSurfLineOn2S>,
        s1: &Adaptor3dSurfaceHandle,
        (su_1, sv_1): (i32, i32),
        s2: &Adaptor3dSurfaceHandle,
        (su_2, sv_2): (i32, i32),
    ) -> Result<(), Failure> {
        const NBI: i32 = 200;
        let mut pd = InfoPd::new(NBI);
        let cap = |n: i32| -> i32 { std::cmp::min(15_i32.wrapping_mul(n), NBI) };
        let (su1, sv1, su2, sv2) = (cap(su_1), cap(sv_1), cap(su_2), cap(sv_2));
        let (mut box1, mut box2) = (BoundingBox3::default(), BoundingBox3::default());
        let mut p1 = std::mem::take(&mut pd.p1);
        let sampling1: Sampling = sample(s1, (su1, sv1), &pd, &mut p1, &mut box1)?;
        let mut p2 = std::mem::take(&mut pd.p2);
        let sampling2: Sampling = sample(s2, (su2, sv2), &pd, &mut p2, &mut box2)?;
        (pd.p1, pd.p2) = (p1, p2);
        if box1.is_out_box(&box2) {
            return Ok(());
        }
        let frame = GridFrame::new(&box1, &box2, (sampling1.dmax, sampling2.dmax));
        let mut ex = Extraction {
            pd,
            frame,
            samplings: (sampling1, sampling2),
            surfaces: (s1, s2),
        };
        let (m1, m2) = self.fill_maps(&mut ex);
        self.extract(&mut ex, (m1, m2), line_on_2s)
    }

    /// The cells and codes of both samplings, and the maps of their triangles.
    fn fill_maps(
        &self,
        ex: &mut Extraction<'_>,
    ) -> (
        IntPatchPrmPrmIntersectionT3Bits,
        IntPatchPrmPrmIntersectionT3Bits,
    ) {
        let nbi: i32 = ex.pd.nbi;
        let (s1, s2) = (&ex.samplings.0, &ex.samplings.1);
        let (mut m1, mut m2) = (
            IntPatchPrmPrmIntersectionT3Bits::new(BASE),
            IntPatchPrmPrmIntersectionT3Bits::new(BASE),
        );
        let pd = &mut ex.pd;
        self.code_points(
            &ex.frame,
            (&pd.p1, ex.frame.box2),
            (s1.su, s1.sv, nbi),
            (&mut pd.ip1, &mut pd.p1ds2),
        );
        self.code_points(
            &ex.frame,
            (&pd.p2, ex.frame.box1),
            (s2.su, s2.sv, nbi),
            (&mut pd.ip2, &mut pd.p2ds1),
        );
        self.fill_map((&pd.ip1, &pd.p1ds2), (s1.su, s1.sv, nbi), &mut m1);
        self.fill_map((&pd.ip2, &pd.p2ds1), (s2.su, s2.sv, nbi), &mut m2);
        m1.reset_and();
        m2.reset_and();
        (m1, m2)
    }

    /// Takes the cells common to both maps in order and appends their start points.
    fn extract(
        &self,
        ex: &mut Extraction<'_>,
        (mut m1, mut m2): (
            IntPatchPrmPrmIntersectionT3Bits,
            IntPatchPrmPrmIntersectionT3Bits,
        ),
        line_on_2s: &Handle<IntSurfLineOn2S>,
    ) -> Result<(), Failure> {
        let mut newind: i32 = 0;
        let mut treated: i32 = 0;
        loop {
            treated -= 1;
            if m1.and(&mut m2, &mut newind) == 0 {
                return Ok(());
            }
            let (i, j, k) = self.integer_grille(newind);
            self.clear_neighbours([i, j, k], (&mut m1, &m2));
            let [x0, y0, z0] = ex.frame.origin;
            let [dx, dy, dz] = ex.frame.step;
            let p = Point3::new(
                dx * f64::from(i) + x0,
                dy * f64::from(j) + y0,
                dz * f64::from(k) + z0,
            );
            let pon2s: IntSurfPntOn2S = self.cell_point(ex, newind, treated, p)?;
            line_on_2s.add(&pon2s);
        }
    }
}
