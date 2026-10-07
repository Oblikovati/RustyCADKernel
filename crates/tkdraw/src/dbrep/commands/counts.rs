// Rust translation of the counting commands of Draw/TKDraw/DBRep/DBRep.cxx (nbshapes,
// numshapes, DumpExtent, countshapes) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `nbshapes`, `numshapes` and `countshapes`.
use super::CommandEntry;
use crate::draw::{DrawInterpretor, DrawSession};
use rustycad_tkbrep::brep_tools::BRepToolsShapeSet;
use rustycad_tkbrep::topexp::{Explorer, map_shapes_of_type};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape, ShapeSet};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `nbshapes`.
pub(super) const NBSHAPES: CommandEntry = (
    "nbshapes",
    "\n nbshapes s - shows the number of sub-shapes in <s>;\n nbshapes s -t - shows the number of sub-shapes in <s> counting the same sub-shapes with different location as different sub-shapes.",
    nbshapes,
);
/// `numshapes`.
pub(super) const NUMSHAPES: CommandEntry = ("numshapes", "numshapes s; size of shape", numshapes);
/// `countshapes`.
pub(super) const COUNTSHAPES: CommandEntry =
    ("countshapes", "countshapes s; count of shape", countshapes);

/// The shape kinds from COMPOUND to SHAPE with their `DumpExtent` labels.
const EXTENT_LABELS: [(ShapeEnum, &str); 9] = [
    (ShapeEnum::Shape, " SHAPE     : "),
    (ShapeEnum::Compound, " COMPOUND  : "),
    (ShapeEnum::CompSolid, " COMPSOLID : "),
    (ShapeEnum::Solid, " SOLID     : "),
    (ShapeEnum::Shell, " SHELL     : "),
    (ShapeEnum::Face, " FACE      : "),
    (ShapeEnum::Wire, " WIRE      : "),
    (ShapeEnum::Edge, " EDGE      : "),
    (ShapeEnum::Vertex, " VERTEX    : "),
];

/// `BRepTools_ShapeSet::DumpExtent` of the set holding `shape`.
fn shape_set_extent(shape: &Shape) -> Result<String, Failure> {
    let mut set = BRepToolsShapeSet::default();
    set.add(shape)?;
    let mut text = AsciiString::default();
    set.dump_extent_string(&mut text)?;
    Ok(String::from_utf8_lossy(text.as_bytes()).into_owned())
}

/// The static `DumpExtent`: the `TopExp::MapShapes` count of each kind (same sub-shape
/// with another location counted again), from VERTEX up to COMPOUND, then the total.
fn dump_extent(shape: &Shape) -> Result<String, Failure> {
    let mut text = String::new();
    let mut total = 0;
    for (kind, label) in EXTENT_LABELS[1..].iter().rev() {
        let mut map = IndexedMapOfShape::default();
        map_shapes_of_type(shape, *kind, &mut map)?;
        let count = map.extent();
        text.push_str(&format!("{label}{count}\n"));
        total += count;
    }
    text.push_str(&format!("{}{total}\n", EXTENT_LABELS[0].1));
    Ok(text)
}

/// `nbshapes names... [-t]`: the sub-shape counts of each shape.
fn nbshapes(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let n = a.len();
    if n < 2 {
        return Ok(1);
    }
    let total = a[n - 1] == "-t";
    for name in &a[1..] {
        let shape = session.shape(name)?;
        if shape.is_null() {
            continue;
        }
        di.append(format!("Number of shapes in {name}\n"))?;
        let text = if total {
            dump_extent(&shape)?
        } else {
            shape_set_extent(&shape)?
        };
        di.append(text)?;
    }
    Ok(0)
}

/// `numshapes names...`: as `nbshapes`, with an extra newline.
fn numshapes(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() < 2 {
        return Ok(1);
    }
    for name in &a[1..] {
        let shape = session.shape(name)?;
        if shape.is_null() {
            continue;
        }
        di.append(format!("Number of shapes in {name}\n"))?;
        di.append(shape_set_extent(&shape)?)?.append("\n")?;
    }
    Ok(0)
}

/// The kinds `countshapes` counts, in its order, with their labels.
const COUNT_LABELS: [(ShapeEnum, &str); 8] = [
    (ShapeEnum::Vertex, " VERTEX     : "),
    (ShapeEnum::Edge, " EDGE       : "),
    (ShapeEnum::Wire, " WIRE       : "),
    (ShapeEnum::Face, " FACE       : "),
    (ShapeEnum::Shell, " SHELL      : "),
    (ShapeEnum::Solid, " SOLID      : "),
    (ShapeEnum::CompSolid, " COMPSOLID  : "),
    (ShapeEnum::Compound, " COMPOUND   : "),
];

/// `countshapes names...`: the distinct sub-shapes (`IsSame`) of each kind, then the
/// total.
fn countshapes(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() < 2 {
        return Ok(1);
    }
    for name in &a[1..] {
        let shape = session.shape(name)?;
        if shape.is_null() {
            continue;
        }
        di.append(format!("Number of shapes in {name}\n"))?;
        let mut seen = MapOfShape::default();
        for (kind, label) in COUNT_LABELS {
            let mut count = 0;
            let mut explorer = Explorer::new(&shape, kind, ShapeEnum::Shape)?;
            while explorer.more() {
                if seen.add(explorer.current().clone()) {
                    count += 1;
                }
                explorer.next()?;
            }
            di.append(format!("{label}{count}\n"))?;
        }
        di.append(format!(" SHAPE      : {}\n\n", seen.extent()))?;
    }
    Ok(0)
}
