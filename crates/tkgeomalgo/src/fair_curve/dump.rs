// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_Batten.cxx and
// FairCurve_MinimalVariation.cxx (Dump) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Dump` of the batten and the minimal-variation curve: one row per constraint
//! with the computed ("Old") and requested ("New") values. As natively,
//! `o.width(7)` pads only the old value; reals use the default stream format.
use super::batten::BattenConstraints;
use super::{FairCurveAnalysisCode, FairCurveBatten, FairCurveMinimalVariation};
use rustycad_tkernel::tcollection::AsciiString;
use std::io;

/// A real in the default stream format (`%g`).
fn g(value: f64) -> String {
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}

/// `label`, the old value padded to 7 characters, ` | ` and the new value.
fn row(o: &mut impl io::Write, label: &str, old: String, new: String) -> io::Result<()> {
    writeln!(o, "{label}{old:>7} | {new}")
}

/// The rows of the end points and angles.
fn point_rows(
    o: &mut impl io::Write,
    old: &BattenConstraints,
    new: &BattenConstraints,
) -> io::Result<()> {
    row(o, "  P1    X      |", g(old.p1.x()), g(new.p1.x()))?;
    row(o, "        Y      |", g(old.p1.y()), g(new.p1.y()))?;
    row(o, "  P2    X      |", g(old.p2.x()), g(new.p2.x()))?;
    row(o, "        Y      |", g(old.p2.y()), g(new.p2.y()))?;
    row(o, "      Angle1   |", g(old.angle1), g(new.angle1))?;
    row(o, "      Angle2   |", g(old.angle2), g(new.angle2))
}

/// The rows of the height and slope.
fn height_rows(
    o: &mut impl io::Write,
    old: &BattenConstraints,
    new: &BattenConstraints,
) -> io::Result<()> {
    row(o, "      Height   |", g(old.height), g(new.height))?;
    row(o, "      Slope    |", g(old.slope), g(new.slope))
}

/// The rows of the sliding and constraint orders, then the analysis code.
fn tail_rows(o: &mut impl io::Write, batten: &FairCurveBatten) -> io::Result<()> {
    let (old, new) = (&batten.old, &batten.new);
    row(
        o,
        " SlidingFactor |",
        g(old.sliding_factor),
        g(new.sliding_factor),
    )?;
    let flag = |value: bool| i32::from(value).to_string();
    row(
        o,
        " FreeSliding   |",
        flag(old.free_sliding),
        flag(new.free_sliding),
    )?;
    let (old1, new1) = (old.constraint_order1, new.constraint_order1);
    row(o, " ConstrOrder1  |", old1.to_string(), new1.to_string())?;
    let (old2, new2) = (old.constraint_order2, new.constraint_order2);
    row(o, " ConstrOrder2  |", old2.to_string(), new2.to_string())?;
    let code: &str = match batten.code {
        FairCurveAnalysisCode::Ok => "Ok",
        FairCurveAnalysisCode::NotConverged => "NotConverged",
        FairCurveAnalysisCode::InfiniteSliding => "InfiniteSliding",
        FairCurveAnalysisCode::NullHeight => "NullHeight",
    };
    writeln!(o, "AnalysisCode : {code}")
}

impl FairCurveBatten {
    /// Writes the native report of the constraints; `batten.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, o: &mut impl io::Write) -> io::Result<()> {
        writeln!(o, "  Batten       |Old  |   New")?;
        point_rows(o, &self.old, &self.new)?;
        height_rows(o, &self.old, &self.new)?;
        tail_rows(o, self)
    }
}

impl FairCurveMinimalVariation {
    /// Writes the native report, with the curvatures and the physical ratio;
    /// `mvc.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, o: &mut impl io::Write) -> io::Result<()> {
        let batten: &FairCurveBatten = self;
        writeln!(o, "  MVCurve      |Old  |   New")?;
        point_rows(o, &batten.old, &batten.new)?;
        let (old, new) = (self.old_curvatures, self.new_curvatures);
        row(o, " Curvature1    |", g(old[0]), g(new[0]))?;
        row(o, " Curvature2    |", g(old[1]), g(new[1]))?;
        height_rows(o, &batten.old, &batten.new)?;
        row(
            o,
            " PhysicalRatio |",
            g(self.old_physical_ratio),
            g(self.new_physical_ratio),
        )?;
        tail_rows(o, batten)
    }
}
