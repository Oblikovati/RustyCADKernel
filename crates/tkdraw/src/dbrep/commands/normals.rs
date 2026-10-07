// Rust translation of the `normals` command of Draw/TKDraw/DBRep/DBRep.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `normals`: the face normals of a shape as displayed segments.
use super::CommandEntry;
use crate::dbrep::drawable_shape::{DBRepDrawableShape, MeshNormals};
use crate::dbrep::{color_orientation, write_color_orientation};
use crate::draw::{DrawInterpretor, DrawSegment3D, DrawSession, DrawableHandle};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{self, Vector3};
use std::rc::Rc;

/// `normals`.
pub(super) const NORMALS: CommandEntry = (
    "normals",
    "normals shape [Length {10}] [-NbAlongU {1}] [-NbAlongV {1}] [-UseMesh] [-print], display normals",
    normals,
);

/// The options of `normals`.
struct NormalsRequest {
    use_mesh: bool,
    length: f64,
    nb_u: i32,
    nb_v: i32,
    print: bool,
}

/// `normals shape [length] [-length l] [-nbu n] [-nbv n] [-nbuv n] [-usemesh] [-print]`:
/// displays the normals (colored by face orientation) and prints them with `-print`.
///
/// Deviation: the C++ map of faces iterates in hash order (object addresses); the
/// faces are taken in exploration order here.
fn normals(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let out = session.console();
    if a.len() < 2 {
        out.write_out("Syntax error: wrong number of arguments!\n");
        di.print_help(a[0])?;
        return Ok(1);
    }
    let shape = session.shape(a[1])?;
    if shape.is_null() {
        out.write_out(&format!("Error: shape with name {} is not found\n", a[1]));
        return Ok(1);
    }
    let Some(request) = parse(session, a)? else {
        return Ok(1);
    };
    write_color_orientation(out.as_ref());
    let mut normals = MeshNormals::new();
    if request.use_mesh {
        DBRepDrawableShape::add_shape_mesh_normals(
            &mut normals,
            &shape,
            request.length,
            out.as_ref(),
        )?;
    } else {
        let counts = (request.nb_u, request.nb_v);
        DBRepDrawableShape::add_shape_surface_normals(
            &mut normals,
            &shape,
            request.length,
            counts,
            out.as_ref(),
        )?;
    }
    for (face, segments) in &normals {
        let orientation = face.orientation();
        let color = color_orientation(orientation);
        for (from, to) in segments {
            let segment: DrawableHandle = Rc::new(DrawSegment3D::new(*from, *to, color));
            session.viewer().display(&segment)?;
            if request.print {
                print_normal(
                    di,
                    Vector3::between(*from, *to),
                    orientation == Orientation::Reversed,
                )?;
            }
        }
    }
    Ok(0)
}

/// `(x, y, z)` of the normal (reversed for a REVERSED face), with `-0` printed as 0.
fn print_normal(di: &DrawInterpretor, mut normal: Vector3, reverse: bool) -> Result<(), Failure> {
    if reverse {
        normal.reverse();
    }
    let clean = |v: f64| if v == 0. { 0. } else { v };
    di.append("(")?
        .append(clean(normal.x()))?
        .append(", ")?
        .append(clean(normal.y()))?
        .append(", ")?
        .append(clean(normal.z()))?
        .append(")\n")?;
    Ok(())
}

/// Parses the options; `None` after a reported syntax error.
fn parse(session: &DrawSession, a: &[&str]) -> Result<Option<NormalsRequest>, Failure> {
    let out = session.console();
    let mut request = NormalsRequest {
        use_mesh: false,
        length: 10.0,
        nb_u: 1,
        nb_v: 1,
        print: false,
    };
    let mut index = 2;
    while index < a.len() {
        let text = a[index].to_ascii_lowercase();
        let parameter = AsciiString::new(&text);
        let value = |index: usize| a.get(index).copied();
        let error = match text.as_str() {
            _ if index == 2 && parameter.is_real_value(false) => {
                request.length = parameter.real_value()?;
                zero_length(request.length)
            }
            "-usemesh" | "-mesh" => {
                request.use_mesh = true;
                None
            }
            "-length" | "-len" => {
                index += 1;
                request.length = value(index).map_or(0.0, |v| session.atof(v));
                zero_length(request.length)
            }
            "-nbalongu" | "-nbu" => {
                index += 1;
                request.nb_u = value(index).map_or(0, |v| session.atoi(v));
                (request.nb_u < 1).then_some("Syntax error: NbAlongU should be >=1\n".to_string())
            }
            "-nbalongv" | "-nbv" => {
                index += 1;
                request.nb_v = value(index).map_or(0, |v| session.atoi(v));
                (request.nb_v < 1).then_some("Syntax error: NbAlongV should be >=1\n".to_string())
            }
            "-nbalong" | "-nbuv" => {
                index += 1;
                request.nb_u = value(index).map_or(0, |v| session.atoi(v));
                request.nb_v = request.nb_u;
                (request.nb_u < 1).then_some("Syntax error: NbAlong should be >=1\n".to_string())
            }
            "-print" => {
                request.print = true;
                None
            }
            _ => Some(format!("Syntax error: unknown argument '{text}'\n")),
        };
        if let Some(message) = error {
            out.write_out(&message);
            return Ok(None);
        }
        index += 1;
    }
    Ok(Some(request))
}

/// The zero-length error, when `length` is within `gp::Resolution()` of 0.
fn zero_length(length: f64) -> Option<String> {
    (length.abs() <= gp::resolution())
        .then_some("Syntax error: length should not be zero\n".to_string())
}
