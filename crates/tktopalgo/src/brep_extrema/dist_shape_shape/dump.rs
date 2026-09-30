// Rust translation of BRepExtrema_DistShapeShape::Dump
// (ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistShapeShape.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_DistShapeShape::Dump`: the distance and every solution as text.
use super::BRepExtremaDistShapeShape;
use crate::brep_extrema::BRepExtremaSupportType;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point3;
use std::io::Write;

/// A double as `operator<<` writes it (`%g`, six significant digits).
fn g(value: f64) -> String {
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}

/// The coordinates line of a point.
fn coordinates(point: &Point3) -> String {
    format!("X={} Y={} Z={}\n", g(point.x()), g(point.y()), g(point.z()))
}

/// An I/O error of the stream as a failure.
fn io_failure(error: std::io::Error) -> Failure {
    Failure::new(&format!("BRepExtrema_DistShapeShape::Dump: {error}"))
}

impl BRepExtremaDistShapeShape {
    /// `Dump(o)`: the distance, the number of solutions, then for each solution the
    /// support kinds, both points and the parameters on edges and faces, written to
    /// `out` (StdFail_NotDone when the distance is not found, as `Value()`);
    /// `dist.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        // C++17 sequences `o << "..." << Value()` left to right: the label is written
        // before Value() raises.
        out.write_all(b"the distance  value is :  ").map_err(io_failure)?;
        let mut text: String = format!("{}\n", g(self.value()?));
        text += &format!("the number of solutions is :{}\n\n", self.nb_solution());
        for i in 1..=self.nb_solution() {
            text += &self.solution_text(i)?;
        }
        out.write_all(text.as_bytes()).map_err(io_failure)
    }

    /// The text of the solution `i`.
    fn solution_text(&self, i: i32) -> Result<String, Failure> {
        let (kind1, kind2) = (self.support_type_shape1(i)?, self.support_type_shape2(i)?);
        let mut text: String = format!("solution number {i}: \n");
        text += &format!("the type of the solution on the first shape is {}\n", kind1 as i32);
        text += &format!("the type of the solution on the second shape is {}\n", kind2 as i32);
        text += "the coordinates of  the point on the first shape are: \n";
        text += &coordinates(&self.point_on_shape1(i)?);
        text += "the coordinates of  the point on the second shape are: \n";
        text += &coordinates(&self.point_on_shape2(i)?);
        text += &match kind1 {
            BRepExtremaSupportType::IsVertex => String::new(),
            BRepExtremaSupportType::IsOnEdge => format!("parameter on the first edge :  t= {}\n", g(self.par_on_edge_s1(i)?)),
            BRepExtremaSupportType::IsInFace => {
                let (u, v) = self.par_on_face_s1(i)?;
                format!("parameters on the first face :  u= {} v={}\n", g(u), g(v))
            }
        };
        text += &match kind2 {
            BRepExtremaSupportType::IsVertex => String::new(),
            BRepExtremaSupportType::IsOnEdge => format!("parameter on the second edge : t={}\n", g(self.par_on_edge_s2(i)?)),
            BRepExtremaSupportType::IsInFace => {
                let (u, v) = self.par_on_face_s2(i)?;
                format!("parameters on the second face : u= {} v={}\n", g(u), g(v))
            }
        };
        Ok(text + "\n")
    }
}
