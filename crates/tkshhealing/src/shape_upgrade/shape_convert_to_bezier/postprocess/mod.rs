use super::{ShapeDivider, ShapeUpgradeShapeConvertToBezier};
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

mod edges;

pub(super) fn run(tool: &ShapeUpgradeShapeConvertToBezier) -> Result<(), Failure> {
    let result = tool.result();
    let mut faces = Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let face = faces.current().clone();
        process_face(tool, &face)?;
        faces.next()?;
    }
    Ok(())
}

fn process_face(tool: &ShapeUpgradeShapeConvertToBezier, source: &Shape) -> Result<(), Failure> {
    let mut face = topods::face(source)?;
    face.set_orientation(Orientation::Forward);
    let mut wires = Explorer::new(&face, ShapeEnum::Wire, ShapeEnum::Shape)?;
    while wires.more() {
        let wire = topods::wire(wires.current())?;
        process_wire(tool, &wire, &face)?;
        wires.next()?;
    }
    rustycad_tkbrep::brep_tools::update_face(&face)
}

fn process_wire(
    tool: &ShapeUpgradeShapeConvertToBezier,
    wire: &rustycad_tkbrep::topods::Wire,
    face: &Face,
) -> Result<(), Failure> {
    let fixer = crate::shape_fix::ShapeFixWire::new();
    fixer.load(wire)?;
    fixer.set_face(face)?;
    fixer.set_precision(tool.shape_divide_base().with(|fields| fields.precision));
    fixer.fix_reorder()?;
    fixer.fix_shifted()?;
    let wire_data = fixer.wire_data()?;
    edges::process_wire_edges(tool, face, &wire_data)
}
