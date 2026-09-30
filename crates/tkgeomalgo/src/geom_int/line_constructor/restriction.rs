//! `Perform` on a restriction line: the vertices sorted by parameter with the
//! orientations of the line on both domains, and the parts where the line is inside
//! both.
use super::GeomIntLineConstructor;
use crate::geom_int::line_tool::GeomIntLineTool;
use crate::geom_int::parameter_and_orientation::GeomIntParameterAndOrientation;
use crate::int_patch::{IntPatchLine, IntPatchPoint};
use crate::int_surf::{IntSurfTransition, IntSurfTypeTrans};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::gp::Point2;

/// The orientation of the line on a domain from its transition on the arc: FORWARD
/// going in, REVERSED going out, INTERNAL otherwise (and off the domain).
fn orientation(
    on_dom: bool,
    transition: impl FnOnce() -> Result<IntSurfTransition, Failure>,
) -> Result<Orientation, Failure> {
    if !on_dom {
        return Ok(Orientation::Internal);
    }
    Ok(match transition()?.transition_type() {
        IntSurfTypeTrans::In => Orientation::Forward,
        IntSurfTypeTrans::Out => Orientation::Reversed,
        IntSurfTypeTrans::Touch | IntSurfTypeTrans::Undecided => Orientation::Internal,
    })
}

/// The accumulation of an orientation into a vertex already met at the same parameter:
/// a different known orientation makes it INTERNAL.
fn accumulate(current: Orientation, or: Orientation) -> Orientation {
    if or == Orientation::Internal {
        return current;
    }
    if current == Orientation::Internal {
        return or;
    }
    if or != current {
        Orientation::Internal
    } else {
        current
    }
}

/// Inserts the vertex of parameter `prm` into the sorted sequence, merged with a vertex
/// within `tol`.
fn insert_vertex(
    seqpss: &mut Vec<GeomIntParameterAndOrientation>,
    (prm, or1, or2): (f64, Orientation, Orientation),
    tol: f64,
) {
    for j in 0..seqpss.len() {
        let valj: &mut GeomIntParameterAndOrientation = &mut seqpss[j];
        if (prm - valj.parameter()).abs() <= tol {
            // Accumulate.
            valj.set_orientation1(accumulate(valj.orientation1(), or1));
            valj.set_orientation2(accumulate(valj.orientation2(), or2));
            return;
        }
        if prm < valj.parameter() - tol {
            seqpss.insert(
                j,
                GeomIntParameterAndOrientation::with_values(prm, or1, or2),
            );
            return;
        }
    }
    seqpss.push(GeomIntParameterAndOrientation::with_values(prm, or1, or2));
}

/// Whether the line is in the domain at its start: None when the first oriented vertex
/// does not decide (the line is kept in doubt after a classification), else the side.
fn start_side(seqpss: &[GeomIntParameterAndOrientation], second: bool) -> Option<bool> {
    seqpss.iter().find_map(|v| {
        let or: Orientation = if second {
            v.orientation2()
        } else {
            v.orientation1()
        };
        (or != Orientation::Internal).then_some(or != Orientation::Forward)
    })
}

