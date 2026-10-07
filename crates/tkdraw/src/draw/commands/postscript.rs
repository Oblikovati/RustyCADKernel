//! The PostScript and image output commands of `Draw_GraphicCommands.cxx`:
//! `hardcopy`, `dfont`, `hcolor`, `xwd`.
use super::graphic::view_id;
use crate::draw::{DrawInterpretor, DrawOStream, DrawSession};
use rustycad_tkernel::standard::Failure;
use std::f64::consts::SQRT_2;

/// The paper format of `hardcopy`: margin, width and height in millimetres, scaled from
/// A4 (210 x 210*sqrt(2), 3 mm margin) with the C++ operations of each format.
fn paper_format(name: &str) -> (f64, f64, f64) {
    let scale = |value: f64| match name {
        "a7" => value / (2.0 * SQRT_2),
        "a6" => value / 2.0,
        "a5" => value / SQRT_2,
        "a3" => value * SQRT_2,
        "a2" => value * 2.0,
        "a1" => value * 2.0 * SQRT_2,
        "a0" => value * 4.0,
        _ => value,
    };
    (scale(3.0), scale(210.0), scale(210.0 * 2f64.sqrt()))
}

/// `hardcopy [file=a4.ps] [id=1] [format=a4]`: an EPS file of the view (the file is
/// created even when the view does not exist, as upstream).
pub(super) fn hardcopy(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    let file = a.get(1).copied().unwrap_or("a4.ps");
    let id = match a.get(2) {
        Some(text) => match view_id(session, text) {
            Some(id) => id,
            None => return Ok(1),
        },
        None => 1,
    };
    let (margin, width, height) = paper_format(a.get(3).copied().unwrap_or("a4"));
    const PIXELS_PER_MM: f64 = 28.4;
    let paper = [
        margin * PIXELS_PER_MM,
        margin * PIXELS_PER_MM,
        (width - margin) * PIXELS_PER_MM,
        (height - margin) * PIXELS_PER_MM,
    ]
    .map(|v| v as i32);
    let mut output = DrawOStream::new();
    let frame = session
        .viewer()
        .frame(id)
        .filter(|_| session.viewer().has_view(id));
    let drawn = match frame {
        Some(frame) => write_postscript(session, id, frame, paper, &mut output),
        None => Ok(()),
    };
    // `std::ofstream` failures go unreported upstream; when a drawing fails, the file
    // keeps what was written before the failure, as the closed stream does.
    let _ = std::fs::write(file, output.text());
    drawn.map(|()| 0)
}

/// The EPS header, frame, view drawing and trailer of `hardcopy`.
fn write_postscript(
    session: &DrawSession,
    id: i32,
    (vx0, vy0, vx1, vy1): (i32, i32, i32, i32),
    paper: [i32; 4],
    out: &mut DrawOStream,
) -> Result<(), Failure> {
    let [px0, py0, mut px1, mut py1] = paper;
    let kx = (px1 - px0) as f64 / (vx1 - vx0) as f64;
    let ky = (py1 - py0) as f64 / (vy1 - vy0) as f64;
    let k = kx.abs().min(ky.abs());
    let (kx, ky) = (if kx > 0.0 { k } else { -k }, if ky > 0.0 { k } else { -k });
    px1 = (px0 as f64 + kx * (vx1 - vx0) as f64) as i32;
    py1 = (py0 as f64 + ky * (vy1 - vy0) as f64) as i32;
    let (font, size) = session.font.borrow().clone();
    out.put_str(&format!(
        "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: {px0} {py0} {px1} {py1}\n%%Pages: 1\n"
    ));
    out.put_str(&format!(
        "%%DocumentFonts: {font}\n%%EndComments\n/{font} findfont\n{size} scalefont\nsetfont\n"
    ));
    out.put_str("/m {moveto} bind def\n/l {lineto} bind def\n.1 .1 scale\n3 setlinewidth\n0 setgray\nnewpath\n");
    let rectangle = format!("{px0} {py0} m\n{px1} {py0} l\n{px1} {py1} l\n{px0} {py1} l\n");
    out.put_str(&rectangle)
        .put_str("closepath\nstroke\nnewpath\n")
        .put_str(&rectangle)
        .put_str("closepath\nclip\n");
    session
        .viewer()
        .postscript_view(id, [vx0, vy0, vx1, vy1], [px0, py0, px1, py1], out)?;
    out.put_str("showpage\n%%EOF\n");
    Ok(())
}

/// `dfont [name [size]]`: sets the PostScript font, or resets it without arguments.
pub(super) fn dfont(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    {
        let mut font = session.font.borrow_mut();
        match a.len() {
            1 => *font = ("Helvetica".to_string(), "150".to_string()),
            2 => font.0 = a[1].to_string(),
            3 => *font = (a[1].to_string(), a[2].to_string()),
            _ => {}
        }
    }
    let (name, size) = session.font.borrow().clone();
    di.append(format!("{name} {size}\n"))?;
    Ok(0)
}

/// `hcolor icol width gray`: the PostScript pen of a palette entry, or the palette.
pub(super) fn hcolor(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 4 {
        di.append(
            "code de couleur (Draw.cxx) : \n0 = White,\t 1 = Red,\t 2 = Green,\t 3 = Blue\n",
        )?;
        di.append("4 = Cyan,\t 5 = Gold,\t 6 = Magenta,\t 7 = Maroon\n8 = Orange,\t 9 = Pink,\t 10 = Salmon,\t 11 = Violet\n")?;
        di.append("12 = Yellow,\t 13 = Khaki,\t 14 = Coral\n1 <= width <= 11,  0 (noir)  <= gray <= 1 (blanc)\n")?;
        return Ok(0);
    }
    let (index, width, gray) = (session.atoi(a[1]), session.atoi(a[2]), session.atof(a[3]));
    session.viewer().post_color(index, width, gray);
    Ok(0)
}

/// `xwd [id=1] file`: saves the view image.
pub(super) fn xwd(
    session: &DrawSession,
    _di: &DrawInterpretor,
    a: &[&str],
) -> Result<i32, Failure> {
    if a.len() < 2 {
        return Ok(1);
    }
    session.repaint_now_if_necessary()?;
    let (id, file) = if a.len() > 2 {
        (session.atoi(a[1]), a[2])
    } else {
        (1, a[1])
    };
    match session.viewer().save_view(id, file) {
        Ok(true) => Ok(0),
        Ok(false) => Ok(1),
        Err(message) => {
            session.console().write_err(&message);
            Ok(1)
        }
    }
}
