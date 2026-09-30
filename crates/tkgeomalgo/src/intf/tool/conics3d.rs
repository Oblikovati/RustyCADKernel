// Rust translation of Intf_Tool::HyprBox, ParabBox and the Inters3d overloads
// (ModelingAlgorithms/TKGeomAlgo/Intf/Intf_Tool.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The boxes of a 3d hyperbola or parabola in a domain: the crossings with the six
//! domain faces (IntAna_IntConicQuad), sorted and deduplicated, then the segment loop.
use super::Tool;
use crate::intf::{std_max, std_min};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::is_equal;
use rustycad_tkgeombase::int_ana::IntConicQuad;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Hyperbola3, Parabola3, Plane3, Point3, Vector3, Xyz};

/// The operations the Inters3d/…Box overloads call on their conic.
pub(super) trait BoxConic3d: Copy {
    fn value(&self, u: f64) -> Point3;
    fn d1(&self, u: f64) -> (Point3, Vector3);
    /// IntAna_IntConicQuad(conic, plane, Precision::Angular()).
    fn intersect_plane(&self, plane: Plane3) -> Result<IntConicQuad, Failure>;
}

impl BoxConic3d for Hyperbola3 {
    fn value(&self, u: f64) -> Point3 {
        elclib::value(u, self)
    }
    fn d1(&self, u: f64) -> (Point3, Vector3) {
        elclib::d1(u, self)
    }
    fn intersect_plane(&self, plane: Plane3) -> Result<IntConicQuad, Failure> {
        IntConicQuad::hyperbola_plane(*self, plane, precision::angular())
    }
}

impl BoxConic3d for Parabola3 {
    fn value(&self, u: f64) -> Point3 {
        elclib::value(u, self)
    }
    fn d1(&self, u: f64) -> (Point3, Vector3) {
        elclib::d1(u, self)
    }
    fn intersect_plane(&self, plane: Plane3) -> Result<IntConicQuad, Failure> {
        IntConicQuad::parabola_plane(*self, plane, precision::angular())
    }
}

/// One face of the domain in Inters3d: its plane equation, the fixed axis and value,
/// the two axes read with their ranges and whether each range is `(low, high]`
/// (else `[low, high)`), and the border code.
struct Face {
    equation: [f64; 4],
    fixed: (usize, f64),
    read: [(usize, (f64, f64), bool); 2],
    bord: i32,
}

fn in_range(value: f64, (low, high): (f64, f64), open_low: bool) -> bool {
    if open_low {
        low < value && value <= high
    } else {
        low <= value && value < high
    }
}

/// The outward face normals dotted with the tangent, per border code.
fn border_normal(bord: i32) -> Option<Xyz> {
    match bord {
        1 => Some(Xyz::new(1.0, 0.0, 0.0)),
        2 => Some(Xyz::new(0.0, 1.0, 0.0)),
        3 => Some(Xyz::new(0.0, 0.0, 1.0)),
        4 => Some(Xyz::new(-1.0, 0.0, 0.0)),
        5 => Some(Xyz::new(0.0, -1.0, 0.0)),
        6 => Some(Xyz::new(0.0, 0.0, -1.0)),
        _ => None,
    }
}

impl Tool {
    /// HyprBox: the box of the part of `hyperbola` inside `domain` in `hypr_box` and its
    /// parameter segments, bounded to [-10, 10] (NIZNHY-PKV, 2008) where the other
    /// boxes use `Precision::Infinite`: a whole domain gives a whole box and the
    /// segment [-100, 100]; with no crossing the domain is the box when the apex is
    /// inside it. Errors of IntAna propagate, and a seventh segment raises
    /// Standard_OutOfRange where native code writes past its arrays;
    /// `tool.hypr_box(hyperbola, &domain, &mut hypr_box)?`.
    pub fn hypr_box(
        &mut self,
        hyperbola: Hyperbola3,
        domain: &BoundingBox3,
        hypr_box: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        self.nb_seg = 0;
        hypr_box.set_void();
        if domain.is_whole() {
            hypr_box.set_whole();
            self.set_segment(-100.0, 100.0);
            return Ok(());
        }
        if domain.is_void() {
            return Ok(());
        }
        let nb_pi: i32 = self.inters3d(hyperbola, domain)?;
        if nb_pi > 0 {
            self.cover_crossings_3d(nb_pi as usize, domain, hypr_box);
            let sines: Vec<Option<f64>> = self.crossing_sines_3d(hyperbola, nb_pi as usize);
            self.sweep_hyperbola_segments(&sines, |u| hypr_box.add_point(hyperbola.value(u)))?;
        } else if !domain.is_out_point(hyperbola.value(0.0)) {
            *hypr_box = domain.clone();
            self.set_segment(-100.0, 100.0);
        }
        Ok(())
    }

