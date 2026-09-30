// Rust translation of IntPatch_Point::Dump and its static DumpTransition
// (IntPatch_Point.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_Point::Dump`: the native `printf`/`std::cout` text, written in program order.
use super::dump::{ostream_real, pointer, put, signed_fixed};
use super::point::{IntPatchPoint, PointOnDomain};
use crate::int_surf::{IntSurfSituation, IntSurfTransition, IntSurfTypeTrans};
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;

/// The static `DumpTransition(T)`.
fn dump_transition(t: &IntSurfTransition, out: &mut dyn Write) -> Result<(), Failure> {
    let kind: IntSurfTypeTrans = t.transition_type();
    match kind {
        IntSurfTypeTrans::In => put(out, " In  ")?,
        IntSurfTypeTrans::Out => put(out, " Out ")?,
        IntSurfTypeTrans::Undecided => put(out, " Und.")?,
        IntSurfTypeTrans::Touch => dump_touch(t, out)?,
    }
    if kind == IntSurfTypeTrans::Undecided {
        return Ok(());
    }
    let tangent: bool = t.is_tangent()?;
    put(
        out,
        if tangent {
            " Tangent    "
        } else {
            " Non_Tangent"
        },
    )
}

fn dump_touch(t: &IntSurfTransition, out: &mut dyn Write) -> Result<(), Failure> {
    put(out, " Touch ")?;
    put(
        out,
        match t.situation()? {
            IntSurfSituation::Inside => " Inside ",
            IntSurfSituation::Outside => " Outside",
            IntSurfSituation::Unknown => " Unknown",
        },
    )?;
    put(
        out,
        if t.is_opposite()? {
            "     Opposite "
        } else {
            " Non_Opposite "
        },
    )
}

/// The `*OnS1*`/`*Vtx1*` line of one domain (`index` 1 or 2).
fn dump_domain(d: &PointOnDomain, index: i32, out: &mut dyn Write) -> Result<(), Failure> {
    if d.on {
        let par: String = signed_fixed(d.prm);
        let arc: String = pointer(d.arc.object_address());
        put(out, &format!("*OnS{index}*  par={par} arc{index}={arc}"))?;
    }
    if d.vtx_on {
        let vtx: String = pointer(d.vertex.object_address());
        put(out, &format!(" *Vtx{index}*  vtx{index}={vtx}"))?;
    }
    if d.on || d.vtx_on {
        put(out, "\n")?;
    }
    Ok(())
}

fn dump_transitions(d: &PointOnDomain, index: i32, out: &mut dyn Write) -> Result<(), Failure> {
    put(out, &format!("\n-traline{index}  : "))?;
    dump_transition(&d.traline, out)?;
    put(out, &format!("  -tra{index}  : "))?;
    dump_transition(&d.tra, out)
}

impl IntPatchPoint {
    /// `Dump()`: the native text (the point, its parameters, its domains and
    /// transitions, its tolerance). The arc and vertex addresses are those of the Rust
    /// objects. `p.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        put(out, "----------- IntPatch_Point : \n")?;
        let (u1, v1, u2, v2) = self.parameters();
        let p = self.value();
        let [x, y, z] = [p.x(), p.y(), p.z()].map(signed_fixed);
        let [u1, v1, u2, v2, para] = [u1, v1, u2, v2, self.parameter_on_line()].map(signed_fixed);
        put(
            out,
            &format!("P({x},{y},{z}) UV1({u1},{v1})  UV2({u2},{v2}) (Para:{para})\n"),
        )?;
        dump_domain(&self.s1, 1, out)?;
        dump_domain(&self.s2, 2, out)?;
        if self.s1.on || self.s2.on {
            let (tgt, mult) = (
                i32::from(self.is_tangency_point()),
                i32::from(self.is_multiple()),
            );
            put(out, &format!(" tgt:{tgt}   mult:{mult}"))?;
            if self.s1.on {
                dump_transitions(&self.s1, 1, out)?;
            }
            if self.s2.on {
                dump_transitions(&self.s2, 2, out)?;
            }
            put(out, "\n")?;
        }
        put(
            out,
            &format!("\nToler = {}\n", ostream_real(self.tolerance())),
        )
    }
}
