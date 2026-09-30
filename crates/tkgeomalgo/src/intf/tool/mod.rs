// Rust translation of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_Tool: the boxes and parameter segments of infinite curves (lines, hyperbolas,
//! parabolas) inside a bounded domain.
mod conics2d;
mod conics3d;
mod segments;

use crate::intf::{std_max, std_min};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{Line2, Line3};

/// Intf_Tool: computes the part of a curve inside a domain box, as a box and up to six
/// parameter segments. The native fixed arrays are kept, including their values from
/// earlier calls, which only `nb_segments` bounds.
///
/// ```
/// use rustycad_tkgeomalgo::intf::Tool;
/// use rustycad_tkmath::bnd::BoundingBox2;
/// use rustycad_tkmath::gp::{Direction2, Line2, Point2};
/// let mut domain = BoundingBox2::default();
/// domain.update_bounds([-1.0, -1.0], [1.0, 1.0]);
/// let mut tool = Tool::new();
/// let mut line_box = BoundingBox2::default();
/// tool.lin2d_box(Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap()), &domain, &mut line_box);
/// assert_eq!((tool.nb_segments(), tool.begin_param(1).unwrap(), tool.end_param(1).unwrap()), (1, -1.0, 1.0));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Tool {
    nb_seg: i32,
    begin_on_curve: [f64; 6],
    end_on_curve: [f64; 6],
    bord: [i32; 12],
    xint: [f64; 12],
    yint: [f64; 12],
    zint: [f64; 12],
    parint: [f64; 12],
}

impl Default for Tool {
    fn default() -> Self {
        Self::new()
    }
}

/// The clip of a line against one axis slab of the domain.
enum AxisClip {
    /// The line crosses the slab between these parameters.
    Crossing(f64, f64),
    /// The line is parallel to the slab, at this coordinate inside it.
    Parallel(f64),
    /// The line is parallel to the slab and outside it.
    Outside,
}

/// One axis of a line and of the domain: location and direction components, bounds and
/// open flags.
struct AxisSlab {
    location: f64,
    direction: f64,
    min: f64,
    max: f64,
    open_min: bool,
    open_max: bool,
}

impl AxisSlab {
    fn clip(&self) -> AxisClip {
        let infinite: f64 = precision::infinite();
        let at = |bound: f64| (bound - self.location) / self.direction;
        if self.direction > 0.0 {
            let low = if self.open_min {
                -infinite
            } else {
                at(self.min)
            };
            let high = if self.open_max {
                infinite
            } else {
                at(self.max)
            };
            return AxisClip::Crossing(low, high);
        }
        if self.direction < 0.0 {
            let low = if self.open_max {
                -infinite
            } else {
                at(self.max)
            };
            let high = if self.open_min {
                infinite
            } else {
                at(self.min)
            };
            return AxisClip::Crossing(low, high);
        }
        if self.location < self.min || self.max < self.location {
            return AxisClip::Outside;
        }
        AxisClip::Parallel(self.location)
    }

    /// The range of the coordinate over `[parmin, parmax]`, or the parallel coordinate.
    fn range(&self, clip: &AxisClip, parmin: f64, parmax: f64) -> (f64, f64) {
        if let AxisClip::Parallel(value) = clip {
            return (*value, *value);
        }
        let par1: f64 = self.location + parmin * self.direction;
        let par2: f64 = self.location + parmax * self.direction;
        (std_min(par1, par2), std_max(par1, par2))
    }
}

impl Tool {
    /// Intf_Tool(): no segment, every array zeroed; `Tool::new()`.
    pub fn new() -> Self {
        Self {
            nb_seg: 0,
            begin_on_curve: [0.0; 6],
            end_on_curve: [0.0; 6],
            bord: [0; 12],
            xint: [0.0; 12],
            yint: [0.0; 12],
            zint: [0.0; 12],
            parint: [0.0; 12],
        }
    }

    /// Lin2dBox: the box of the part of `line` inside `domain` in `line_box`, and its
    /// parameter segment. A whole domain gives the infinite ray box and the segment
    /// ±`Precision::Infinite`; a void domain, or a line parallel to a side and outside
    /// it, gives no segment; `tool.lin2d_box(line, &domain, &mut line_box)`.
    pub fn lin2d_box(&mut self, line: Line2, domain: &BoundingBox2, line_box: &mut BoundingBox2) {
        self.nb_seg = 0;
        line_box.set_void();
        if domain.is_whole() {
            line_box.set_ray(line.location(), line.direction());
            line_box.add_direction(line.direction().reversed());
            self.set_infinite_segment();
            return;
        }
        if domain.is_void() {
            return;
        }
        let limits = domain.limits();
        let (location, direction) = (line.location(), line.direction());
        let slabs = [
            slab(
                location.x(),
                direction.x(),
                (limits.x_min, limits.x_max),
                (domain.is_open_xmin(), domain.is_open_xmax()),
            ),
            slab(
                location.y(),
                direction.y(),
                (limits.y_min, limits.y_max),
                (domain.is_open_ymin(), domain.is_open_ymax()),
            ),
        ];
        if let Some((minimum, maximum)) = self.clip_line(&slabs) {
            line_box.update_bounds([minimum[0], minimum[1]], [maximum[0], maximum[1]]);
        }
    }

