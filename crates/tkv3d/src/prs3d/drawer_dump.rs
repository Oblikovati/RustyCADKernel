//! `Prs3d_Drawer::DumpJson`.
use super::Prs3dDrawer;
use super::basic_aspect::dump_aspect_field;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_pointer, field_real};
use std::io;

/// The native field list of `Prs3d_Drawer::DumpJson` (the base attributes are not dumped).
/// Example: `drawer_dump_json(&drawer, &mut stream, -1)?`.
pub(crate) fn drawer_dump_json(
    drawer: &Prs3dDrawer,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let f = drawer.fields_snapshot();
    class_begin(stream, "Prs3d_Drawer")?;
    field_pointer(stream, "myLink.get()", f.link.object_address() as usize)?;
    field_real(stream, "myMaximalParameterValue", f.maximal_parameter_value)?;
    field_real(stream, "myChordialDeviation", f.chordial_deviation)?;
    field_integer(stream, "myTypeOfDeflection", f.type_of_deflection as i32)?;
    field_integer(
        stream,
        "myHasOwnTypeOfDeflection",
        i32::from(f.has_own_type_of_deflection),
    )?;
    field_integer(stream, "myTypeOfHLR", f.type_of_hlr as i32)?;
    field_real(stream, "myDeviationCoefficient", f.deviation_coefficient)?;
    field_real(
        stream,
        "myPreviousDeviationCoefficient",
        f.previous_deviation_coefficient,
    )?;
    field_real(stream, "myDeviationAngle", f.deviation_angle)?;
    field_real(
        stream,
        "myPreviousDeviationAngle",
        f.previous_deviation_angle,
    )?;
    let first_flags = [
        ("myIsoOnPlane", f.iso_on_plane),
        ("myHasOwnIsoOnPlane", f.has_own_iso_on_plane),
        ("myIsoOnTriangulation", f.iso_on_triangulation),
        ("myHasOwnIsoOnTriangulation", f.has_own_iso_on_triangulation),
        ("myIsAutoTriangulated", f.is_auto_triangulated),
        ("myHasOwnIsAutoTriangulated", f.has_own_is_auto_triangulated),
        ("myWireDraw", f.wire_draw),
        ("myHasOwnWireDraw", f.has_own_wire_draw),
    ];
    dump_flags(stream, &first_flags)?;
    dump_aspect_field(stream, depth, "myShadingAspect.get()", &f.shading_aspect)?;
    dump_aspect_field(stream, depth, "myTextAspect.get()", &f.text_aspect)?;
    dump_flags(
        stream,
        &[
            ("myLineArrowDraw", f.line_arrow_draw),
            ("myHasOwnLineArrowDraw", f.has_own_line_arrow_draw),
            ("myDrawHiddenLine", f.draw_hidden_line),
            ("myHasOwnDrawHiddenLine", f.has_own_draw_hidden_line),
        ],
    )?;
    field_integer(stream, "myVertexDrawMode", f.vertex_draw_mode as i32)?;
    dump_flags(
        stream,
        &[
            ("myFreeBoundaryDraw", f.free_boundary_draw),
            ("myHasOwnFreeBoundaryDraw", f.has_own_free_boundary_draw),
            ("myUnFreeBoundaryDraw", f.un_free_boundary_draw),
            (
                "myHasOwnUnFreeBoundaryDraw",
                f.has_own_un_free_boundary_draw,
            ),
        ],
    )?;
    let continuity = f
        .face_boundary_upper_continuity
        .map_or(-1, |shape| shape as i32);
    field_integer(stream, "myFaceBoundaryUpperContinuity", continuity)?;
    dump_flags(
        stream,
        &[
            ("myFaceBoundaryDraw", f.face_boundary_draw),
            ("myHasOwnFaceBoundaryDraw", f.has_own_face_boundary_draw),
            (
                "myHasOwnDimLengthModelUnits",
                f.has_own_dim_length_model_units,
            ),
            (
                "myHasOwnDimAngleModelUnits",
                f.has_own_dim_angle_model_units,
            ),
            (
                "myHasOwnDimLengthDisplayUnits",
                f.has_own_dim_length_display_units,
            ),
            (
                "myHasOwnDimAngleDisplayUnits",
                f.has_own_dim_angle_display_units,
            ),
        ],
    )
}

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` of booleans, as 0 or 1.
fn dump_flags(stream: &mut DumpStream, flags: &[(&str, bool)]) -> io::Result<()> {
    flags
        .iter()
        .try_for_each(|(field, value)| field_integer(stream, field, i32::from(*value)))
}