    /// ParabBox: the 3d counterpart of `parab2d_box` (segments at ±`Precision::Infinite`,
    /// at most six); `tool.parab_box(parabola, &domain, &mut parab_box)?`.
    pub fn parab_box(
        &mut self,
        parabola: Parabola3,
        domain: &BoundingBox3,
        parab_box: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        self.nb_seg = 0;
        parab_box.set_void();
        if domain.is_whole() {
            parab_box.set_whole();
            self.set_infinite_segment();
            return Ok(());
        }
        if domain.is_void() {
            return Ok(());
        }
        let nb_pi: i32 = self.inters3d(parabola, domain)?;
        if nb_pi > 0 {
            self.cover_crossings_3d(nb_pi as usize, domain, parab_box);
            let sines: Vec<Option<f64>> = self.crossing_sines_3d(parabola, nb_pi as usize);
            self.sweep_segments(&sines, |u| parab_box.add_point(parabola.value(u)));
        } else if !domain.is_out_point(parabola.value(0.0)) {
            *parab_box = domain.clone();
            self.set_infinite_segment();
        }
        Ok(())
    }

    fn set_segment(&mut self, begin: f64, end: f64) {
        self.nb_seg = 1;
        self.begin_on_curve[0] = begin;
        self.end_on_curve[0] = end;
    }

    /// The domain grown to the crossing points (std::min/std::max, as natively).
    fn cover_crossings_3d(&self, nb_pi: usize, domain: &BoundingBox3, result: &mut BoundingBox3) {
        let l = domain.limits();
        let mut minimum: [f64; 3] = [l.x_min, l.y_min, l.z_min];
        let mut maximum: [f64; 3] = [l.x_max, l.y_max, l.z_max];
        for npi in 0..nb_pi {
            for (axis, values) in [&self.xint, &self.yint, &self.zint].into_iter().enumerate() {
                minimum[axis] = std_min(minimum[axis], values[npi]);
                maximum[axis] = std_max(maximum[axis], values[npi]);
            }
        }
        result.update_bounds(minimum, maximum);
    }

    fn crossing_sines_3d<C: BoxConic3d>(&self, curve: C, nb_pi: usize) -> Vec<Option<f64>> {
        (0..nb_pi)
            .map(|npi| {
                let (_, tangent) = curve.d1(self.parint[npi]);
                border_normal(self.bord[npi]).map(|normal| normal * *tangent.xyz())
            })
            .collect()
    }

    /// Inters3d: the crossings with the six faces of the domain that are not open, in
    /// the native order (xmin, ymin, zmin, xmax, ymax, zmax), then the native sort and
    /// duplicate removal; returns the count, which the removal may drive below the
    /// number of distinct values (even to zero or below), as natively.
    fn inters3d<C: BoxConic3d>(&mut self, curve: C, domain: &BoundingBox3) -> Result<i32, Failure> {
        let mut nbpi: usize = 0;
        for (closed, face) in domain_faces(domain) {
            if closed {
                let intersection: IntConicQuad = curve.intersect_plane(face_plane(&face)?)?;
                self.keep_face_crossings(&intersection, &face, &mut nbpi)?;
            }
        }
        Ok(self.sort_and_merge_crossings(nbpi as i32))
    }

