// Rust translation of Intf_Tool::Hypr2dBox, Parab2dBox and the Inters2d overloads
// (ModelingAlgorithms/TKGeomAlgo/Intf/Intf_Tool.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The boxes of a 2d hyperbola or parabola in a domain: the crossings with the four
//! domain sides (IntAna2d), sorted by parameter, then the segment loop.
use super::Tool;
use crate::intf::{std_max, std_min};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic, IntPoint};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Direction2, Hyperbola2, Line2, Parabola2, Point2, StandardDirection2, Vector2, Xy,
};

/// The operations the two Inters2d/…2dBox overloads call on their conic.
pub(super) trait BoxConic2d: Copy {
    fn value(&self, u: f64) -> Point2;
    fn d1(&self, u: f64) -> (Point2, Vector2);
    /// IntAna2d_AnaIntersection(conic, IntAna2d_Conic(line)).
    fn intersect_line(&self, line: Line2) -> Result<AnaIntersection, Failure>;
}

impl BoxConic2d for Hyperbola2 {
    fn value(&self, u: f64) -> Point2 {
        elclib::value(u, self)
    }
    fn d1(&self, u: f64) -> (Point2, Vector2) {
        elclib::d1(u, self)
    }
    fn intersect_line(&self, line: Line2) -> Result<AnaIntersection, Failure> {
        AnaIntersection::hyperbola_conic(*self, &Conic::from_line(line))
    }
}

impl BoxConic2d for Parabola2 {
    fn value(&self, u: f64) -> Point2 {
        elclib::value(u, self)
    }
    fn d1(&self, u: f64) -> (Point2, Vector2) {
        elclib::d1(u, self)
    }
    fn intersect_line(&self, line: Line2) -> Result<AnaIntersection, Failure> {
        AnaIntersection::parabola_conic(*self, &Conic::from_line(line))
    }
}

/// One side of the domain in Inters2d: its line, whether the crossing coordinate read
/// is X, the side coordinate, the range, whether the range is `(low, high]` (else
/// `[low, high)`), and the border code.
struct Side {
    line: Line2,
    reads_x: bool,
    fixed: f64,
    range: (f64, f64),
    open_low: bool,
    bord: i32,
}

fn in_range(value: f64, (low, high): (f64, f64), open_low: bool) -> bool {
    if open_low {
        low < value && value <= high
    } else {
        low <= value && value < high
    }
}

fn side_line(origin: (f64, f64), direction: StandardDirection2) -> Line2 {
    Line2::new(
        Point2::new(origin.0, origin.1),
        Direction2::from_standard(direction),
    )
}

/// The inward normals crossed with the tangent, per border code (gp_XY ^ gp_XY).
fn border_normal(bord: i32) -> Option<Xy> {
    match bord {
        1 => Some(Xy::new(-1.0, 0.0)),
        2 => Some(Xy::new(0.0, -1.0)),
        3 => Some(Xy::new(1.0, 0.0)),
        4 => Some(Xy::new(0.0, 1.0)),
        _ => None,
    }
}

impl Tool {
    /// Hypr2dBox: the box of the part of `hyperbola` inside `domain` in `hypr_box` and
    /// its parameter segments (at most six). A whole domain gives a whole box and one
    /// infinite segment; with no border crossing the whole domain is the box when the
    /// apex is inside it. Errors of IntAna2d propagate;
    /// `tool.hypr2d_box(hyperbola, &domain, &mut hypr_box)?`.
    pub fn hypr2d_box(
        &mut self,
        hyperbola: Hyperbola2,
        domain: &BoundingBox2,
        hypr_box: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        self.conic2d_box(hyperbola, domain, hypr_box)
    }

    /// Parab2dBox: the same for a parabola; `tool.parab2d_box(parabola, &domain, &mut parab_box)?`.
    pub fn parab2d_box(
        &mut self,
        parabola: Parabola2,
        domain: &BoundingBox2,
        parab_box: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        self.conic2d_box(parabola, domain, parab_box)
    }

    fn conic2d_box<C: BoxConic2d>(
        &mut self,
        curve: C,
        domain: &BoundingBox2,
        result: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        self.nb_seg = 0;
        result.set_void();
        if domain.is_whole() {
            result.set_whole();
            self.set_infinite_segment();
            return Ok(());
        }
        if domain.is_void() {
            return Ok(());
        }
        let nb_pi: i32 = self.inters2d(curve, domain)?;
        if nb_pi > 0 {
            self.cover_crossings_2d(nb_pi as usize, domain, result);
            self.sort_crossings_by_parameter(nb_pi as usize);
            let sines: Vec<Option<f64>> = (0..nb_pi as usize)
                .map(|npi| {
                    let (_, tangent) = curve.d1(self.parint[npi]);
                    border_normal(self.bord[npi]).map(|normal| normal.crossed(tangent.xy()))
                })
                .collect();
            self.sweep_segments(&sines, |u| result.add_point(curve.value(u)));
        } else if !domain.is_out_point(curve.value(0.0)) {
            *result = domain.clone();
            self.set_infinite_segment();
        }
        Ok(())
    }

