//! Native shaded Add orchestration preserves actual WF fallbacks and ordered face boundaries.
use super::shaded_shape_triangles::ShadedTexelParameters;
use super::{StdPrsShadedShape, StdPrsToolTriangulatedShape, StdPrsVolume};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point2;
use rustycad_tkservice::graphic3d::*;
struct ShadedPresentationContext<'a> {
    prs: &'a Handle<dyn Graphic3dStructureSource>,
    drawer: &'a Handle<Prs3dDrawer>,
    group: &'a Handle<dyn Graphic3dGroup>,
    texels: ShadedTexelParameters,
}
type ShadedPresentationResources<'a> = (
    &'a Handle<dyn Graphic3dStructureSource>,
    &'a Handle<Prs3dDrawer>,
    &'a Handle<dyn Graphic3dGroup>,
);
impl<'a> ShadedPresentationContext<'a> {
    fn from_arguments(
        resources: ShadedPresentationResources<'a>,
        uv: (bool, Point2, Point2, Point2),
    ) -> Self {
        let (prs, drawer, group) = resources;
        let (enabled, origin, repeat, scale) = uv;
        let texels = ShadedTexelParameters {
            enabled,
            origin,
            repeat,
            scale,
        };
        Self {
            prs,
            drawer,
            group,
            texels,
        }
    }
}
impl StdPrsShadedShape {
    /// Build native shaded groups without UVs; `Self::add(&prs,&shape,&drawer,StdPrsVolume::Autodetection,&Handle::default())?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
        volume: StdPrsVolume,
        group: &Handle<dyn Graphic3dGroup>,
    ) -> Result<(), Failure> {
        let dummy = Point2::default();
        Self::add_with_texels(
            prs, shape, drawer, false, dummy, dummy, dummy, volume, group,
        )
    }
    /// Preserve the full native UV and optional group overload; `Self::add_with_texels(&prs,&shape,&drawer,true,origin,repeat,scale,volume,&group)?`.
    #[allow(clippy::too_many_arguments)] // Pinned OCCT 3d097a0 overload carries the UV triple and grouping policy.
    pub fn add_with_texels(
        prs: &Handle<dyn Graphic3dStructureSource>,
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
        enabled: bool,
        origin: Point2,
        repeat: Point2,
        scale: Point2,
        volume: StdPrsVolume,
        group: &Handle<dyn Graphic3dGroup>,
    ) -> Result<(), Failure> {
        let resources = (prs, drawer, group);
        let context =
            ShadedPresentationContext::from_arguments(resources, (enabled, origin, repeat, scale));
        add_shaded_presentation(shape, volume, &context)
    }
}
fn add_shaded_presentation(
    shape: &Shape,
    volume: StdPrsVolume,
    context: &ShadedPresentationContext<'_>,
) -> Result<(), Failure> {
    if shape.is_null() {
        return Ok(());
    }
    if context.drawer.is_auto_triangulation() {
        StdPrsToolTriangulatedShape::tessellate(shape, context.drawer)?;
    }
    StdPrsShadedShape::add_wireframe_for_free_elements(context.prs, shape, context.drawer)?;
    StdPrsShadedShape::add_wireframe_for_faces_without_triangles(
        context.prs,
        shape,
        context.drawer,
    )?;
    add_shaded_volume_groups(shape, volume, context)?;
    add_shaded_face_boundary_group(shape, context)
}
fn shade_native_shape(
    shape: &Shape,
    closed: bool,
    context: &ShadedPresentationContext<'_>,
) -> Result<(), Failure> {
    let uv = context.texels;
    let triangles = StdPrsShadedShape::fill_triangles_with_texels(
        shape, uv.enabled, uv.origin, uv.repeat, uv.scale,
    )?;
    if triangles.is_null() {
        return Ok(());
    }
    let group = selected_shaded_group(context)?;
    group.set_closed(closed);
    group.set_group_primitives_aspect(&context.drawer.shading_aspect().aspect().cast());
    group.add_primitive_array(&triangles.cast(), true);
    Ok(())
}
fn selected_shaded_group(
    context: &ShadedPresentationContext<'_>,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    if context.group.is_null() {
        return context.prs.new_group();
    }
    Ok(context.group.clone())
}
fn add_shaded_volume_groups(
    shape: &Shape,
    volume: StdPrsVolume,
    context: &ShadedPresentationContext<'_>,
) -> Result<(), Failure> {
    let kind = shape.shape_type()?;
    let collect = matches!(
        kind,
        ShapeEnum::Compound | ShapeEnum::CompSolid | ShapeEnum::Solid
    );
    if volume != StdPrsVolume::Autodetection || !collect {
        return shade_native_shape(shape, volume == StdPrsVolume::Closed, context);
    }
    let (closed, opened) = shaded_volume_compounds(shape)?;
    shade_native_volume_compound(&closed, true, context)?;
    shade_native_volume_compound(&opened, false, context)
}
fn shade_native_volume_compound(
    compound: &Compound,
    closed: bool,
    context: &ShadedPresentationContext<'_>,
) -> Result<(), Failure> {
    if compound.nb_children() == 0 {
        return Ok(());
    }
    shade_native_shape(compound, closed, context)
}
fn add_shaded_face_boundary_group(
    shape: &Shape,
    context: &ShadedPresentationContext<'_>,
) -> Result<(), Failure> {
    if !context.drawer.face_boundary_draw() {
        return Ok(());
    }
    let lines = StdPrsShadedShape::fill_face_boundaries_with_continuity(
        shape,
        context.drawer.face_boundary_upper_continuity(),
    )?;
    if lines.is_null() {
        return Ok(());
    }
    let group = selected_shaded_group(context)?;
    group.set_group_primitives_aspect(&context.drawer.face_boundary_aspect().aspect().cast());
    group.add_primitive_array(&lines.cast(), true);
    Ok(())
}

fn shaded_volume_compounds(shape: &Shape) -> Result<(Compound, Compound), Failure> {
    let builder = BRepBuilder::default();
    let (mut closed, mut opened) = (Compound::default(), Compound::default());
    builder.make_compound(&mut closed);
    builder.make_compound(&mut opened);
    StdPrsShadedShape::explore_solids(shape, &builder, &mut closed, &mut opened, true)?;
    Ok((closed, opened))
}
