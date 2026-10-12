//! Native shaded fallback compounds preserve isolated element order and temporary iso counts.
use super::{StdPrsShadedShape, StdPrsWfShape};
use crate::prs3d::{Prs3dDrawer, Prs3dVertexDrawMode};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{self, Compound, Shape, TopoDSShapeReadSource},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;

impl StdPrsShadedShape {
    /// Draw genuine isolated edges/vertices through native WF sequencing; `Self::add_wireframe_for_free_elements(&prs,&shape,&drawer)?`.
    pub fn add_wireframe_for_free_elements(
        prs: &Handle<dyn Graphic3dStructureSource>,
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        Self::add_free_elements_reference(prs, shape, drawer)
    }
    /// Draw actual unmeshed faces with native temporary U/V count5; `Self::add_wireframe_for_faces_without_triangles(&prs,&shape,&drawer)?`.
    pub fn add_wireframe_for_faces_without_triangles(
        prs: &Handle<dyn Graphic3dStructureSource>,
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        Self::add_unmeshed_faces_reference(prs, shape, drawer)
    }
    /// Reread a live native source before free-element collection; `Self::add_free_elements_reference(&prs,&reference,&drawer)?`.
    pub fn add_free_elements_reference<S: TopoDSShapeReadSource>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        source: &S,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let has_faces = source.read_current_shape(|shape| {
            Ok(Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?.more())
        })?;
        if !has_faces {
            return StdPrsWfShape::add_reference(prs, source, drawer);
        }
        let all_vertices = drawer.vertex_draw_mode() == Prs3dVertexDrawMode::All;
        if !all_vertices
            && source.read_current_shape(|shape| shape.shape_type())? != ShapeEnum::Compound
        {
            return Ok(());
        }
        let free = source
            .read_current_shape(|shape| shaded_free_elements_compound(shape, all_vertices))?;
        if free.nb_children() > 0 {
            StdPrsWfShape::add(prs, &free, drawer)?;
        }
        Ok(())
    }
    /// Collect current unmeshed faces before graphics callbacks; `Self::add_unmeshed_faces_reference(&prs,&reference,&drawer)?`.
    pub fn add_unmeshed_faces_reference<S: TopoDSShapeReadSource>(
        prs: &Handle<dyn Graphic3dStructureSource>,
        source: &S,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let unmeshed = source.read_current_shape(shaded_unmeshed_faces_compound)?;
        if unmeshed.nb_children() == 0 {
            return Ok(());
        }
        let previous = set_unmeshed_shaded_iso_counts(drawer);
        // OCCT 3d097a0 restores only after success, so Failure deliberately leaves counts5.
        StdPrsWfShape::add(prs, &unmeshed, drawer)?;
        restore_unmeshed_shaded_iso_counts(drawer, previous);
        Ok(())
    }
}
fn shaded_free_elements_compound(shape: &Shape, all_vertices: bool) -> Result<Compound, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Face)? {
        builder.add(&mut compound, &edge)?;
    }
    let avoid = if all_vertices {
        ShapeEnum::Shape
    } else {
        ShapeEnum::Edge
    };
    for vertex in Explorer::new(shape, ShapeEnum::Vertex, avoid)? {
        builder.add(&mut compound, &vertex)?;
    }
    Ok(compound)
}
fn shaded_unmeshed_faces_compound(shape: &Shape) -> Result<Compound, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for entry in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = topods::face(&entry)?;
        if BRepTool::triangulation(&face, mesh_purpose::NONE)?
            .0
            .is_null()
        {
            builder.add(&mut compound, &face)?;
        }
    }
    Ok(compound)
}

fn set_unmeshed_shaded_iso_counts(drawer: &Prs3dDrawer) -> [i32; 2] {
    let previous = [
        drawer.u_iso_aspect().number(),
        drawer.v_iso_aspect().number(),
    ];
    drawer.u_iso_aspect().set_number(5);
    drawer.v_iso_aspect().set_number(5);
    previous
}
fn restore_unmeshed_shaded_iso_counts(drawer: &Prs3dDrawer, previous: [i32; 2]) {
    // Native re-reads the slots after virtual WF callbacks; retained handles could now differ.
    drawer.u_iso_aspect().set_number(previous[0]);
    drawer.v_iso_aspect().set_number(previous[1]);
}
