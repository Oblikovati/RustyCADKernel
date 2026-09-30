// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PolyArc.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_PolyArc`: the polygon of an arc of restriction, sampled on the part of the
//! arc near the polygon it is compared with.
use super::h_inter_tool::arc_of;
use super::polygo::{IntPatchPolygo, IntPatchPolygoBase, polygo_polygon2d};
use crate::int_start::TheArc;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point2;

/// `IntPatch_PolyArc`: the sampled points of the arc (`brise`) with their parameters
/// (`param`), the offset of the points and whether the arc is closed.
///
/// ```ignore
/// let polygon = IntPatchPolyArc::new(&arc, 20, 0.0, 1.0, &other_box)?;
/// assert_eq!(polygon.nb_points()?, 20);
/// ```
#[derive(Clone, Debug)]
pub struct IntPatchPolyArc {
    base: IntPatchPolygoBase,
    brise: Vec<Point2>,
    param: Vec<f64>,
    offsetx: f64,
    offsety: f64,
    ferme: bool,
}

/// The box of the other polygon (`bx0`, `by0`, the limits) and the radius of the
/// significant zone (`r`, `r2`).
struct OtherBox {
    center: [f64; 2],
    limits: [f64; 4],
    r: f64,
    r2: f64,
}

/// `MinMax(a1, a2, amin, amax)`.
fn min_max(a1: f64, a2: f64) -> (f64, f64) {
    if a1 < a2 { (a1, a2) } else { (a2, a1) }
}

impl IntPatchPolyArc {
    /// `IntPatch_PolyArc(A, NbSample, Pfirst, Plast, BoxOtherPolygon)`: `nb_sample` points
    /// of the arc over `[pfirst, plast]`, resampled once over the part near the other
    /// box when that part is less than half of them. Standard_ConstructionError for an
    /// infinite bound or fewer than two samples, and for a void box (`Bnd_Box2d::Get`).
    /// `IntPatchPolyArc::new(&arc, 20, 0.0, 1.0, &other_box)?`.
    pub fn new(
        arc: &TheArc,
        nb_sample: i32,
        pfirst: f64,
        plast: f64,
        box_other_polygon: &BoundingBox2,
    ) -> Result<Self, Failure> {
        let size: usize = nb_sample.max(1) as usize;
        let mut polygon = Self {
            base: IntPatchPolygoBase::new(0.0),
            brise: vec![Point2::default(); size],
            param: vec![0.0; size],
            offsetx: 0.0,
            offsety: 0.0,
            ferme: false,
        };
        if pfirst == real_first() || plast == real_last() || nb_sample <= 1 {
            let reason = format!(
                "bounds [{pfirst}, {plast}] and {nb_sample} samples; expected finite bounds and at least 2 samples"
            );
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                &format!("IntPatch_PolyArc() - {reason}"),
            ));
        }
        let line: &dyn Adaptor2dCurve2d = arc_of(arc, "IntPatch_PolyArc")?;
        let [bxmin, bymin, bxmax, bymax] = box_other_polygon.get()?;
        let mut r: f64 = (bxmax - bxmin) + (bymax - bymin);
        r *= 0.8;
        let mut other = OtherBox {
            center: [(bxmax + bxmin) * 0.5, (bymax + bymin) * 0.5],
            limits: [bxmin, bymin, bxmax, bymax],
            r,
            r2: r * r * 49.0,
        };
        polygon.sample_loop(line, nb_sample, (pfirst, plast), &mut other)?;
        polygon.base.error *= 1.2;
        if polygon.base.error < 0.00000001 {
            polygon.base.error = 0.00000001;
        }
        polygon.base.bounding.enlarge(polygon.base.error);
        polygon.ferme = line.value(pfirst)?.distance(line.value(plast)?) <= 1e-7;
        Ok(polygon)
    }

    /// The sampling, repeated (at most 10 times) while no segment is near the other
    /// box, the radius doubling each time; resampled once over the significant zone.
    fn sample_loop(
        &mut self,
        line: &dyn Adaptor2dCurve2d,
        nb_sample: i32,
        (mut pdeb, mut pfin): (f64, f64),
        other: &mut OtherBox,
    ) -> Result<(), Failure> {
        let mut nbloop: i32 = 0;
        loop {
            nbloop += 1;
            let (mut index_inf, mut index_sup) =
                self.sample(line, nb_sample, (pdeb, pfin), other)?;
            if index_inf > index_sup {
                other.r += other.r;
                other.r2 = other.r * other.r * 49.0;
            } else if (index_sup - index_inf) < (nb_sample / 2) {
                // Too few significant points: sample once more between them.
                nbloop = 10;
                pdeb = self.param[(index_inf - 1) as usize];
                pfin = self.param[(index_sup - 1) as usize];
                (index_inf, index_sup) = (nb_sample + 1, 0);
            }
            if !(index_inf > index_sup && nbloop <= 10) {
                return Ok(());
            }
        }
    }

    /// One sampling of `[pdeb, pfin]`: the points, the box, the deflection (the distance
    /// of the mid-parameter point to the chord middle) and the first and last indices of
    /// the significant zone.
    fn sample(
        &mut self,
        line: &dyn Adaptor2dCurve2d,
        nb_sample: i32,
        (pdeb, pfin): (f64, f64),
        other: &OtherBox,
    ) -> Result<(i32, i32), Failure> {
        let (mut index_inf, mut index_sup) = (nb_sample + 1, 0);
        let pas: f64 = (pfin - pdeb) / f64::from(nb_sample - 1);
        self.param[0] = pdeb;
        let p: Point2 = line.d0(pdeb)?;
        let (mut xs, mut ys) = (p.x(), p.y());
        self.brise[0] = Point2::new(xs, ys);
        self.base.bounding.set_void();
        self.base.bounding.add_point(self.brise[0]);
        self.base.error = 0.0;
        for i in 2..=nb_sample {
            let at: usize = (i - 1) as usize;
            self.param[at] = pdeb + f64::from(i - 1) * pas;
            let p: Point2 = line.d0(self.param[at])?;
            let (x, y) = (p.x(), p.y());
            self.brise[at] = Point2::new(x, y);
            let (xxs, yys) = (0.5 * (xs + x), 0.5 * (ys + y));
            if is_significant(other, [xs, ys, x, y], [xxs, yys]) {
                // MSV: the previous and the next segments too, since only the middle point
                // is checked (see BUC60946).
                if index_inf > i {
                    index_inf = (i - 2).max(1);
                }
                if index_sup < i {
                    index_sup = (i + 1).min(nb_sample);
                }
            }
            self.base.bounding.add_point(self.brise[at]);
            let pm: Point2 = line.d0(self.param[at] - pas * 0.5)?;
            let (xm, ym) = (pm.x() - xxs, pm.y() - yys);
            let xm: f64 = (xm * xm + ym * ym).sqrt();
            self.base.error = self.base.error.max(xm);
            (xs, ys) = (x, y);
        }
        Ok((index_inf, index_sup))
    }

    /// `Closed()`: whether the ends of the arc meet within 1e-7. `polygon.is_closed()`.
    pub fn is_closed(&self) -> bool {
        self.ferme
    }

    /// `Parameter(Index)`: the parameter of the sample point; Standard_OutOfRange outside
    /// `1..=NbPoints()`. `polygon.parameter(1)?`.
    pub fn parameter(&self, index: i32) -> Result<f64, Failure> {
        crate::int_start::failures::sequence_value(
            &self.param,
            index,
            "IntPatch_PolyArc::Parameter",
        )
        .copied()
    }

    /// `SetOffset(OffsetX, OffsetY)`: moves the points and the box; the box is rebuilt
    /// from its limits (Standard_ConstructionError for a void box).
    /// `polygon.set_offset(1.0, 0.0)?`.
    pub fn set_offset(&mut self, ox: f64, oy: f64) -> Result<(), Failure> {
        let [xmin, ymin, xmax, ymax] = self.base.bounding.get()?;
        let g: f64 = self.base.bounding.gap();
        self.base.bounding.set_void();
        self.base.bounding.update_bounds(
            [xmin - self.offsetx, ymin - self.offsety],
            [xmax - self.offsetx, ymax - self.offsety],
        );
        (self.offsetx, self.offsety) = (ox, oy);
        self.base.bounding.update_bounds(
            [xmin + self.offsetx, ymin + self.offsety],
            [xmax + self.offsetx, ymax + self.offsety],
        );
        self.base.bounding.set_gap(g);
        Ok(())
    }
}

