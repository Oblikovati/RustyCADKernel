// Rust translation of IntPatch_InterferencePolyhedron::TangentZoneValue
// (IntPatch_InterferencePolyhedron.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_InterferencePolyhedron::TangentZoneValue`: the zone of tangency of two
//! coplanar triangles, from the positions of the vertices of each triangle in the
//! other one and the crossings of their edges.
use super::super::polyhedron::IntPatchPolyhedron;
use super::{IntPatchInterferencePolyhedron, POURCENT3};
use crate::intf::{PIType, SectionPoint, TangentZone, contain};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

/// The points of the zone under construction (`Tpi`, `nbpInt`) and the positions of
/// the vertices of each triangle (`tOP`, `tTP`).
struct ZonePoints {
    tpi: Vec<SectionPoint>,
    nbp_int: i32,
    t_op: [PIType; 3],
    t_tp: [PIType; 3],
}

impl ZonePoints {
    fn push(&mut self, point: SectionPoint) {
        self.tpi.push(point);
        self.nbp_int += 1;
    }
}

impl IntPatchInterferencePolyhedron {
    /// `TangentZoneValue(TheTZ, FirstPol, Tri1, SeconPol, Tri2)`: fills `zone` with the
    /// vertices of each triangle inside the other one; when neither triangle is inside
    /// the other, the edge crossings (at most five points in all) are offered to the zone
    /// too. True when the zone has at least three points.
    pub(super) fn tangent_zone_value(
        &self,
        zone: &mut TangentZone,
        first: &IntPatchPolyhedron,
        tri1: i32,
        second: &IntPatchPolyhedron,
        tri2: i32,
    ) -> Result<bool, Failure> {
        let (po, pt) = (self.object_points(first)?, self.tool_points(second)?);
        let mut z = ZonePoints {
            tpi: Vec::new(),
            nbp_int: 0,
            t_op: [PIType::External; 3],
            t_tp: [PIType::External; 3],
        };
        for nob in 0..3 {
            self.object_vertex(&mut z, nob, (po, pt), tri2);
        }
        let mut finished: bool = true;
        if z.nbp_int < 3 {
            for nou in 0..3 {
                self.tool_vertex(&mut z, nou, (po, pt), tri1);
            }
            finished = z.t_tp.iter().all(|t| *t != PIType::External);
        }
        for point in &z.tpi[..z.nbp_int as usize] {
            zone.append(*point);
        }
        if !finished {
            self.edge_crossings(&mut z, zone, pt);
        }
        if z.nbp_int < 3 {
            z.nbp_int = 0;
        }
        Ok(z.nbp_int > 0)
    }

    /// The position of the object vertex `nob` in the tool triangle: on a vertex, on
    /// an edge, or inside the face.
    fn object_vertex(
        &self,
        z: &mut ZonePoints,
        nob: usize,
        (po, pt): ([Xyz; 3], [Xyz; 3]),
        tri2: i32,
    ) {
        let tol: f64 = self.tolerance;
        let vertex = |z: &mut ZonePoints, dime: PIType, a1: i32, a2: i32, par: f64| {
            z.push(SectionPoint::with_data(
                Point3::from(po[nob]),
                PIType::Vertex,
                self.oi[nob],
                0,
                0.0,
                dime,
                a1,
                a2,
                par,
                1.0,
            ));
        };
        for nou in 0..3 {
            let nou2: usize = POURCENT3[nou + 1];
            if self.dp_op_t[nob][nou] <= tol {
                vertex(z, PIType::Vertex, self.ti[nou], 0, 0.0);
                (z.t_op[nob], z.t_tp[nou]) = (PIType::Vertex, PIType::Vertex);
                break;
            }
            let (a, b) = (self.dp_op_t[nob][nou], self.dp_op_t[nob][nou2]);
            if self.dp_oe_t[nob][nou].abs() <= tol && a + b < self.vtt[nou].modulus() {
                let mut par: f64 = a / (a + b);
                if self.ti[nou] > self.ti[nou2] {
                    par = 1.0 - par;
                }
                let (lo, hi) = (
                    self.ti[nou].min(self.ti[nou2]),
                    self.ti[nou].max(self.ti[nou2]),
                );
                vertex(z, PIType::Edge, lo, hi, par);
                z.t_op[nob] = PIType::Edge;
                break;
            }
        }
        if z.t_op[nob] == PIType::External
            && contain(
                Point3::from(pt[0]),
                Point3::from(pt[1]),
                Point3::from(pt[2]),
                Point3::from(po[nob]),
            )
        {
            vertex(z, PIType::Face, tri2, 0, 0.0);
            z.t_op[nob] = PIType::Face;
        }
    }

