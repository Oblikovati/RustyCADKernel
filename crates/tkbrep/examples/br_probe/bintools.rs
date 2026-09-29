//! BinTools probe cases, part 1: the Rust twin of tools/oracle/tkbrep_probe_bintools.cxx
//! (the shared printers, the scalar encodings and the stream primitives).
use super::brep::{point, point2d};
use super::common::{Reader, end, flag, int, long, real, text, word, written_bytes};
use rustycad_tkbrep::bintools::{self, BinIStream, BinOStream, ObjectType};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis1, Direction2, Direction3, Point3, Transform3};
use rustycad_tkmath::poly::Triangle;

/// `bt_bytes`: the hex text, or `h <length> <fnv1a64>` for a long string.
pub fn bytes_token(bytes: &[u8]) {
    if bytes.len() <= 4096 {
        text(bytes);
        return;
    }
    let hash = bytes.iter().fold(1469598103934665603_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(1099511628211)
    });
    word("h");
    int(bytes.len() as i32);
    long(hash);
}

/// `bt_mask_line`: `Type: text` keeps `Type` for another exception class.
fn mask_line(line: &str) -> String {
    let Some(colon) = line.find(": ") else {
        return line.to_string();
    };
    let prefix = &line[..colon];
    let name = !prefix.is_empty()
        && prefix.contains('_')
        && prefix
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_');
    if !name || prefix == "Standard_Failure" {
        return line.to_string();
    }
    prefix.to_string()
}

fn masked_message(message: &str) -> String {
    message
        .split('\n')
        .map(mask_line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// `bt_ascii`: every character from U+0080 on becomes `?`.
fn ascii(message: &str) -> Vec<u8> {
    message
        .chars()
        .map(|c| if c.is_ascii() { c as u8 } else { b'?' })
        .collect()
}

/// `bt_failure_parts`.
pub fn failure_parts(exception_type: &str, message: &str) {
    word(exception_type);
    if exception_type == "Standard_Failure" {
        text(&ascii(&masked_message(message)));
    }
}

/// `bt_failure`.
pub fn failure(error: &Failure) {
    failure_parts(error.exception_type(), error.message());
}

/// `BT_TRY`.
pub fn attempt(block: impl FnOnce() -> Result<(), Failure>) {
    if let Err(error) = block() {
        failure(&error);
    }
}

/// `bt_read_bytes`: `zeros` zero bytes, then the given bytes.
pub fn read_bytes(r: &mut Reader<'_>) -> Vec<u8> {
    let zeros = r.integer().max(0) as usize;
    let mut bytes = vec![0_u8; zeros];
    bytes.extend(r.bytes());
    bytes
}

/// A new writable in-memory stream (`std::ostringstream`).
pub fn out_stream() -> IoStream {
    IoStream::from_bytes(Vec::new())
}

/// `br_bin_scalars`.
pub fn scalars_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let values: Vec<f64> = (0..n).map(|_| r.real()).collect();
    let mut o = out_stream();
    for &v in &values {
        bintools::put_real(&mut o, v);
        bintools::put_short_real(&mut o, v as f32);
        bintools::put_integer(&mut o, v as i32);
        bintools::put_bool(&mut o, v != 0.0);
        bintools::put_ext_char(&mut o, (v as i32) as u16);
    }
    let bytes = written_bytes(o);
    bytes_token(&bytes);
    let mut is = IoStream::from_bytes(bytes);
    for _ in &values {
        let d = bintools::get_real(&mut is).unwrap();
        let f = bintools::get_short_real(&mut is).unwrap();
        let k = bintools::get_integer(&mut is).unwrap();
        let b = bintools::get_bool(&mut is);
        let c = bintools::get_ext_char(&mut is).unwrap();
        real(d);
        real(f64::from(f));
        int(k);
        flag(b);
        int(i32::from(c));
    }
    attempt(|| bintools::get_real(&mut is).map(|_| ()));
    flag(bintools::get_bool(&mut is));
    int(i32::from(is.rdstate()));
    attempt(|| bintools::get_short_real(&mut IoStream::from_bytes(b"abc".to_vec())).map(|_| ()));
    attempt(|| bintools::get_integer(&mut IoStream::from_bytes(b"ab".to_vec())).map(|_| ()));
    attempt(|| bintools::get_ext_char(&mut IoStream::from_bytes(b"a".to_vec())).map(|_| ()));
    end();
}

fn direction(r: &mut Reader<'_>) -> Result<Direction3, Failure> {
    let (x, y) = (r.real(), r.real());
    Direction3::new(x, y, r.real())
}

/// The case 12 transformation: translation * rotation * scale.
fn transform(r: &mut Reader<'_>) -> Result<Transform3, Failure> {
    let (x, y, z) = (r.real(), r.real(), r.real());
    let translation = Transform3::translation([x, y, z]);
    let axis = direction(r)?;
    let mut rotation = Transform3::default();
    rotation.set_rotation(Axis1::new(Point3::new(0.0, 0.0, 0.0), axis), r.real())?;
    let mut scale = Transform3::default();
    scale.set_scale(Point3::new(0.0, 0.0, 0.0), r.real())?;
    Ok(translation.multiplied(rotation).multiplied(scale))
}

fn orientation(i: i32) -> Orientation {
    Orientation::ALL[i as usize]
}

fn ostream_op(s: &mut BinOStream<'_>, r: &mut Reader<'_>, op: i32) -> Result<(), Failure> {
    match op {
        0 => s.write_reference(r.real() as u64),
        1 => {
            let t = ShapeEnum::ALL[r.integer() as usize];
            s.write_shape(t, orientation(r.integer()));
        }
        2 => s.write_object_type(ObjectType(r.integer() as u8)),
        3 => s.write_byte(r.integer() as u8),
        4 => s.write_real(r.real()),
        5 => s.write_bool(r.flag()),
        6 => s.write_integer(r.integer()),
        7 => s.write_ext_char(r.integer() as u16),
        8 => s.write_point3(point(r)),
        9 => s.write_direction3(direction(r)?),
        10 => s.write_point2(point2d(r)),
        11 => {
            let x = r.real();
            s.write_direction2(Direction2::new(x, r.real())?);
        }
        12 => s.write_transform(&transform(r)?),
        13 => {
            let (a, b) = (r.integer(), r.integer());
            s.write_triangle(&Triangle::new(a, b, r.integer()));
        }
        14 => {
            let (x, y) = (r.real() as f32, r.real() as f32);
            s.write_vec3f(Vec3::new(x, y, r.real() as f32));
        }
        15 => {
            let (a, b) = (r.flag(), r.flag());
            s.put_bools3(a, b, r.flag());
        }
        16 => {
            let mut v = [false; 7];
            for x in &mut v {
                *x = r.flag();
            }
            s.put_bools7(v);
        }
        _ => {
            for _ in 0..r.integer() {
                s.write_byte(0);
            }
        }
    }
    Ok(())
}

/// `br_bin_ostream`.
pub fn ostream_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut o = out_stream();
    for _ in 0..r.integer() {
        o.write(&[0]);
    }
    {
        let mut s = BinOStream::new(&mut o);
        long(s.position());
        for _ in 0..r.integer() {
            let op = r.integer();
            attempt(|| ostream_op(&mut s, &mut r, op));
            long(s.position());
        }
    }
    bytes_token(&written_bytes(o));
    end();
}

