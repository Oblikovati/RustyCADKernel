//! Real context style construction and borrowed handle graph, compared with OCCT.
use super::common::{Reader, end, flag, int, real};
use rustycad_tkernel::{
    quantity::{Color, ColorSpace},
    standard::Handle,
};
use rustycad_tkservice::{
    aspect::AspectTypeOfFacingModel, graphic3d::*, null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::{ais::*, prs3d::*, select_mgr::*, v3d::V3dViewer};
fn context_color(color: Color) {
    let [r, g, b] = color.values(ColorSpace::Rgb);
    real(r);
    real(g);
    real(b);
}
fn context_line(line: Handle<Prs3dLineAspect>) {
    flag(!line.is_null());
    let Some(line) = line.get() else { return };
    context_color(line.aspect().interior_color());
    real(line.aspect().width() as f64);
    int(line.aspect().line_kind() as i32);
}
fn context_plane(plane: Handle<Prs3dPlaneAspect>) {
    flag(!plane.is_null());
    let Some(plane) = plane.get() else { return };
    real(plane.plane_x_length());
    real(plane.plane_y_length());
    context_line(plane.edges_aspect());
    context_line(plane.iso_aspect());
    context_line(plane.arrow_aspect());
}
fn context_datum(datum: Handle<Prs3dDatumAspect>) {
    flag(!datum.is_null());
    let Some(datum) = datum.get() else { return };
    for part in Prs3dDatumParts::DRAWN {
        context_line(datum.line_aspect(part));
    }
    for axis in [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ] {
        real(datum.axis_length(axis));
    }
}
fn context_style(style: &Prs3dDrawer, base: &Handle<Prs3dDrawer>) {
    int(style.display_mode());
    int(style.z_layer());
    int(style.method() as i32);
    context_color(style.color());
    flag(style.link() == *base);
    flag(style.is_auto_triangulation());
    context_line(style.line_aspect());
    context_line(style.wire_aspect());
    context_line(style.free_boundary_aspect());
    context_line(style.un_free_boundary_aspect());
    context_line(style.hidden_line_aspect());
    context_plane(style.plane_aspect());
    context_datum(style.datum_aspect());
    context_marker_shading(style);
    flag(
        !style.plane_aspect().is_null()
            && style.plane_aspect().edges_aspect().aspect()
                == base.plane_aspect().edges_aspect().aspect(),
    );
    flag(
        !style.line_aspect().is_null()
            && style.line_aspect().aspect() == base.line_aspect().aspect(),
    );
}
fn context_marker_shading(style: &Prs3dDrawer) {
    let point = style.point_aspect();
    flag(!point.is_null());
    if let Some(point) = point.get() {
        context_color(point.aspect().interior_color());
        int(point.aspect().marker_kind() as i32);
        real(point.aspect().scale() as f64);
    }
    let shade = style.shading_aspect();
    flag(!shade.is_null());
    if let Some(shade) = shade.get() {
        context_color(shade.color(AspectTypeOfFacingModel::FrontSide));
        int(shade.material(AspectTypeOfFacingModel::FrontSide).name() as i32);
    }
}
fn context_style_id(style: &Handle<Prs3dDrawer>, styles: &[Handle<Prs3dDrawer>]) -> i32 {
    styles
        .iter()
        .position(|candidate| candidate == style)
        .map_or(-1, |index| index as i32)
}
fn context_kind(index: i32) -> Prs3dTypeOfHighlight {
    [
        Prs3dTypeOfHighlight::None,
        Prs3dTypeOfHighlight::Selected,
        Prs3dTypeOfHighlight::Dynamic,
        Prs3dTypeOfHighlight::LocalSelected,
        Prs3dTypeOfHighlight::LocalDynamic,
        Prs3dTypeOfHighlight::SubIntensity,
    ][index as usize]
}
fn context_state(ctx: &AisInteractiveContext, styles: &[Handle<Prs3dDrawer>]) {
    flag(ctx.to_hilight_selected());
    flag(ctx.automatic_hilight());
    flag(ctx.get_auto_activate_selection());
    int(ctx.picking_strategy() as i32);
    int(context_style_id(&ctx.default_drawer(), styles));
    for i in 0..6 {
        int(context_style_id(
            &ctx.highlight_style(context_kind(i)).unwrap(),
            styles,
        ));
    }
    int(context_style_id(&ctx.dynamic_highlight_style(), styles));
    int(context_style_id(&ctx.selection_style(), styles));
}
fn context_op(
    ctx: &AisInteractiveContext,
    styles: &[Handle<Prs3dDrawer>],
    op: i32,
    slot: i32,
    value: i32,
) {
    match op {
        0 => ctx.set_default_drawer(&styles[value as usize]),
        1 => ctx
            .set_highlight_style(context_kind(slot), &styles[value as usize])
            .unwrap(),
        2 => ctx.set_dynamic_highlight_style(&styles[value as usize]),
        3 => ctx.set_selection_style(&styles[value as usize]),
        4 => ctx.set_to_hilight_selected(value != 0),
        5 => ctx.set_automatic_hilight(value != 0),
        6 => ctx.set_auto_activate_selection(value != 0),
        7 => ctx.set_picking_strategy(if value == 0 {
            SelectMgrPickingStrategy::FirstAcceptable
        } else {
            SelectMgrPickingStrategy::OnlyTopmost
        }),
        8 => ctx.update_current_viewer(),
        _ => unreachable!(),
    }
}
pub fn ais_context_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let ctx = AisInteractiveContext::new(&viewer).unwrap();
    let mut styles = vec![Handle::default()];
    for i in 0..6 {
        styles.push(ctx.highlight_style(context_kind(i)).unwrap().clone());
    }
    styles.push(Handle::new(Prs3dDrawer::new()));
    flag(*ctx.current_viewer() == viewer);
    flag(!ctx.main_prs_mgr().is_null());
    flag(!ctx.selection_manager().is_null());
    flag(*ctx.main_selector() == *ctx.selection_manager().selector());
    flag(ctx.selection().is_empty());
    int(ctx.main_selector().pixel_tolerance());
    for style in &styles[1..7] {
        context_style(style, &styles[1]);
    }
    context_state(&ctx, &styles);
    for _ in 0..r.integer() {
        let (op, slot, value) = (r.integer(), r.integer(), r.integer());
        context_op(&ctx, &styles, op, slot, value);
        context_state(&ctx, &styles);
    }
    end();
}
