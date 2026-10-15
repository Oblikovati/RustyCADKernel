// Rust translation of Poly::Write and Poly::Dump in FoundationClasses/TKMath/Poly/Poly.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Compact "save" text and the informative "dump" text with the native `setw` columns.
use crate::{
    gp::Point3,
    poly::{Polygon2D, Polygon3D, Triangulation},
    stream_format::general,
};
use rustycad_tkernel::standard::Failure;
use std::io::Write;

/// Formats one real as `operator<<(double)` does on the caller's `std::ostream`, whose
/// precision and float field the native writers inherit; [`general`] is the default.
/// `write_polygon3d_with(&polygon, &mut out, true, &|value| format!("{value:.3e}"))?`.
pub type RealText<'a> = &'a dyn Fn(f64) -> String;

/// Streams the values of one line, with a `setw` column only in dump mode.
struct Columns<'a, W: Write> {
    output: &'a mut W,
    compact: bool,
    real_text: RealText<'a>,
}
impl<W: Write> Columns<'_, W> {
    fn text(&mut self, text: &str) -> Result<(), Failure> {
        self.output
            .write_all(text.as_bytes())
            .map_err(|error| Failure::new(&format!("Poly::Write stream failed: {error}")))
    }
    fn row_label(&mut self, index: i32) -> Result<(), Failure> {
        if self.compact {
            return Ok(());
        }
        self.text(&format!("{index:>10} : "))
    }
    fn real(&mut self, value: f64, separator: &str) -> Result<(), Failure> {
        let number = (self.real_text)(value);
        let text = if self.compact {
            number
        } else {
            // `std::setw(17) << value`: right-aligned in at least 17 columns.
            format!("{number:>17}")
        };
        self.text(&text)?;
        self.text(separator)
    }
    fn integer(&mut self, value: i32, separator: &str) -> Result<(), Failure> {
        let text = if self.compact {
            value.to_string()
        } else {
            format!("{value:>10}")
        };
        self.text(&text)?;
        self.text(separator)
    }
    fn dump_only(&mut self, text: &str) -> Result<(), Failure> {
        if self.compact {
            Ok(())
        } else {
            self.text(text)
        }
    }
    fn counts(&mut self, compact: &str, dump: &str) -> Result<(), Failure> {
        self.text(if self.compact { compact } else { dump })
    }
    fn point3(&mut self, index: i32, point: Point3) -> Result<(), Failure> {
        self.row_label(index)?;
        self.real(point.x(), " ")?;
        self.real(point.y(), " ")?;
        self.real(point.z(), "\n")
    }
    fn deflection(&mut self, deflection: f64) -> Result<(), Failure> {
        self.dump_only("Deflection : ")?;
        let number = (self.real_text)(deflection);
        self.text(&format!("{number}\n"))
    }
}
/// `Poly::Write(Poly_Triangulation)`: compact save text or, when not `compact`, the dump;
/// `write_triangulation(&mesh, &mut out, true)?`.
pub fn write_triangulation(
    triangulation: &Triangulation,
    output: &mut impl Write,
    compact: bool,
) -> Result<(), Failure> {
    write_triangulation_with(triangulation, output, compact, &general)
}
/// [`write_triangulation`] on a stream whose numbers `real_text` formats;
/// `write_triangulation_with(&mesh, &mut out, false, &|v| format!("{v}"))?`.
pub fn write_triangulation_with(
    triangulation: &Triangulation,
    output: &mut impl Write,
    compact: bool,
    real_text: RealText<'_>,
) -> Result<(), Failure> {
    let mut columns = Columns {
        output,
        compact,
        real_text,
    };
    let (nodes, triangles) = (triangulation.nb_nodes(), triangulation.nb_triangles());
    let with = if triangulation.has_uv_nodes() {
        "with"
    } else {
        "without"
    };
    columns.text("Poly_Triangulation\n")?;
    columns.counts(
        &format!("{nodes} {triangles} {}\n", u8::from(with == "with")),
        &format!("{nodes:>8} Nodes\n{triangles:>8} Triangles\n{with} UV nodes\n"),
    )?;
    columns.deflection(triangulation.deflection())?;
    columns.dump_only("\n3D Nodes :\n")?;
    for index in 1..=nodes {
        columns.point3(index, triangulation.node(index)?)?;
    }
    write_uv_nodes(&mut columns, triangulation)?;
    columns.dump_only("\nTriangles :\n")?;
    for index in 1..=triangles {
        let (a, b, c) = triangulation.triangle(index)?.get();
        columns.row_label(index)?;
        columns.integer(a, " ")?;
        columns.integer(b, " ")?;
        columns.integer(c, "\n")?;
    }
    Ok(())
}
fn write_uv_nodes<W: Write>(
    columns: &mut Columns<'_, W>,
    triangulation: &Triangulation,
) -> Result<(), Failure> {
    if !triangulation.has_uv_nodes() {
        return Ok(());
    }
    columns.dump_only("\nUV Nodes :\n")?;
    for index in 1..=triangulation.nb_nodes() {
        let node = triangulation.uv_node(index)?;
        columns.row_label(index)?;
        columns.real(node.x(), " ")?;
        columns.real(node.y(), "\n")?;
    }
    Ok(())
}
/// `Poly::Write(Poly_Polygon3D)`; `write_polygon3d(&polygon, &mut out, true)?`.
pub fn write_polygon3d(
    polygon: &Polygon3D,
    output: &mut impl Write,
    compact: bool,
) -> Result<(), Failure> {
    write_polygon3d_with(polygon, output, compact, &general)
}
/// [`write_polygon3d`] on a stream whose numbers `real_text` formats;
/// `write_polygon3d_with(&polygon, &mut out, true, &|v| format!("{v:e}"))?`.
pub fn write_polygon3d_with(
    polygon: &Polygon3D,
    output: &mut impl Write,
    compact: bool,
    real_text: RealText<'_>,
) -> Result<(), Failure> {
    let mut columns = Columns {
        output,
        compact,
        real_text,
    };
    let count = polygon.nb_nodes();
    let with = if polygon.has_parameters() {
        "with"
    } else {
        "without"
    };
    columns.text("Poly_Polygon3D\n")?;
    columns.counts(
        &format!("{count} {}\n", u8::from(with == "with")),
        &format!("{count:>8} Nodes\n{with} parameters\n"),
    )?;
    columns.deflection(polygon.deflection())?;
    columns.dump_only("\nNodes :\n")?;
    for (offset, node) in polygon.nodes().iter().enumerate() {
        columns.point3(offset as i32 + 1, *node)?;
    }
    if !polygon.has_parameters() {
        return Ok(());
    }
    columns.dump_only("\nParameters :\n")?;
    let parameters = polygon.parameters()?;
    for index in 1..=count {
        let value = parameters
            .value(index)
            .map_err(|e| e.into_failure("Poly::Write"))?;
        let number = (columns.real_text)(*value);
        columns.text(&format!("{number} "))?;
    }
    columns.text("\n")
}
/// `Poly::Write(Poly_Polygon2D)`; `write_polygon2d(&polygon, &mut out, true)?`.
pub fn write_polygon2d(
    polygon: &Polygon2D,
    output: &mut impl Write,
    compact: bool,
) -> Result<(), Failure> {
    write_polygon2d_with(polygon, output, compact, &general)
}
/// [`write_polygon2d`] on a stream whose numbers `real_text` formats;
/// `write_polygon2d_with(&polygon, &mut out, false, &|v| format!("{v}"))?`.
pub fn write_polygon2d_with(
    polygon: &Polygon2D,
    output: &mut impl Write,
    compact: bool,
    real_text: RealText<'_>,
) -> Result<(), Failure> {
    let mut columns = Columns {
        output,
        compact,
        real_text,
    };
    let count = polygon.nb_nodes();
    columns.text("Poly_Polygon2D\n")?;
    if compact {
        columns.text(&format!("{count} "))?;
    } else {
        columns.text(&format!("{count:>8} Nodes\n"))?;
    }
    columns.deflection(polygon.deflection())?;
    columns.dump_only("\nNodes :\n")?;
    for (offset, node) in polygon.nodes().iter().enumerate() {
        columns.row_label(offset as i32 + 1)?;
        columns.real(node.x(), " ")?;
        columns.real(node.y(), "\n")?;
    }
    Ok(())
}
/// `Poly::Dump(Poly_Triangulation)`: the non-compact write; `dump_triangulation(&mesh, &mut out)?`.
pub fn dump_triangulation(
    triangulation: &Triangulation,
    output: &mut impl Write,
) -> Result<(), Failure> {
    write_triangulation(triangulation, output, false)
}
/// `Poly::Dump(Poly_Polygon3D)`; `dump_polygon3d(&polygon, &mut out)?`.
pub fn dump_polygon3d(polygon: &Polygon3D, output: &mut impl Write) -> Result<(), Failure> {
    write_polygon3d(polygon, output, false)
}
/// `Poly::Dump(Poly_Polygon2D)`; `dump_polygon2d(&polygon, &mut out)?`.
pub fn dump_polygon2d(polygon: &Polygon2D, output: &mut impl Write) -> Result<(), Failure> {
    write_polygon2d(polygon, output, false)
}

#[cfg(test)]
#[path = "write_tests.rs"]
mod tests;