    /// The position of the tool vertex `nou`, not yet placed, in the object triangle:
    /// on an edge or inside the face.
    fn tool_vertex(
        &self,
        z: &mut ZonePoints,
        nou: usize,
        (po, pt): ([Xyz; 3], [Xyz; 3]),
        tri1: i32,
    ) {
        if z.t_tp[nou] != PIType::External {
            return;
        }
        let tol: f64 = self.tolerance;
        for nob in 0..3 {
            let nob2: usize = POURCENT3[nob + 1];
            let (a, b) = (self.dp_op_t[nob][nou], self.dp_op_t[nob2][nou]);
            if self.de_op_t[nob][nou].abs() <= tol && a + b < self.voo[nob].modulus() {
                let mut par: f64 = a / (a + b);
                if self.oi[nob] > self.oi[nob2] {
                    par = 1.0 - par;
                }
                let (lo, hi) = (
                    self.oi[nob].min(self.oi[nob2]),
                    self.oi[nob].max(self.oi[nob2]),
                );
                z.push(SectionPoint::with_data(
                    Point3::from(pt[nou]),
                    PIType::Edge,
                    lo,
                    hi,
                    par,
                    PIType::Vertex,
                    self.ti[nou],
                    0,
                    0.0,
                    1.0,
                ));
                z.t_tp[nou] = PIType::Edge;
                break;
            }
        }
        if z.t_tp[nou] == PIType::External
            && contain(
                Point3::from(po[0]),
                Point3::from(po[1]),
                Point3::from(po[2]),
                Point3::from(pt[nou]),
            )
        {
            z.push(SectionPoint::with_data(
                Point3::from(pt[nou]),
                PIType::Face,
                tri1,
                0,
                0.0,
                PIType::Vertex,
                self.ti[nou],
                0,
                0.0,
                1.0,
            ));
            z.t_tp[nou] = PIType::Face;
        }
    }

    /// The edge/edge crossings of the two triangles, while fewer than five points are
    /// known; each is offered to the zone (`Intf_TangentZone::Insert`, which never
    /// accepts one natively), then offered again in reverse order.
    fn edge_crossings(&self, z: &mut ZonePoints, zone: &mut TangentZone, pt: [Xyz; 3]) {
        let mut pi_to_insert: Vec<i32> = Vec::new();
        'object: for nob in 0..3 {
            let nob2: usize = POURCENT3[nob + 1];
            for nou in 0..3 {
                let nou2: usize = POURCENT3[nou + 1];
                let crossing: bool = self.dp_oe_t[nob][nou] * self.dp_oe_t[nob2][nou] < 0.0
                    && self.de_op_t[nob][nou] * self.de_op_t[nob][nou2] < 0.0;
                if !crossing {
                    continue;
                }
                if z.nbp_int >= 5 {
                    break;
                }
                let point: SectionPoint = self.crossing_point([nob, nob2, nou, nou2], pt);
                z.push(point);
                if !zone.insert(&point) {
                    pi_to_insert.push(z.nbp_int);
                }
            }
            if z.nbp_int >= 5 {
                break 'object;
            }
        }
        retry_insertions(zone, &z.tpi, &mut pi_to_insert);
    }

    /// The crossing of the object edge `nob nob2` and the tool edge `nou nou2`.
    fn crossing_point(&self, [nob, nob2, nou, nou2]: [usize; 4], pt: [Xyz; 3]) -> SectionPoint {
        let mut par_o: f64 =
            self.dp_oe_t[nob][nou] / (self.dp_oe_t[nob][nou] - self.dp_oe_t[nob2][nou]);
        let mut par_t: f64 =
            self.de_op_t[nob][nou] / (self.de_op_t[nob][nou] - self.de_op_t[nob][nou2]);
        let lepi: Point3 = Point3::from(pt[nou]).translated(Vector3::from(self.vtt[nou] * par_t));
        if self.oi[nob] > self.oi[nob2] {
            par_o = 1.0 - par_o;
        }
        if self.ti[nou] > self.ti[nou2] {
            par_t = 1.0 - par_t;
        }
        SectionPoint::with_data(
            lepi,
            PIType::Edge,
            self.oi[nob].min(self.oi[nob2]),
            self.oi[nob].max(self.oi[nob2]),
            par_o,
            PIType::Edge,
            self.ti[nou].min(self.ti[nou2]),
            self.ti[nou].max(self.ti[nou2]),
            par_t,
            self.incidence,
        )
    }
}

/// The second chance of the points the zone refused, from the last one: each accepted
/// one leaves the list and the scan restarts from the new last one. The native code
/// then dumps the refused points (`Intf_SectionPoint::Dump`, which prints nothing).
fn retry_insertions(zone: &mut TangentZone, tpi: &[SectionPoint], pi_to_insert: &mut Vec<i32>) {
    let mut nob: i32 = pi_to_insert.len() as i32 - 1;
    while nob >= 0 {
        while !zone.insert(&tpi[(pi_to_insert[nob as usize] - 1) as usize]) {
            nob -= 1;
            if nob < 0 {
                break;
            }
        }
        if nob >= 0 {
            pi_to_insert.remove(nob as usize);
            nob = pi_to_insert.len() as i32 - 1;
        }
    }
}