    /// Stores the crossings of one face that lie in its ranges; both read coordinates
    /// are written before the range test, as natively.
    fn keep_face_crossings(
        &mut self,
        intersection: &IntConicQuad,
        face: &Face,
        nbpi: &mut usize,
    ) -> Result<(), Failure> {
        if !intersection.is_done() || intersection.is_in_quadric()? {
            return Ok(());
        }
        for npi in 1..=intersection.nb_points()? {
            let point: Point3 = intersection.point(npi)?;
            let coordinates: [f64; 3] = [point.x(), point.y(), point.z()];
            let mut inside = true;
            for &(axis, range, open_low) in &face.read {
                self.coordinate_mut(axis)[*nbpi] = coordinates[axis];
                inside = inside && in_range(coordinates[axis], range, open_low);
            }
            if inside {
                self.coordinate_mut(face.fixed.0)[*nbpi] = face.fixed.1;
                self.parint[*nbpi] = intersection.param_on_conic(npi)?;
                self.bord[*nbpi] = face.bord;
                *nbpi += 1;
            }
        }
        Ok(())
    }

    fn coordinate_mut(&mut self, axis: usize) -> &mut [f64; 12] {
        match axis {
            0 => &mut self.xint,
            1 => &mut self.yint,
            _ => &mut self.zint,
        }
    }

    /// The native sort: an exchange sort that also removes a parameter equal (at
    /// `RealSmall`) to its successor, inside the inner loop, so that one pair can be
    /// removed several times.
    fn sort_and_merge_crossings(&mut self, nbpi: i32) -> i32 {
        let mut distinct: i32 = nbpi;
        for i in (1..nbpi).rev() {
            for j in 0..i {
                let (iu, ju) = (i as usize, j as usize);
                if self.parint[iu] <= self.parint[ju] {
                    self.swap_crossings(iu, ju);
                }
                if (i < nbpi - 1) && is_equal(self.parint[iu], self.parint[iu + 1]) {
                    distinct -= 1;
                    for k in i..distinct {
                        self.shift_crossing(k as usize);
                    }
                }
            }
        }
        distinct
    }

    fn swap_crossings(&mut self, i: usize, j: usize) {
        self.parint.swap(i, j);
        self.zint.swap(i, j);
        self.yint.swap(i, j);
        self.xint.swap(i, j);
        self.bord.swap(i, j);
    }

    fn shift_crossing(&mut self, k: usize) {
        self.parint[k] = self.parint[k + 1];
        self.zint[k] = self.zint[k + 1];
        self.yint[k] = self.yint[k + 1];
        self.xint[k] = self.xint[k + 1];
        self.bord[k] = self.bord[k + 1];
    }
}

/// The six faces with whether each is closed (a face on an open side is skipped).
fn domain_faces(domain: &BoundingBox3) -> [(bool, Face); 6] {
    let l = domain.limits();
    let (x, y, z) = ((l.x_min, l.x_max), (l.y_min, l.y_max), (l.z_min, l.z_max));
    let face = |equation: [f64; 4],
                fixed: (usize, f64),
                read: [(usize, (f64, f64), bool); 2],
                bord: i32| Face {
        equation,
        fixed,
        read,
        bord,
    };
    [
        (
            !domain.is_open_xmin(),
            face(
                [1.0, 0.0, 0.0, -l.x_min],
                (0, l.x_min),
                [(1, y, false), (2, z, false)],
                1,
            ),
        ),
        (
            !domain.is_open_ymin(),
            face(
                [0.0, 1.0, 0.0, -l.y_min],
                (1, l.y_min),
                [(0, x, true), (2, z, false)],
                2,
            ),
        ),
        (
            !domain.is_open_zmin(),
            face(
                [0.0, 0.0, 1.0, -l.z_min],
                (2, l.z_min),
                [(0, x, true), (1, y, true)],
                3,
            ),
        ),
        (
            !domain.is_open_xmax(),
            face(
                [-1.0, 0.0, 0.0, l.x_max],
                (0, l.x_max),
                [(1, y, true), (2, z, true)],
                4,
            ),
        ),
        (
            !domain.is_open_ymax(),
            face(
                [0.0, -1.0, 0.0, l.y_max],
                (1, l.y_max),
                [(0, x, false), (2, z, true)],
                5,
            ),
        ),
        (
            !domain.is_open_zmax(),
            face(
                [0.0, 0.0, -1.0, l.z_max],
                (2, l.z_max),
                [(0, x, false), (1, y, false)],
                6,
            ),
        ),
    ]
}

/// gp_Pln(A, B, C, D) of a face.
fn face_plane(face: &Face) -> Result<Plane3, Failure> {
    let [a, b, c, d] = face.equation;
    Plane3::from_equation(a, b, c, d)
}
