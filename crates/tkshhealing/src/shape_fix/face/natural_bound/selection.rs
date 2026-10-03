use super::intervals::{cut_interval, find_best_interval};
use crate::shape_analysis::{ShapeAnalysis, ShapeAnalysisSurface, adjust_by_period};
use rustycad_tkbrep::topods::{self, Builder, Face, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};

pub(super) fn wire_translations(
    face: &Face,
    wires: &[Wire],
    surface: &ShapeAnalysisSurface,
) -> Result<(Point2, Vec<Vector2>), Failure> {
    let (u_min, u_max, v_min, v_max) = surface.bounds();
    let (u_closed, v_closed) = closure_flags(surface)?;
    let mut ranges = FreeRanges::new((u_min, u_max), (v_min, v_max));
    let mut centers = Vec::with_capacity(wires.len());
    cut_wire_bounds(face, wires, &mut ranges, &mut centers, u_closed, v_closed)?;
    let shift = best_open_center(&ranges, u_closed, v_closed);
    let target = target_center(shift, (u_min, u_max), (v_min, v_max), u_closed, v_closed);
    let translations = centers
        .iter()
        .map(|center| {
            center_shift(
                *center,
                target,
                (u_closed, v_closed),
                (u_max - u_min, v_max - v_min),
            )
        })
        .collect();
    Ok((shift, translations))
}

struct FreeRanges {
    u: Vec<(f64, f64)>,
    v: Vec<(f64, f64)>,
    u_period: f64,
    v_period: f64,
}

impl FreeRanges {
    fn new(u: (f64, f64), v: (f64, f64)) -> Self {
        Self {
            u: vec![u],
            v: vec![v],
            u_period: u.1 - u.0,
            v_period: v.1 - v.0,
        }
    }
}

fn closure_flags(surface: &ShapeAnalysisSurface) -> Result<(bool, bool), Failure> {
    Ok((surface.is_u_closed(-1.0)?, surface.is_v_closed(-1.0)?))
}

fn cut_wire_bounds(
    face: &Face,
    wires: &[Wire],
    ranges: &mut FreeRanges,
    centers: &mut Vec<Point2>,
    u_closed: bool,
    v_closed: bool,
) -> Result<(), Failure> {
    for wire in wires {
        let (u_min, u_max, v_min, v_max) = wire_face_bounds(face, wire)?;
        if u_closed {
            cut_interval(&mut ranges.u, (u_min, u_max), ranges.u_period);
        }
        if v_closed {
            cut_interval(&mut ranges.v, (v_min, v_max), ranges.v_period);
        }
        centers.push(Point2::new((u_min + u_max) * 0.5, (v_min + v_max) * 0.5));
    }
    Ok(())
}

fn wire_face_bounds(face: &Face, wire: &Wire) -> Result<(f64, f64, f64, f64), Failure> {
    let mut wire_shape = face.empty_copied()?;
    Builder.add(&mut wire_shape, &wire.clone().into_shape())?;
    ShapeAnalysis::get_face_uv_bounds(&topods::face(&wire_shape)?)
}

fn best_open_center(ranges: &FreeRanges, u_closed: bool, v_closed: bool) -> Point2 {
    Point2::new(
        if u_closed {
            find_best_interval(&ranges.u)
        } else {
            0.0
        },
        if v_closed {
            find_best_interval(&ranges.v)
        } else {
            0.0
        },
    )
}

fn target_center(
    shift: Point2,
    u_bounds: (f64, f64),
    v_bounds: (f64, f64),
    u_closed: bool,
    v_closed: bool,
) -> Point2 {
    let u_midpoint = if u_closed {
        (u_bounds.1 - u_bounds.0) * 0.5
    } else {
        0.0
    };
    let v_midpoint = if v_closed {
        (v_bounds.1 - v_bounds.0) * 0.5
    } else {
        0.0
    };
    Point2::new(shift.x() + u_midpoint, shift.y() + v_midpoint)
}

fn center_shift(
    center: Point2,
    target: Point2,
    closed: (bool, bool),
    periods: (f64, f64),
) -> Vector2 {
    let u = if closed.0 {
        adjust_by_period(center.x(), target.x(), periods.0)
    } else {
        0.0
    };
    let v = if closed.1 {
        adjust_by_period(center.y(), target.y(), periods.1)
    } else {
        0.0
    };
    Vector2::new(u, v)
}