fn istream_op(s: &mut BinIStream<'_>, r: &mut Reader<'_>, op: i32) -> Result<(), Failure> {
    match op {
        0 => int(i32::from(s.read_type().0)),
        1 => {
            int(s.shape_type());
            int(s.shape_orientation() as i32);
        }
        2 => flag(s.is_reference()),
        3 => long(s.read_reference()?),
        4 => s.go_to(r.real() as u64),
        5 => real(s.read_real()?),
        6 => int(s.read_integer()?),
        7 => {
            let p = s.read_pnt()?;
            real(p.x());
            real(p.y());
            real(p.z());
        }
        8 => int(i32::from(s.read_byte())),
        9 => flag(s.read_bool()),
        10 => real(f64::from(s.read_short_real())),
        11 => {
            let t = s.read_transform()?;
            for i in 1..=3 {
                for j in 1..=4 {
                    real(t.value(i, j)?);
                }
            }
        }
        12 => s.read_bools3().into_iter().for_each(flag),
        13 => s.read_bools7().into_iter().for_each(flag),
        14 => s.update_position(),
        15 => flag(s.is_ok()),
        _ => int(i32::from(s.last_type().0)),
    }
    Ok(())
}

/// `br_bin_istream`.
pub fn istream_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut is = IoStream::from_bytes(read_bytes(&mut r));
    {
        let mut s = BinIStream::new(&mut is);
        long(s.position());
        for _ in 0..r.integer() {
            let op = r.integer();
            attempt(|| istream_op(&mut s, &mut r, op));
            long(s.position());
            int(i32::from(s.stream().rdstate()));
        }
    }
    end();
}