impl GeomIntLineConstructor {
    /// `Perform` on a restriction line.
    pub(super) fn perform_rline(
        &mut self,
        l: &Handle<dyn IntPatchLine>,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        self.seqp.clear();
        let nbvtx: i32 = GeomIntLineTool::nb_vertex(l)?;
        if nbvtx == 0 {
            // Kept a priori: a 2d point on each surface is needed, which the caller does.
            self.push_part(
                GeomIntLineTool::first_parameter(l)?,
                GeomIntLineTool::last_parameter(l)?,
            );
            self.done = true;
            return Ok(());
        }
        let mut seqpss: Vec<GeomIntParameterAndOrientation> = Vec::new();
        for i in 1..=nbvtx {
            let v: IntPatchPoint = GeomIntLineTool::vertex(l, i)?;
            let or1 = orientation(v.is_on_dom_s1(), || v.transition_line_arc1())?;
            let or2 = orientation(v.is_on_dom_s2(), || v.transition_line_arc2())?;
            insert_vertex(&mut seqpss, (v.parameter_on_line(), or1, or2), tol);
        }
        // The state at the beginning of the line.
        let (trim1, dans_s1) = match start_side(&seqpss, false) {
            Some(side) => (true, side),
            None => {
                if self.first_free_vertex_out(l, 1, tol)? {
                    self.done = true;
                    return Ok(());
                }
                // Kept in doubt.
                (false, true)
            }
        };
        let (trim2, dans_s2) = match start_side(&seqpss, true) {
            Some(side) => (true, side),
            None => {
                if self.first_free_vertex_out(l, 2, tol)? {
                    self.done = true;
                    return Ok(());
                }
                (false, true)
            }
        };
        if !(trim1 || trim2) {
            // Necessarily in both domains.
            self.push_part(
                GeomIntLineTool::first_parameter(l)?,
                GeomIntLineTool::last_parameter(l)?,
            );
            self.done = true;
            return Ok(());
        }
        self.peel(l, &seqpss, (dans_s1, dans_s2))?;
        self.done = true;
        Ok(())
    }

    /// Whether the first vertex off the domain `k` is OUT of it.
    fn first_free_vertex_out(
        &self,
        l: &Handle<dyn IntPatchLine>,
        k: i32,
        tol: f64,
    ) -> Result<bool, Failure> {
        for i in 1..=GeomIntLineTool::nb_vertex(l)? {
            let v: IntPatchPoint = GeomIntLineTool::vertex(l, i)?;
            let on_dom: bool = if k == 1 {
                v.is_on_dom_s1()
            } else {
                v.is_on_dom_s2()
            };
            if on_dom {
                continue;
            }
            let (u, vv) = if k == 1 {
                v.parameters_on_s1()
            } else {
                v.parameters_on_s2()
            };
            return Ok(self.dom(k)?.classify(Point2::new(u, vv), tol, true)? == State::Out);
        }
        Ok(false)
    }

    /// The sequence of vertices peeled into the parts where the line is in both domains.
    fn peel(
        &mut self,
        l: &Handle<dyn IntPatchLine>,
        seqpss: &[GeomIntParameterAndOrientation],
        (mut dans_s1, mut dans_s2): (bool, bool),
    ) -> Result<(), Failure> {
        let thefirst: f64 = GeomIntLineTool::first_parameter(l)?;
        let thelast: f64 = GeomIntLineTool::last_parameter(l)?;
        let mut firstp: f64 = thefirst;
        for v in seqpss {
            let (or1, or2) = (v.orientation1(), v.orientation2());
            if dans_s1 && dans_s2 {
                dans_s1 = dans_s1 && or1 != Orientation::Reversed;
                dans_s2 = dans_s2 && or2 != Orientation::Reversed;
                if !dans_s1 || !dans_s2 {
                    let lastp: f64 = v.parameter();
                    let stofirst: f64 = std_max(firstp, thefirst);
                    let stolast: f64 = std_min(lastp, thelast);
                    if stolast > stofirst {
                        self.push_part(stofirst, stolast);
                    }
                    if lastp > thelast {
                        break;
                    }
                }
                continue;
            }
            dans_s1 = if dans_s1 {
                or1 != Orientation::Reversed
            } else {
                or1 == Orientation::Forward
            };
            dans_s2 = if dans_s2 {
                or2 != Orientation::Reversed
            } else {
                or2 == Orientation::Forward
            };
            if dans_s1 && dans_s2 {
                firstp = v.parameter();
            }
        }
        // Finally, the end.
        if dans_s1 && dans_s2 {
            let lastp: f64 = thelast;
            firstp = std_max(firstp, thefirst);
            if lastp > firstp {
                self.push_part(firstp, lastp);
            }
        }
        Ok(())
    }
}
