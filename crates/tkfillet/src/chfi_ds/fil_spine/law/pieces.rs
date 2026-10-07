//! The elementary laws of a fillet radius: the static `mklaw` of ChFiDS_FilSpine.cxx and
//! the constant pieces.
use crate::chfi_ds::errors::domain_error;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::law::{LawConstant, LawFunction, LawInterpol, LawS};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Point2, Xy};

/// A shared radius law.
pub(super) type LawHandle = Handle<dyn LawFunction>;

/// A `Law_Constant` of value `radius` on `[first, last]`.
pub(super) fn constant(radius: f64, first: f64, last: f64) -> LawHandle {
    let law = LawConstant::new();
    law.set(radius, first, last);
    Handle::new(law).cast()
}

/// The span and end radii of one `mklaw` call: `[start, end]`, with a negative radius
/// meaning "take it from the pairs".
#[derive(Clone, Copy)]
pub(super) struct Span {
    pub start: f64,
    pub end: f64,
    pub start_radius: f64,
    pub end_radius: f64,
}

/// The periodic frame of `mklaw`: whether the abscissae are brought into `[deb, fin]`.
#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub recadre: bool,
    pub deb: f64,
    pub fin: f64,
    pub tol3d: f64,
}

/// `mklaw(res, pr, curdeb, curfin, Rdeb, Rfin, recadre, deb, fin, tol3d)`: the laws of
/// the span from the pairs `pairs` it holds, appended to `out`.
pub(super) fn make_laws(
    out: &mut Vec<LawHandle>,
    pairs: &[Xy],
    span: Span,
    frame: Frame,
) -> Result<(), Failure> {
    let collected = collect_pairs(pairs, span, frame);
    if collected.points.is_empty() {
        out.push(law_without_pairs(span)?);
        return Ok(());
    }
    let mut points = collected.points;
    if !collected.on_start && span.start_radius >= 0. {
        points.push(Xy::new(span.start, span.start_radius));
    }
    if !collected.on_end && span.end_radius >= 0. {
        points.push(Xy::new(span.end, span.end_radius));
    }
    sort_by_abscissa(&mut points);
    remove_duplicates(&mut points, frame.tol3d);
    if collected.rad < 0. {
        out.push(constant(points[0].y(), span.start, points[0].x()));
    }
    if points.len() > 1 {
        out.push(interpolated(&points)?);
    }
    if collected.raf < 0. {
        let last = points[points.len() - 1];
        out.push(constant(last.y(), last.x(), span.end));
    }
    Ok(())
}

/// The pairs of the span (moved onto its ends when within `tol3d`), the end radii read
/// from them, and whether a pair fell on each end.
struct Collected {
    points: Vec<Xy>,
    rad: f64,
    raf: f64,
    on_start: bool,
    on_end: bool,
}

fn collect_pairs(pairs: &[Xy], span: Span, frame: Frame) -> Collected {
    let mut out = Collected {
        points: Vec::new(),
        rad: span.start_radius,
        raf: span.end_radius,
        on_start: false,
        on_end: false,
    };
    for pair in pairs {
        let mut wcur = pair.x();
        if frame.recadre {
            wcur = elclib::in_period(wcur, frame.deb, frame.fin);
        }
        if !(span.start - frame.tol3d <= wcur && wcur <= span.end + frame.tol3d) {
            continue;
        }
        let placed = place_pair(&mut out, *pair, wcur, span, frame.tol3d);
        out.points.push(placed);
    }
    out
}

/// One pair of the span, moved onto an end when it is within `tol3d` of it.
fn place_pair(out: &mut Collected, pair: Xy, wcur: f64, span: Span, tol3d: f64) -> Xy {
    if wcur - span.start < tol3d {
        out.on_start = true;
        if span.start_radius < 0. {
            out.rad = pair.y();
        }
        return Xy::new(span.start, out.rad);
    }
    if span.end - wcur < tol3d {
        out.on_end = true;
        if span.end_radius < 0. {
            out.raf = pair.y();
        }
        return Xy::new(span.end, out.raf);
    }
    Xy::new(wcur, pair.y())
}

/// The law of a span that holds no pair: Standard_DomainError when both end radii are
/// unknown, a constant law when one is, an S law between them otherwise.
fn law_without_pairs(span: Span) -> Result<LawHandle, Failure> {
    if span.start_radius < 0. && span.end_radius < 0. {
        return Err(domain_error("Impossible to create the law"));
    }
    if span.start_radius < 0. || span.end_radius < 0. {
        let r = if span.end_radius < 0. {
            span.start_radius
        } else {
            span.end_radius
        };
        return Ok(constant(r, span.start, span.end));
    }
    let law = LawS::new();
    law.set(span.start, span.start_radius, span.end, span.end_radius)?;
    Ok(Handle::new(law).cast())
}

/// The native exchange sort: each pair swapped with every later smaller abscissa.
fn sort_by_abscissa(points: &mut [Xy]) {
    let count = points.len();
    for i in 0..count.saturating_sub(1) {
        for j in i + 1..count {
            if points[i].x() > points[j].x() {
                points.swap(i, j);
            }
        }
    }
}

/// Removes the earlier of two consecutive pairs closer than `tol3d`.
fn remove_duplicates(points: &mut Vec<Xy>, tol3d: f64) {
    let mut count = points.len();
    let mut finished = count <= 1;
    let mut i = 0;
    while !finished {
        if (points[i].x() - points[i + 1].x()).abs() < tol3d {
            points.remove(i);
            count -= 1;
        } else {
            i += 1;
        }
        finished = i + 1 >= count;
    }
}

/// A `Law_Interpol` through `points`, with zero end tangents, not periodic.
fn interpolated(points: &[Xy]) -> Result<LawHandle, Failure> {
    let mut buffer: Vec<Point2> = points
        .iter()
        .map(|xy| Point2::new(xy.x(), xy.y()))
        .collect();
    let tpr = Array1::from_slice(&mut buffer);
    let law = LawInterpol::new();
    law.set_with_tangents(&tpr, 0., 0., false)?;
    Ok(Handle::new(law).cast())
}

/// A periodic `Law_Interpol` through `points` (`Set(pr, true)`).
pub(super) fn periodic_interpolated(points: &[Xy]) -> Result<LawHandle, Failure> {
    let mut buffer: Vec<Point2> = points
        .iter()
        .map(|xy| Point2::new(xy.x(), xy.y()))
        .collect();
    let pr = Array1::from_slice(&mut buffer);
    let law = LawInterpol::new();
    law.set(&pr, true)?;
    Ok(Handle::new(law).cast())
}
