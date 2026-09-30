// Rust translation of IntPatch_WLine::Dump and IntPatch_RLine::Dump (IntPatch_WLine.cxx,
// IntPatch_RLine.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Dump(theMode)` of the walking and restriction lines, whose bodies are the same
//! but for the class letter of the banner.
use super::dump::{ostream_real, put, signed_fixed};
use super::point_line::IntPatchPointLine;
use rustycad_tkernel::standard::Failure;
use std::fmt::Write;

/// `Dump(theMode)` of `IntPatch_<letter>Line`.
pub(super) fn dump_point_line(
    line: &dyn IntPatchPointLine,
    letter: &str,
    mode: i32,
    out: &mut dyn Write,
) -> Result<(), Failure> {
    let banner: String =
        format!(" ----------- D u m p    I n t P a t c h  _  {letter} L i n e  -(begin)------\n");
    put(out, &banner)?;
    let nb_points: i32 = line.nb_pnts()?;
    match mode {
        0 => dump_all(line, nb_points, out)?,
        1 => dump_coordinates(line, nb_points, 3, out)?,
        2 => dump_coordinates(line, nb_points, 1, out)?,
        _ => dump_coordinates(line, nb_points, 2, out)?,
    }
    put(
        out,
        "\n--------------------------------------------------- (end) -------\n",
    )
}

/// Mode 0: every point with its parameters, then every vertex.
fn dump_all(
    line: &dyn IntPatchPointLine,
    nb_points: i32,
    out: &mut dyn Write,
) -> Result<(), Failure> {
    let nb_vertex: i32 = line.nb_vertex();
    put(out, "Num    [X  Y  Z]     [U1  V1]   [U2  V2]\n")?;
    for i in 1..=nb_points {
        let point = line.point(i)?;
        let (u1, v1, u2, v2) = point.parameters();
        let p = point.value();
        let [x, y, z, u1, v1, u2, v2] = [p.x(), p.y(), p.z(), u1, v1, u2, v2].map(signed_fixed);
        put(
            out,
            &format!("{i:4}  [{x} {y} {z}]  [{u1} {v1}]  [{u2} {v2}]\n"),
        )?;
    }
    for i in 1..=nb_vertex {
        let vertex = line.vertex(i)?;
        vertex.dump(out)?;
        let polr: f64 = vertex.parameter_on_line();
        // `static_cast<int>(polr)`: truncation toward zero.
        let pol: i32 = polr as i32;
        if pol >= 1 && pol <= nb_vertex {
            let p = line.vertex(pol)?.value();
            let [x, y, z] = [p.x(), p.y(), p.z()].map(ostream_real);
            let polr: String = ostream_real(polr);
            put(
                out,
                &format!("----> IntSurf_PntOn2S : {polr}, Pnt ({x},{y},{z})\n"),
            )?;
        }
    }
    Ok(())
}

/// Modes 1 to 3: the 3d points (`what` 3) or the points on the first (1) or the second
/// (2) surface.
fn dump_coordinates(
    line: &dyn IntPatchPointLine,
    nb_points: i32,
    what: i32,
    out: &mut dyn Write,
) -> Result<(), Failure> {
    for i in 1..=nb_points {
        let point = line.point(i)?;
        let (u1, v1, u2, v2) = point.parameters();
        let p = point.value();
        let coordinates: Vec<f64> = match what {
            3 => vec![p.x(), p.y(), p.z()],
            1 => vec![u1, v1],
            _ => vec![u2, v2],
        };
        let text: Vec<String> = coordinates.into_iter().map(signed_fixed).collect();
        put(out, &format!("point p{i} {}\n", text.join(" ")))?;
    }
    Ok(())
}
