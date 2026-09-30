// Rust translation of the analytic and conic branches of IntPatch_LineConstructor::Perform
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_LineConstructor.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_LineConstructor::Perform` on the analytic and the conic lines: each part
//! between two vertices whose middle point is inside both domains becomes a line.
use super::add_line::add_line;
use super::helpers::{compute_parametric_tolerance, local_parameter, parameters, recadre};
use super::wline_rline::Domains;
use crate::int_patch::{IntPatchALine, IntPatchGLine, IntPatchIType, IntPatchLine};
use crate::int_start::TheTopolTool;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Point2, Point3};
use std::f64::consts::PI;

/// The line under construction: the input line, the line number of each vertex
/// (`TabIndex`) and the lines built.
pub(super) struct Split<'a> {
    pub(super) l: &'a Handle<dyn IntPatchLine>,
    pub(super) tab_index: Vec<i32>,
    pub(super) numline: i32,
    pub(super) slin: &'a mut Vec<Handle<dyn IntPatchLine>>,
}

impl Split<'_> {
    /// The part between the vertices `i` and `j` becomes line number `numline + 1`.
    pub(super) fn add_part(&mut self, i: i32, j: i32) -> Result<(), Failure> {
        self.numline += 1;
        self.tab_index[i as usize] = self.numline;
        self.tab_index[j as usize] = self.numline;
        add_line(self.l, (i, j), &mut self.tab_index, self.slin)
    }

    /// The part between the vertices `i` and `j` becomes a line, without a line number
    /// (the restriction branch leaves `TabIndex` alone).
    pub(super) fn add_part_untagged(&mut self, i: i32, j: i32) -> Result<(), Failure> {
        add_line(self.l, (i, j), &mut self.tab_index, self.slin)
    }

    /// The vertices on no line that are on both domains (the external edge-edge
    /// interferences) become lines of one vertex each.
    pub(super) fn add_isolated_vertices(
        &mut self,
        nbvtx: i32,
        on_both: impl Fn(i32) -> Result<bool, Failure>,
    ) -> Result<(), Failure> {
        for i in 1..=nbvtx {
            if self.tab_index[i as usize] == 0 && on_both(i)? {
                self.numline += 1;
                self.tab_index[i as usize] = self.numline;
                add_line(self.l, (i, i), &mut self.tab_index, self.slin)?;
            }
        }
        Ok(())
    }
}

/// The reframed parameters of the 3d point on both quadrics.
fn quadric_parameters(domains: &Domains<'_>, p: Point3) -> Result<[f64; 4], Failure> {
    let mut uv: [f64; 4] = parameters(domains.s1, domains.s2, p)?;
    recadre(domains.s1, domains.s2, &mut uv)?;
    Ok(uv)
}

/// The states of the parameters in both domains at the tolerance `tol` (the second
/// domain is not classified when the first answers OUT).
fn classify_both(domains: &Domains<'_>, uv: [f64; 4], tol: f64) -> Result<(State, State), Failure> {
    let in1: State = domains.d1.classify(Point2::new(uv[0], uv[1]), tol, false)?;
    if in1 == State::Out {
        return Ok((in1, State::Out));
    }
    Ok((
        in1,
        domains.d2.classify(Point2::new(uv[2], uv[3]), tol, false)?,
    ))
}

/// `Perform` on an analytic line.
pub(super) fn perform_aline(split: &mut Split<'_>, domains: &Domains<'_>) -> Result<(), Failure> {
    let tol: f64 = precision::p_confusion() * 100.0;
    let aline: Handle<IntPatchALine> = split.l.cast::<IntPatchALine>();
    let nbvtx: i32 = aline.nb_vertex();
    for i in 1..nbvtx {
        let firstp: f64 = aline.vertex(i)?.parameter_on_line();
        let lastp: f64 = aline.vertex(i + 1)?.parameter_on_line();
        if firstp == lastp {
            continue;
        }
        let pmid: f64 = (firstp + lastp) * 0.5;
        let uv: [f64; 4] = quadric_parameters(domains, aline.value(pmid)?)?;
        let (in1, in2) = classify_both(domains, uv, tol)?;
        if !(in1 == State::Out || in2 == State::Out) {
            split.add_part(i, i + 1)?;
        }
    }
    split.add_isolated_vertices(nbvtx, |i| {
        let v = aline.vertex(i)?;
        Ok(v.is_on_dom_s1() && v.is_on_dom_s2())
    })
}

/// The point of the conic line at `u` (`ElCLib::Value`).
fn conic_value(gline: &IntPatchGLine, typ: IntPatchIType, u: f64) -> Result<Point3, Failure> {
    Ok(match typ {
        IntPatchIType::Lin => elclib::value(u, &gline.line()?),
        IntPatchIType::Circle => elclib::value(u, &gline.circle()?),
        IntPatchIType::Ellipse => elclib::value(u, &gline.ellipse()?),
        IntPatchIType::Hyperbola => elclib::value(u, &gline.hyperbola()?),
        IntPatchIType::Parabola => elclib::value(u, &gline.parabola()?),
        _ => Point3::new(0.0, 0.0, 0.0),
    })
}

/// The tolerance of the first arc of the domain (after `Init`, when `more` answers
/// that there is one) at `(u, v)` of the surface, else `tol` unchanged.
fn arc_tolerance(
    (s, d): (&Adaptor3dSurfaceHandle, &dyn TheTopolTool),
    more: bool,
    (u, v): (f64, f64),
    tol: f64,
) -> Result<f64, Failure> {
    if !more {
        return Ok(tol);
    }
    let d1 = s.d1(u, v)?;
    Ok(compute_parametric_tolerance(
        d.tol_3d_arc(&d.value()?)?,
        d1.d1u,
        d1.d1v,
    ))
}

/// An UNKNOWN state counts as OUT (the section with an infinite prism).
fn known(state: State) -> State {
    if state == State::Unknown {
        State::Out
    } else {
        state
    }
}

/// The conic part between two vertices: its middle point classified with the
/// tolerances of the first arcs of the domains (`Tol` persists across the parts).
fn classify_conic_part(
    domains: &Domains<'_>,
    uv: [f64; 4],
    tol: &mut f64,
) -> Result<bool, Failure> {
    domains.d1.init();
    // The native code tests the second domain after initializing the first one.
    *tol = arc_tolerance(domains.side(true), domains.d2.more(), (uv[0], uv[1]), *tol)?;
    let in1: State = domains
        .d1
        .classify(Point2::new(uv[0], uv[1]), *tol, false)?;
    domains.d2.init();
    let more2: bool = in1 != State::Out && domains.d2.more();
    *tol = arc_tolerance(domains.side(false), more2, (uv[2], uv[3]), *tol)?;
    let in2: State = match in1 {
        State::Out => State::Out,
        _ => domains
            .d2
            .classify(Point2::new(uv[2], uv[3]), *tol, false)?,
    };
    Ok(!(known(in1) == State::Out || known(in2) == State::Out))
}

/// The closing part of a circle or an ellipse, from its last vertex to its first one a
/// turn later, when its middle is within the bounds of the line.
// The negated comparisons send the NaN operands where the native ones do.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn closing_part(
    split: &mut Split<'_>,
    domains: &Domains<'_>,
    (gline, typ, nbvtx): (&IntPatchGLine, IntPatchIType, i32),
    tol: f64,
) -> Result<(), Failure> {
    let firstp: f64 = gline.vertex(nbvtx)?.parameter_on_line();
    let lastp: f64 = PI + PI + gline.vertex(1)?.parameter_on_line();
    let cadrinf: f64 = local_parameter(split.l, true)?;
    let cadrsup: f64 = local_parameter(split.l, false)?;
    let mut acadr: f64 = (firstp + lastp) * 0.5;
    while acadr < cadrinf {
        acadr += PI + PI;
    }
    while acadr > cadrsup {
        acadr -= PI + PI;
    }
    if !(acadr >= cadrinf && acadr <= cadrsup)
        || !((firstp - lastp).abs() > precision::p_confusion())
    {
        return Ok(());
    }
    let pmid: f64 = (firstp + lastp) * 0.5;
    let uv: [f64; 4] = quadric_parameters(domains, conic_value(gline, typ, pmid)?)?;
    let (in1, in2) = classify_both(domains, uv, tol)?;
    if !(known(in1) == State::Out || known(in2) == State::Out) {
        split.add_part(nbvtx, 1)?;
    }
    Ok(())
}

/// `Perform` on a conic line.
// The negated comparison sends the NaN operands where the native one does.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
pub(super) fn perform_gline(split: &mut Split<'_>, domains: &Domains<'_>) -> Result<(), Failure> {
    let mut tol: f64 = precision::p_confusion() * 100.0;
    let typ: IntPatchIType = split.l.arc_type();
    let gline: Handle<IntPatchGLine> = split.l.cast::<IntPatchGLine>();
    let nbvtx: i32 = gline.nb_vertex();
    for i in 1..nbvtx {
        let firstp: f64 = gline.vertex(i)?.parameter_on_line();
        let lastp: f64 = gline.vertex(i + 1)?.parameter_on_line();
        if !((firstp - lastp).abs() > precision::p_confusion()) {
            continue;
        }
        let pmid: f64 = (firstp + lastp) * 0.5;
        let uv: [f64; 4] = quadric_parameters(domains, conic_value(&gline, typ, pmid)?)?;
        if classify_conic_part(domains, uv, &mut tol)? {
            split.add_part(i, i + 1)?;
        }
    }
    if matches!(typ, IntPatchIType::Circle | IntPatchIType::Ellipse) {
        closing_part(split, domains, (&gline, typ, nbvtx), tol)?;
    }
    split.add_isolated_vertices(nbvtx, |i| {
        let v = gline.vertex(i)?;
        Ok(v.is_on_dom_s1() && v.is_on_dom_s2())
    })
}
