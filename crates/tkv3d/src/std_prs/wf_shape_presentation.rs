//! Native serial WF presentation sequencing; registered when the genuine isoline dependency lands.
use super::wf_shape_edges::{append_wf_edge_polylines, classified_wf_edges};
use super::{
    StdPrsIsolineDestination, StdPrsIsolinePolylines, StdPrsIsolines, StdPrsShapeTool,
    StdPrsToolTriangulatedShape, StdPrsWfShape,
};
use crate::prs3d::{Prs3d, Prs3dDrawer, Prs3dLineAspect};
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{self, Face, Shape},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkservice::graphic3d::*;
const WF_ISO_U: usize = 0;
const WF_ISO_V: usize = 1;
const WF_UNFREE: usize = 2;
const WF_FREE: usize = 3;
const WF_COMMON: usize = 4;
type WfPresentationPolylines = [StdPrsIsolinePolylines; 5];

impl StdPrsWfShape {
    /// Build the native default serial presentation from real topology; `Self::add(&prs,&shape,&drawer)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        if shape.is_null() {
            return Ok(());
        }
        if drawer.is_auto_triangulation() {
            StdPrsToolTriangulatedShape::tessellate(shape, drawer)?;
        }
        add_wf_mesh_free_edges(prs, shape, drawer)?;
        let deflection = StdPrsToolTriangulatedShape::get_deflection(shape, drawer)?;
        let mut polylines = std::array::from_fn(|_| StdPrsIsolinePolylines::default());
        append_wf_face_isolines(shape, drawer, deflection, &mut polylines)?;
        emit_wf_iso_groups(prs, drawer, &mut polylines)?;
        append_wf_boundary_groups(shape, drawer, deflection, &mut polylines)?;
        emit_wf_boundary_groups(prs, drawer, &mut polylines)?;
        add_wf_vertex_group(prs, shape, drawer)
    }
}
fn add_wf_mesh_free_edges(
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
    drawer: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    let edges = StdPrsWfShape::add_edges_on_triangulation(shape, true)?;
    if edges.is_null() {
        return Ok(());
    }
    let group = prs.new_group()?;
    group.set_primitives_aspect(&drawer.free_boundary_aspect().aspect().cast());
    group.add_primitive_array(&edges, true);
    Ok(())
}
fn add_wf_vertex_group(
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
    drawer: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    let vertices = StdPrsWfShape::add_vertexes(shape, drawer.vertex_draw_mode())?;
    if vertices.is_null() {
        return Ok(());
    }
    let group = prs.new_group()?;
    group.set_primitives_aspect(&drawer.point_aspect().aspect().cast());
    group.add_primitive_array(&vertices.cast(), true);
    Ok(())
}
fn wf_iso_routes(drawer: &Prs3dDrawer) -> (usize, usize) {
    let u = drawer.u_iso_aspect().aspect();
    let v = drawer.v_iso_aspect().aspect();
    let wire = drawer.wire_aspect().aspect();
    // Native establishes V->U before independently rerouting U/V into the common wire list.
    let mut v_route = if v.is_equal(&*u) { WF_ISO_U } else { WF_ISO_V };
    let u_route = if u.is_equal(&*wire) {
        WF_COMMON
    } else {
        WF_ISO_U
    };
    if v.is_equal(&*wire) {
        v_route = WF_COMMON;
    }
    (u_route, v_route)
}
fn append_wf_face_isolines(
    shape: &Shape,
    drawer: &Handle<Prs3dDrawer>,
    deflection: f64,
    polylines: &mut WfPresentationPolylines,
) -> Result<(), Failure> {
    let routes = wf_iso_routes(drawer);
    for entry in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = topods::face(&entry)?;
        if drawer.iso_on_plane() || !StdPrsShapeTool::is_planar_face(&face)? {
            append_routed_wf_isolines(&face, drawer, deflection, routes, polylines)?;
        }
    }
    Ok(())
}
fn append_routed_wf_isolines(
    face: &Face,
    drawer: &Handle<Prs3dDrawer>,
    deflection: f64,
    routes: (usize, usize),
    polylines: &mut WfPresentationPolylines,
) -> Result<(), Failure> {
    let destination = wf_isoline_destination(polylines, routes);
    StdPrsIsolines::add_native_polylines(face, drawer, deflection, destination)
}
fn wf_isoline_destination(
    polylines: &mut WfPresentationPolylines,
    routes: (usize, usize),
) -> StdPrsIsolineDestination<'_> {
    if routes.0 == routes.1 {
        return StdPrsIsolineDestination::Shared(&mut polylines[routes.0]);
    }
    let [u, v] = polylines
        .get_disjoint_mut([routes.0, routes.1])
        .expect("WF isoline routes are distinct indices within five native lists");
    StdPrsIsolineDestination::Distinct { u, v }
}

fn emit_wf_iso_groups(
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &Prs3dDrawer,
    polylines: &mut WfPresentationPolylines,
) -> Result<(), Failure> {
    Prs3d::add_primitives_group(prs, &drawer.u_iso_aspect().cast(), &mut polylines[WF_ISO_U])?;
    Prs3d::add_primitives_group(prs, &drawer.v_iso_aspect().cast(), &mut polylines[WF_ISO_V])
}
fn wf_boundary_route(
    enabled: bool,
    aspect: &Prs3dLineAspect,
    drawer: &Prs3dDrawer,
    own: usize,
) -> Option<usize> {
    if !enabled {
        return None;
    }
    let common = aspect.aspect().is_equal(&*drawer.wire_aspect().aspect());
    Some(if common { WF_COMMON } else { own })
}
fn append_wf_boundary_groups(
    shape: &Shape,
    drawer: &Handle<Prs3dDrawer>,
    deflection: f64,
    polylines: &mut WfPresentationPolylines,
) -> Result<(), Failure> {
    let routes = wf_boundary_routes(drawer);
    for (edges, route) in classified_wf_edges(shape)?.into_iter().zip(routes) {
        if let Some(route) = route {
            append_wf_edge_polylines(&edges, drawer, deflection, &mut polylines[route])?;
        }
    }
    Ok(())
}
fn emit_wf_boundary_groups(
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &Prs3dDrawer,
    polylines: &mut WfPresentationPolylines,
) -> Result<(), Failure> {
    Prs3d::add_primitives_group(
        prs,
        &drawer.un_free_boundary_aspect(),
        &mut polylines[WF_UNFREE],
    )?;
    Prs3d::add_primitives_group(prs, &drawer.free_boundary_aspect(), &mut polylines[WF_FREE])?;
    Prs3d::add_primitives_group(prs, &drawer.wire_aspect(), &mut polylines[WF_COMMON])
}

fn wf_boundary_routes(drawer: &Prs3dDrawer) -> [Option<usize>; 3] {
    [
        drawer.wire_draw().then_some(WF_COMMON),
        wf_boundary_route(
            drawer.free_boundary_draw(),
            &drawer.free_boundary_aspect(),
            drawer,
            WF_FREE,
        ),
        wf_boundary_route(
            drawer.un_free_boundary_draw(),
            &drawer.un_free_boundary_aspect(),
            drawer,
            WF_UNFREE,
        ),
    ]
}
