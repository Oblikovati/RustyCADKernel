//! Native constructor style ownership, including shared plane-edge aspects.
use crate::prs3d::*;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::{
    aspect::{
        AspectTypeOfFacingModel, AspectTypeOfHighlightMethod, AspectTypeOfLine, AspectTypeOfMarker,
    },
    graphic3d::{
        GRAPHIC3D_ZLAYER_ID_DEFAULT, GRAPHIC3D_ZLAYER_ID_TOP, GRAPHIC3D_ZLAYER_ID_TOPMOST,
        GRAPHIC3D_ZLAYER_ID_UNKNOWN, Graphic3dMaterialAspect, Graphic3dNameOfMaterial,
        Graphic3dPresentationAttributesSource,
    },
};
pub(super) fn context_default_styles() -> Result<[Handle<Prs3dDrawer>; 6], Failure> {
    let styles = std::array::from_fn(|_| Handle::new(Prs3dDrawer::new()));
    styles[0].setup_own_defaults()?;
    styles[0].set_z_layer(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    styles[0].set_display_mode(0);
    for (index, color, layer) in [
        (2, NamedColor::CYAN1, GRAPHIC3D_ZLAYER_ID_TOP),
        (4, NamedColor::CYAN1, GRAPHIC3D_ZLAYER_ID_TOPMOST),
        (1, NamedColor::GRAY80, GRAPHIC3D_ZLAYER_ID_UNKNOWN),
        (3, NamedColor::GRAY80, GRAPHIC3D_ZLAYER_ID_UNKNOWN),
    ] {
        styles[index].link_to(&styles[0]);
        initialize_highlight_style(&styles[index], Color::named(color))?;
        styles[index].set_z_layer(layer);
    }
    styles[5].set_z_layer(GRAPHIC3D_ZLAYER_ID_UNKNOWN);
    styles[5].set_method(AspectTypeOfHighlightMethod::Color);
    styles[5].set_color(Color::named(NamedColor::GRAY40));
    initialize_context_attributes(&styles[0]);
    Ok(styles)
}
fn initialize_highlight_style(style: &Prs3dDrawer, color: Color) -> Result<(), Failure> {
    style.set_method(AspectTypeOfHighlightMethod::Color);
    style.set_display_mode(0);
    style.set_color(color);
    style.setup_own_shading_aspect(&Handle::default())?;
    style.setup_own_point_aspect(&Handle::default())?;
    copy_highlight_lines(style)?;
    style.set_plane_aspect(Handle::new(Prs3dPlaneAspect::new()?));
    // OCCT 3d097a0 assigns Prs3d_LineAspect here, sharing its Graphic3d handle.
    style
        .plane_aspect()
        .edges_aspect()
        .set_aspect(style.link().plane_aspect().edges_aspect().aspect());
    style.set_datum_aspect(Handle::new(Prs3dDatumAspect::new()?));
    color_highlight_style(style, color)?;
    for line in highlight_lines(style) {
        line.set_width(2.)?;
    }
    style
        .point_aspect()
        .set_type_of_marker(AspectTypeOfMarker::OPoint);
    style.point_aspect().set_scale(2.)?;
    style.set_auto_triangulation(false);
    Ok(())
}
fn copy_highlight_lines(style: &Prs3dDrawer) -> Result<(), Failure> {
    let source = style.link();
    for (setter, getter) in [
        (
            Prs3dDrawer::set_line_aspect as fn(&Prs3dDrawer, Handle<Prs3dLineAspect>),
            Prs3dDrawer::line_aspect as fn(&Prs3dDrawer) -> Handle<Prs3dLineAspect>,
        ),
        (Prs3dDrawer::set_wire_aspect, Prs3dDrawer::wire_aspect),
        (
            Prs3dDrawer::set_free_boundary_aspect,
            Prs3dDrawer::free_boundary_aspect,
        ),
        (
            Prs3dDrawer::set_un_free_boundary_aspect,
            Prs3dDrawer::un_free_boundary_aspect,
        ),
    ] {
        let own = Handle::new(Prs3dLineAspect::new(
            Color::named(NamedColor::BLACK),
            AspectTypeOfLine::Solid,
            1.,
        )?);
        prs3d_copy_aspects(&*own.aspect(), &*getter(&source).aspect());
        setter(style, own);
    }
    Ok(())
}
fn highlight_lines(style: &Prs3dDrawer) -> [Handle<Prs3dLineAspect>; 5] {
    [
        style.wire_aspect(),
        style.line_aspect(),
        style.plane_aspect().edges_aspect(),
        style.free_boundary_aspect(),
        style.un_free_boundary_aspect(),
    ]
}
fn color_highlight_style(style: &Prs3dDrawer, color: Color) -> Result<(), Failure> {
    style
        .shading_aspect()
        .set_color(color, AspectTypeOfFacingModel::BothSide)?;
    for line in highlight_lines(style) {
        line.set_color(color);
    }
    style.plane_aspect().arrow_aspect().set_color(color);
    style.plane_aspect().iso_aspect().set_color(color);
    style.point_aspect().set_color(color);
    for part in Prs3dDatumParts::DRAWN {
        if let Some(line) = style.datum_aspect().line_aspect(part).get() {
            line.set_color(color);
        }
    }
    Ok(())
}
pub(super) fn initialize_context_attributes(drawer: &Prs3dDrawer) {
    drawer.shading_aspect().set_material(
        &Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Brass),
        AspectTypeOfFacingModel::BothSide,
    );
    let hidden = drawer.hidden_line_aspect();
    hidden.set_color(Color::named(NamedColor::GRAY20));
    hidden
        .set_width(1.)
        .expect("native context width 1 must be positive");
    hidden.set_type_of_line(AspectTypeOfLine::Dash);
    let datum = drawer.datum_aspect();
    datum.set_axis_length(100., 100., 100.);
    for part in [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ] {
        datum
            .line_aspect(part)
            .set_color(Color::named(NamedColor::LIGHTSTEELBLUE4));
    }
    let plane = drawer.plane_aspect();
    plane.set_plane_length(200., 200.);
    plane
        .edges_aspect()
        .set_color(Color::named(NamedColor::SKYBLUE));
}
