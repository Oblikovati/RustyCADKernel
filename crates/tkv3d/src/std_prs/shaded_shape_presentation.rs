//! Native shaded Add orchestration preserves actual WF fallbacks and ordered face boundaries.
use super::shaded_shape_triangles::ShadedTexelParameters;
use super::{StdPrsShadedShape, StdPrsToolTriangulatedShape, StdPrsVolume};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape, TopoDSShapeReadSource},
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
    /// Read the current borrowed shape between native graphics callbacks; `Self::add_reference(&prs,&source,&drawer,StdPrsVolume::Autodetection,&Handle::default())?`.
    pub fn add_reference<S: TopoDSShapeReadSource>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        source: &S,
        drawer: &Handle<Prs3dDrawer>,
        volume: StdPrsVolume,
        group: &Handle<dyn Graphic3dGroup>,
    ) -> Result<(), Failure> {
        let dummy = Point2::default();
        let context = ShadedPresentationContext::from_arguments(
            (prs, drawer, group),
            (false, dummy, dummy, dummy),
        );
        add_shaded_presentation(source, volume, &context)
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
fn add_shaded_presentation<S: TopoDSShapeReadSource>(
    source: &S,
    volume: StdPrsVolume,
    context: &ShadedPresentationContext<'_>,
) -> Result<(), Failure> {
    if source.read_current_shape(|shape| Ok(shape.is_null()))? {
        return Ok(());
    }
    if context.drawer.is_auto_triangulation() {
        source.read_current_shape(|shape| {
            StdPrsToolTriangulatedShape::tessellate(shape, context.drawer)
        })?;
    }
    StdPrsShadedShape::add_free_elements_reference(context.prs, source, context.drawer)?;
    StdPrsShadedShape::add_unmeshed_faces_reference(context.prs, source, context.drawer)?;
    add_shaded_volume_groups(source, volume, context)?;
    add_shaded_face_boundary_group(source, context)
}
fn shade_native_shape<S: TopoDSShapeReadSource>(
    source: &S,
    closed: bool,
    context: &ShadedPresentationContext<'_>,
) -> Result<(), Failure> {
    let uv = context.texels;
    let triangles = source.read_current_shape(|shape| {
        StdPrsShadedShape::fill_triangles_with_texels(
            shape, uv.enabled, uv.origin, uv.repeat, uv.scale,
        )
    })?;
    if triangles.is_null() {
        return Ok(());
    }
    let group = selected_shaded_group(context)?;
    group.set_closed(closed);
    group.set_group_primitives_aspect(
        &context
            .drawer
            .shading_aspect()
            .aspect_ref()
            .observe()
            .observe_as::<dyn Graphic3dAspectsSource>(),
    );
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
fn add_shaded_volume_groups<S: TopoDSShapeReadSource>(
    shape: &S,
    volume: StdPrsVolume,
    context: &ShadedPresentationContext<'_>,
) -> Result<(), Failure> {
    let kind = shape.read_current_shape(|current| current.shape_type())?;
    let collect = matches!(
        kind,
        ShapeEnum::Compound | ShapeEnum::CompSolid | ShapeEnum::Solid
    );
    if volume != StdPrsVolume::Autodetection || !collect {
        return shade_native_shape(shape, volume == StdPrsVolume::Closed, context);
    }
    let (closed, opened) = shape.read_current_shape(shaded_volume_compounds)?;
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
    shade_native_shape(&**compound, closed, context)
}
fn add_shaded_face_boundary_group<S: TopoDSShapeReadSource>(
    shape: &S,
    context: &ShadedPresentationContext<'_>,
) -> Result<(), Failure> {
    if !context.drawer.face_boundary_draw() {
        return Ok(());
    }
    let lines = shape.read_current_shape(|current| {
        StdPrsShadedShape::fill_face_boundaries_with_continuity(
            current,
            context.drawer.face_boundary_upper_continuity(),
        )
    })?;
    if lines.is_null() {
        return Ok(());
    }
    let group = selected_shaded_group(context)?;
    group.set_group_primitives_aspect(
        &context
            .drawer
            .face_boundary_aspect()
            .aspect_ref()
            .observe()
            .observe_as::<dyn Graphic3dAspectsSource>(),
    );
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