    /// The domain grown to the crossing points (std::min/std::max, as natively).
    fn cover_crossings_2d(&self, nb_pi: usize, domain: &BoundingBox2, result: &mut BoundingBox2) {
        let limits = domain.limits();
        let (mut xmin, mut ymin, mut xmax, mut ymax) =
            (limits.x_min, limits.y_min, limits.x_max, limits.y_max);
        for npi in 0..nb_pi {
            xmin = std_min(xmin, self.xint[npi]);
            xmax = std_max(xmax, self.xint[npi]);
            ymin = std_min(ymin, self.yint[npi]);
            ymax = std_max(ymax, self.yint[npi]);
        }
        result.update_bounds([xmin, ymin], [xmax, ymax]);
    }

    /// The native selection sort of the parameters and border codes (the coordinates
    /// are not permuted, as natively).
    fn sort_crossings_by_parameter(&mut self, nb_pi: usize) {
        for npi in 0..nb_pi {
            let mut npk: usize = npi;
            for npj in npi + 1..nb_pi {
                if self.parint[npj] < self.parint[npk] {
                    npk = npj;
                }
            }
            if npk != npi {
                self.parint.swap(npk, npi);
                self.bord.swap(npk, npi);
            }
        }
    }

    /// Inters2d: the crossings with the four sides of the domain that are not open,
    /// in the native order (top, left, bottom, right); returns their count.
    fn inters2d<C: BoxConic2d>(&mut self, curve: C, domain: &BoundingBox2) -> Result<i32, Failure> {
        let l = domain.limits();
        let sides = [
            (
                !domain.is_open_ymax(),
                Side {
                    line: side_line((0.0, l.y_max), StandardDirection2::NegX),
                    reads_x: true,
                    fixed: l.y_max,
                    range: (l.x_min, l.x_max),
                    open_low: true,
                    bord: 1,
                },
            ),
            (
                !domain.is_open_xmin(),
                Side {
                    line: side_line((l.x_min, 0.0), StandardDirection2::NegY),
                    reads_x: false,
                    fixed: l.x_min,
                    range: (l.y_min, l.y_max),
                    open_low: true,
                    bord: 2,
                },
            ),
            (
                !domain.is_open_ymin(),
                Side {
                    line: side_line((0.0, l.y_min), StandardDirection2::X),
                    reads_x: true,
                    fixed: l.y_min,
                    range: (l.x_min, l.x_max),
                    open_low: false,
                    bord: 3,
                },
            ),
            (
                !domain.is_open_xmax(),
                Side {
                    line: side_line((l.x_max, 0.0), StandardDirection2::Y),
                    reads_x: false,
                    fixed: l.x_max,
                    range: (l.y_min, l.y_max),
                    open_low: false,
                    bord: 4,
                },
            ),
        ];
        let mut nbpi: usize = 0;
        for (closed, side) in sides.iter() {
            if *closed {
                let intersection: AnaIntersection = curve.intersect_line(side.line)?;
                self.keep_side_crossings(&intersection, side, &mut nbpi)?;
            }
        }
        Ok(nbpi as i32)
    }

    /// Stores the crossings of one side that lie in its range; the read coordinate is
    /// written before the range test, as natively.
    fn keep_side_crossings(
        &mut self,
        intersection: &AnaIntersection,
        side: &Side,
        nbpi: &mut usize,
    ) -> Result<(), Failure> {
        if !intersection.is_done() || intersection.is_empty()? {
            return Ok(());
        }
        for npi in 1..=intersection.nb_points()? {
            let point: IntPoint = intersection.point(npi)?;
            let value: f64 = if side.reads_x {
                point.value().x()
            } else {
                point.value().y()
            };
            let (read, fixed) = if side.reads_x {
                (&mut self.xint, &mut self.yint)
            } else {
                (&mut self.yint, &mut self.xint)
            };
            read[*nbpi] = value;
            if in_range(value, side.range, side.open_low) {
                fixed[*nbpi] = side.fixed;
                self.parint[*nbpi] = point.param_on_first();
                self.bord[*nbpi] = side.bord;
                *nbpi += 1;
            }
        }
        Ok(())
    }
}
