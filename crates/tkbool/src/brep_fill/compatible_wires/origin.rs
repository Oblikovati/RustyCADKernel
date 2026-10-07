// Rust translation of BRepFill_CompatibleWires::ComputeOrigin
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_CompatibleWires.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ComputeOrigin`: closed sections of one edge count restarted and oriented to
//! follow the previous section as closely as possible (the least sum of distances
//! between their vertices, and sample points for sections of at most two edges).
use super::BRepFillCompatibleWires;
use super::helpers::{nth, plane_of_wire, wire_closed};
use crate::brep_fill::ThruSectionErrorStatus;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{self, Edge, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Plane3, Point3, Vector3, Xyz};

/// The vertices and edges of a section in wire explorer order.
struct SectionWalk {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
}

/// The vertex and edge sequences of the wire explorer.
fn walk(wire: &Wire) -> Result<SectionWalk, Failure> {
    let mut sequence = SectionWalk {
        vertices: Vec::new(),
        edges: Vec::new(),
    };
    let mut explorer = WireExplorer::with_wire(wire)?;
    while explorer.more() {
        sequence.vertices.push(explorer.current_vertex().clone());
        sequence.edges.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(sequence)
}

/// The plane of the wire, `gp_Pln()` (the XOY plane) when it has none.
fn plane_or_default(wire: &Wire) -> Result<Plane3, Failure> {
    Ok(plane_of_wire(wire)?.unwrap_or_default())
}

/// The comparison of a section with the previous one: the previous sequences, the
/// offset that brings the current section on the previous one and the number of
/// samples per edge (0 for sections of more than two edges).
struct Matcher<'a> {
    prev: &'a SectionWalk,
    cur: &'a SectionWalk,
    offset: Xyz,
    nb_samples: i32,
}

impl Matcher<'_> {
    /// The distance between the vertex `n` of the previous section and the vertex `k`
    /// of the current one moved by the offset.
    fn vertex_distance(&self, n: usize, k: usize) -> Result<f64, Failure> {
        let pprev: Point3 = BRepTool::pnt(nth(&self.prev.vertices, n, "previous vertex")?)?;
        let p: Point3 =
            Point3::from(BRepTool::pnt(nth(&self.cur.vertices, k, "vertex")?)?.xyz() + self.offset);
        Ok(pprev.distance(&p))
    }

    /// The distances between the inner sample points of the previous edge `n` and the
    /// current edge `k`, the current edge walked forward or backward.
    fn sample_distance(&self, n: usize, k: usize, backward: bool) -> Result<f64, Failure> {
        let prev_edge: &Edge = nth(&self.prev.edges, n, "previous edge")?;
        let cur_edge: &Edge = nth(&self.cur.edges, k, "edge")?;
        let prev_curve = BRepAdaptorCurve::with_edge(prev_edge)?;
        let curve = BRepAdaptorCurve::with_edge(cur_edge)?;
        let ns: f64 = f64::from(self.nb_samples);
        let (prev_first, cur_first) = (prev_curve.first_parameter()?, curve.first_parameter()?);
        let sample_on_prev: f64 = (prev_curve.last_parameter()? - prev_first) / ns;
        let sample_on_cur: f64 = (curve.last_parameter()? - cur_first) / ns;
        let mut sum: f64 = 0.;
        for nbs in 1..self.nb_samples {
            let (fwd, bwd) = (f64::from(nbs), f64::from(self.nb_samples - nbs));
            let prev_factor: f64 = if prev_edge.orientation() == Orientation::Forward {
                fwd
            } else {
                bwd
            };
            let par_on_prev: f64 = prev_first + prev_factor * sample_on_prev;
            let cur_forward: bool = (cur_edge.orientation() == Orientation::Forward) != backward;
            let cur_factor: f64 = if cur_forward { fwd } else { bwd };
            let par_on_cur: f64 = cur_first + cur_factor * sample_on_cur;
            let pon_prev: Point3 = prev_curve.value(par_on_prev)?;
            let pon_cur: Point3 = Point3::from(curve.value(par_on_cur)?.xyz() + self.offset);
            sum += pon_prev.distance(&pon_cur);
        }
        Ok(sum)
    }

    /// The vertex distance and, with samples, the sample distances of the pair `(n, k)`
    /// (edge `k_edge` of the current section).
    fn pair_distance(
        &self,
        n: usize,
        k: usize,
        k_edge: usize,
        backward: bool,
    ) -> Result<f64, Failure> {
        let mut sum: f64 = self.vertex_distance(n, k)?;
        if self.nb_samples > 0 {
            sum += self.sample_distance(n, k_edge, backward)?;
        }
        Ok(sum)
    }

    /// The sum of distances when the current section starts at vertex `j`, forward.
    fn forward_sum(&self, j: usize) -> Result<f64, Failure> {
        let the_length: usize = self.prev.vertices.len();
        let mut sum: f64 = 0.;
        for (n, k) in (1..).zip((j..=the_length).chain(1..j)) {
            sum += self.pair_distance(n, k, k, false)?;
        }
        Ok(sum)
    }

    /// The sum of distances when the current section starts at vertex `j`, backward
    /// (the edge before each vertex is compared, the last edge before the first vertex).
    fn backward_sum(&self, j: usize) -> Result<f64, Failure> {
        let the_length: usize = self.prev.vertices.len();
        let mut sum: f64 = 0.;
        let mut n: usize = 1;
        for k in (1..=j).rev() {
            let k_cur: usize = if k == 1 { the_length } else { k - 1 };
            sum += self.pair_distance(n, k, k_cur, true)?;
            n += 1;
        }
        for k in (j + 1..=the_length).rev() {
            sum += self.pair_distance(n, k, k - 1, true)?;
            n += 1;
        }
        Ok(sum)
    }

    /// The start vertex and direction with the least sum of distances (forward first
    /// at each start, the first minimum kept).
    fn best_start(&self) -> Result<(usize, bool), Failure> {
        let (mut jmin, mut forward, mut min_sum_dist) = (1, false, precision::infinite());
        for j in 1..=self.prev.vertices.len() {
            let sum: f64 = self.forward_sum(j)?;
            if sum < min_sum_dist {
                (min_sum_dist, jmin, forward) = (sum, j, true);
            }
            let sum: f64 = self.backward_sum(j)?;
            if sum < min_sum_dist {
                (min_sum_dist, jmin, forward) = (sum, j, false);
            }
        }
        Ok((jmin, forward))
    }
}

/// The section rebuilt from vertex `jmin`, forward or with its edges reversed
/// backward, and the sequences of the rebuilt section for the next comparison.
fn rebuild(cur: &SectionWalk, jmin: usize, forward: bool) -> Result<(Wire, SectionWalk), Failure> {
    let builder = BRepBuilder::default();
    let mut new_wire = Wire::default();
    builder.make_wire(&mut new_wire);
    let the_length: usize = cur.edges.len();
    let mut next = SectionWalk {
        vertices: Vec::new(),
        edges: Vec::new(),
    };
    if forward {
        for j in (jmin..=the_length).chain(1..jmin) {
            builder.add(&mut new_wire, nth(&cur.edges, j, "edge")?)?;
            next.vertices.push(nth(&cur.vertices, j, "vertex")?.clone());
            next.edges.push(nth(&cur.edges, j, "edge")?.clone());
        }
    } else {
        for j in (1..jmin).rev().chain((jmin..=the_length).rev()) {
            let reversed: Edge = topods::edge(&nth(&cur.edges, j, "edge")?.reversed())?;
            builder.add(&mut new_wire, &reversed)?;
            next.edges.push(reversed);
        }
        for j in (1..=jmin).rev().chain((jmin + 1..=the_length).rev()) {
            next.vertices.push(nth(&cur.vertices, j, "vertex")?.clone());
        }
    }
    new_wire.set_closed(true)?;
    new_wire.set_orientation(Orientation::Forward);
    Ok((new_wire, next))
}

impl BRepFillCompatibleWires {
    /// `ComputeOrigin(polar)`: the closed sections (all must be; NotSameTopology
    /// otherwise) restarted and oriented one after the other.
    pub(super) fn compute_origin(&mut self, _polar: bool) -> Result<(), Failure> {
        let nb_sects: usize = self.my_work.len();
        let ideb: usize = if self.my_degen1 { 2 } else { 1 };
        let ifin: usize = if self.my_degen2 {
            nb_sects - 1
        } else {
            nb_sects
        };
        let v_closed: bool = !self.my_degen1
            && !self.my_degen2
            && self.my_work[ideb - 1].is_same(&self.my_work[ifin - 1]);
        let mut all_closed: bool = true;
        for i in ideb..=ifin {
            all_closed = all_closed && wire_closed(&self.my_work[i - 1])?;
        }
        if !all_closed {
            self.my_status = ThruSectionErrorStatus::NotSameTopology;
            return Ok(());
        }
        self.reorder_sections(ideb, ifin)?;
        // Blocking sections?
        if v_closed {
            self.my_work[nb_sects - 1] = self.my_work[0].clone();
        }
        Ok(())
    }

    /// Each section after the first matched to the previous one, its barycenter
    /// brought onto the previous one along the plane of the first section.
    fn reorder_sections(&mut self, ideb: usize, ifin: usize) -> Result<(), Failure> {
        let first_wire: Wire = topods::wire(&self.my_work[ideb - 1])?;
        let mut prev: SectionWalk = walk(&first_wire)?;
        let the_length: usize = prev.vertices.len();
        let nb_samples: i32 = if the_length <= 2 { 4 } else { 0 };
        let first_plane: Plane3 = plane_or_default(&first_wire)?;
        let mut prev_bary: Point3 = first_plane.location();
        let normal: Vector3 = Vector3::from(first_plane.axis().direction());
        for i in ideb + 1..=ifin {
            let a_wire: Wire = topods::wire(&self.my_work[i - 1])?;
            // The offset: the current barycenter projected on the first plane to the
            // previous barycenter.
            let mut cur_bary: Point3 = plane_or_default(&a_wire)?.location();
            let a_vec = Vector3::between(prev_bary, cur_bary);
            let offset_proj: Vector3 = normal * a_vec.dot(normal);
            cur_bary.translate(-offset_proj);
            let offset = Vector3::between(cur_bary, prev_bary);
            let cur: SectionWalk = walk(&a_wire)?;
            let (jmin, forward) = if i == self.my_work.len() && self.my_degen2 {
                // The last point section.
                (1, true)
            } else {
                Matcher {
                    prev: &prev,
                    cur: &cur,
                    offset: *offset.xyz(),
                    nb_samples,
                }
                .best_start()?
            };
            let (new_wire, next) = rebuild(&cur, jmin, forward)?;
            self.my_work[i - 1] = new_wire.into_shape();
            prev = next;
            prev_bary = cur_bary;
        }
        Ok(())
    }
}