    /// LinBox: the 3d counterpart of `lin2d_box`; `tool.lin_box(line, &domain, &mut line_box)`.
    pub fn lin_box(&mut self, line: Line3, domain: &BoundingBox3, line_box: &mut BoundingBox3) {
        self.nb_seg = 0;
        line_box.set_void();
        if domain.is_whole() {
            line_box.set_ray(line.location(), line.direction());
            line_box.add_direction(line.direction().reversed());
            self.set_infinite_segment();
            return;
        }
        if domain.is_void() {
            return;
        }
        let limits = domain.limits();
        let (location, direction) = (line.location(), line.direction());
        let slabs = [
            slab(
                location.x(),
                direction.x(),
                (limits.x_min, limits.x_max),
                (domain.is_open_xmin(), domain.is_open_xmax()),
            ),
            slab(
                location.y(),
                direction.y(),
                (limits.y_min, limits.y_max),
                (domain.is_open_ymin(), domain.is_open_ymax()),
            ),
            slab(
                location.z(),
                direction.z(),
                (limits.z_min, limits.z_max),
                (domain.is_open_zmin(), domain.is_open_zmax()),
            ),
        ];
        if let Some((minimum, maximum)) = self.clip_line(&slabs) {
            line_box.update_bounds(
                [minimum[0], minimum[1], minimum[2]],
                [maximum[0], maximum[1], maximum[2]],
            );
        }
    }

    /// NbSegments; `tool.nb_segments()`.
    pub fn nb_segments(&self) -> i32 {
        self.nb_seg
    }

    /// BeginParam: the first parameter of segment `segment` (one-based); raises
    /// Standard_OutOfRange outside 1..=NbSegments; `tool.begin_param(1)?`.
    pub fn begin_param(&self, segment: i32) -> Result<f64, Failure> {
        self.check_segment(segment, "Intf_Tool::BeginParam")?;
        Ok(self.begin_on_curve[(segment - 1) as usize])
    }

    /// EndParam: the last parameter of segment `segment`; raises Standard_OutOfRange
    /// outside 1..=NbSegments; `tool.end_param(1)?`.
    pub fn end_param(&self, segment: i32) -> Result<f64, Failure> {
        self.check_segment(segment, "Intf_Tool::EndParam")?;
        Ok(self.end_on_curve[(segment - 1) as usize])
    }

    fn check_segment(&self, segment: i32, member: &str) -> Result<(), Failure> {
        if segment < 1 || segment > self.nb_seg {
            return Err(Failure::out_of_range(&format!(
                "{member}: segment {segment}; expected 1..={}",
                self.nb_seg
            )));
        }
        Ok(())
    }

    /// The single segment ±`Precision::Infinite` of a whole domain.
    fn set_infinite_segment(&mut self) {
        self.nb_seg = 1;
        self.begin_on_curve[0] = -precision::infinite();
        self.end_on_curve[0] = precision::infinite();
    }

    /// Clips the line against every slab (the first axis sets the parameter range, the
    /// others narrow it with std::max/std::min, as natively), stores the segment and
    /// returns the coordinate ranges; None when the line misses a parallel slab.
    fn clip_line<const N: usize>(&mut self, slabs: &[AxisSlab; N]) -> Option<([f64; N], [f64; N])> {
        let mut clips: Vec<AxisClip> = Vec::with_capacity(N);
        let (mut parmin, mut parmax) = (-precision::infinite(), precision::infinite());
        for (axis, slab) in slabs.iter().enumerate() {
            let clip: AxisClip = slab.clip();
            match clip {
                AxisClip::Outside => return None,
                AxisClip::Crossing(low, high) if axis == 0 => (parmin, parmax) = (low, high),
                AxisClip::Crossing(low, high) => {
                    (parmin, parmax) = (std_max(parmin, low), std_min(parmax, high))
                }
                AxisClip::Parallel(_) => {}
            }
            clips.push(clip);
        }
        self.nb_seg += 1;
        self.begin_on_curve[0] = parmin;
        self.end_on_curve[0] = parmax;
        let ranges: Vec<(f64, f64)> = slabs
            .iter()
            .zip(&clips)
            .map(|(s, c)| s.range(c, parmin, parmax))
            .collect();
        Some((
            std::array::from_fn(|i| ranges[i].0),
            std::array::from_fn(|i| ranges[i].1),
        ))
    }
}

fn slab(location: f64, direction: f64, bounds: (f64, f64), open: (bool, bool)) -> AxisSlab {
    AxisSlab {
        location,
        direction,
        min: bounds.0,
        max: bounds.1,
        open_min: open.0,
        open_max: open.1,
    }
}
