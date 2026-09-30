//! Public WireDivide state transitions are compared with the native class.
use super::{
    brep_lib_common::shape,
    common::{Reader, attempt, end, flag, int},
    healing_common::flags,
    shapes::tp_shapes,
    wire_data::{sh_edge, sh_face},
};
use rustycad_tkbrep::topods::{Edge, Face, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisTransferParameters,
    shape_build::ShapeBuildReShape,
    shape_upgrade::{
        ShapeUpgradeEdgeDivide, ShapeUpgradeFixSmallCurves, ShapeUpgradeWireDivide, WireDivider,
    },
};
use rustycad_tktopalgo::brep_lib::BRepLibMakeWire;
/// Compare native state transitions; `wire_state_case(&request)`.
pub fn wire_state_case(values: &[f64]) {
    attempt(|| state_request(&mut Reader::new(values)));
    end();
}
fn state_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let shapes = tp_shapes(reader)?;
    let divider = ShapeUpgradeWireDivide::new();
    let edge = sh_edge(&shapes, reader.integer())?;
    let face = sh_face(&shapes, reader.integer())?;
    let surface = &shapes.surfaces[reader.integer() as usize];
    let location = &shapes.locations[reader.integer() as usize];
    let wire = BRepLibMakeWire::from_edge(&edge)?.wire()?;
    divider.set_context(&Handle::new(ShapeBuildReShape::new()));
    print_state(&divider)?;
    initialize_geometry(&divider, &wire, &edge, &face, surface, location)?;
    configure_and_perform(&divider, &wire, &edge, &face, reader)
}
fn initialize_geometry(
    divider: &ShapeUpgradeWireDivide,
    wire: &Wire,
    edge: &Edge,
    face: &Face,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<(), Failure> {
    divider.init(wire, face);
    print_state(divider)?;
    divider.init_surface(wire, surface)?;
    print_state(divider)?;
    divider.set_face(face);
    print_state(divider)?;
    divider.set_surface(surface)?;
    print_state(divider)?;
    divider.set_surface_location(surface, location)?;
    print_state(divider)?;
    divider.load_wire(&Wire::default());
    print_state(divider)?;
    divider.load_edge(edge)?;
    print_state(divider)?;
    divider.load_wire(wire);
    print_state(divider)?;
    Ok(())
}
fn configure_and_perform(
    divider: &ShapeUpgradeWireDivide,
    wire: &Wire,
    edge: &Edge,
    face: &Face,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    let identity = super::wire_divide::configure_tools(divider, reader);
    flag(identity.0);
    flag(identity.1);
    replace_tools(divider);
    divider.set_edge_mode(reader.integer());
    print_state(divider)?;
    divider.set_face(face);
    attempt(|| divider.perform());
    print_state(divider)?;
    divider.load_edge(edge)?;
    print_state(divider)?;
    divider.init(wire, face);
    print_state(divider)?;
    divider.set_transfer_param_tool(&Handle::default());
    divider.set_edge_divide_tool(&Handle::default());
    divider.set_fix_small_curve_tool(&Handle::default());
    print_state(divider)
}
fn replace_tools(divider: &ShapeUpgradeWireDivide) {
    let transfer = Handle::new(ShapeAnalysisTransferParameters::new());
    let criterion = Handle::new(ShapeUpgradeEdgeDivide::new());
    let small = Handle::new(ShapeUpgradeFixSmallCurves::new());
    divider.set_transfer_param_tool(&transfer);
    divider.set_edge_divide_tool(&criterion);
    divider.set_fix_small_curve_tool(&small);
    flag(divider.transfer_param_tool() == transfer);
    flag(divider.edge_divide_tool() == criterion);
    flag(divider.fix_small_curve_tool() == small);
}
fn print_state(divider: &ShapeUpgradeWireDivide) -> Result<(), Failure> {
    flags(|status| divider.status_flag(status));
    divider.wire_divide_data().with(|fields| {
        int(fields.edge_mode);
        shape(&fields.face)?;
        shape(&fields.wire)
    })?;
    flag(divider.split_curve_3d_tool().is_null());
    flag(divider.split_curve_2d_tool().is_null());
    flag(divider.transfer_param_tool().is_null());
    flag(divider.edge_divide_tool().is_null());
    flag(divider.fix_small_curve_tool().is_null());
    Ok(())
}