/// Whether the segment `(xs, ys) (x, y)` of middle `(xxs, yys)` is near the other box:
/// its middle within `r` (Manhattan) of the box center, or, for a segment longer than
/// `7 r`, its box meeting the other box (MSV: cda 002 H2).
fn is_significant(other: &OtherBox, [xs, ys, x, y]: [f64; 4], [xxs, yys]: [f64; 2]) -> bool {
    let [bx0, by0] = other.center;
    let is_mid_pt_in_box: bool = ((bx0 - xxs).abs() + (by0 - yys).abs()) < other.r;
    let mut is_seg_out: bool = true;
    if !is_mid_pt_in_box {
        let d: f64 = (x - xs) * (x - xs) + (y - ys) * (y - ys);
        if d > other.r2 {
            let [bxmin, bymin, bxmax, bymax] = other.limits;
            let (xmin, xmax) = min_max(xs, x);
            let (ymin, ymax) = min_max(ys, y);
            is_seg_out = xmax < bxmin || xmin > bxmax || ymax < bymin || ymin > bymax;
        }
    }
    is_mid_pt_in_box || !is_seg_out
}

impl IntPatchPolygo for IntPatchPolyArc {
    fn polygo_base(&self) -> &IntPatchPolygoBase {
        &self.base
    }
    /// `NbPoints()`: the number of samples.
    fn nb_points(&self) -> Result<i32, Failure> {
        Ok(self.brise.len() as i32)
    }
    /// `Point(Index)`: the sample point moved by the offset; Standard_OutOfRange
    /// outside `1..=NbPoints()`.
    fn point(&self, index: i32) -> Result<Point2, Failure> {
        let p: Point2 = *crate::int_start::failures::sequence_value(
            &self.brise,
            index,
            "IntPatch_PolyArc::Point",
        )?;
        if self.offsetx == 0.0 && self.offsety == 0.0 {
            return Ok(p);
        }
        Ok(Point2::new(p.x() + self.offsetx, p.y() + self.offsety))
    }
}

polygo_polygon2d!(IntPatchPolyArc);
