//! Native face dispatch chooses triangulation only when every explored face has a mesh.
use super::{
    StdPrsIsolineDestination, StdPrsIsolinePolylines, StdPrsIsolines,
    isoline_triangle_api::add_isoline_groups,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{Face, face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
impl StdPrsIsolines {
    /// Select the real surface or mesh path with caller-owned lists; `StdPrsIsolines::add_native_polylines(&face,&drawer,0.1,output)?`.
    pub fn add_native_polylines(
        face: &Face,
        drawer: &Handle<Prs3dDrawer>,
        deflection: f64,
        output: StdPrsIsolineDestination<'_>,
    ) -> Result<(), Failure> {
        if drawer.iso_on_triangulation() && isoline_face_has_triangulation(face)? {
            return Self::add_on_triangulation_polylines(face, drawer, output);
        }
        Self::add_on_surface_polylines(face, drawer, deflection, output)
    }
    /// Select the native builder and append U then V presentation groups; `StdPrsIsolines::add(&prs,&face,&drawer,0.1)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Face,
        drawer: &Handle<Prs3dDrawer>,
        deflection: f64,
    ) -> Result<(), Failure> {
        let [mut u, mut v] = super::isoline_outputs::empty_isoline_lists();
        Self::add_native_polylines(
            face,
            drawer,
            deflection,
            StdPrsIsolineDestination::distinct(&mut u, &mut v),
        )?;
        add_isoline_groups(prs, drawer, &mut u, &mut v)
    }
    /// Append copied convenience polylines while retaining native partial failures; `StdPrsIsolines::add_polylines(&face,&drawer,0.1,&mut u,&mut v)?`.
    pub fn add_polylines(
        face: &Face,
        drawer: &Handle<Prs3dDrawer>,
        deflection: f64,
        u: &mut Vec<Vec<Point3>>,
        v: &mut Vec<Vec<Point3>>,
    ) -> Result<(), Failure> {
        let [mut u_handles, mut v_handles] = super::isoline_outputs::empty_isoline_lists();
        let result = Self::add_native_polylines(
            face,
            drawer,
            deflection,
            StdPrsIsolineDestination::distinct(&mut u_handles, &mut v_handles),
        );
        copy_isoline_polylines(&u_handles, u);
        copy_isoline_polylines(&v_handles, v);
        result
    }
}
fn isoline_face_has_triangulation(input: &Face) -> Result<bool, Failure> {
    let mut explorer = Explorer::new(input, ShapeEnum::Face, ShapeEnum::Shape)?;
    while explorer.more() {
        let (mesh, _) = BRepTool::triangulation(&face(explorer.current())?, 0)?;
        if mesh.is_null() {
            return Ok(false);
        }
        explorer.next()?;
    }
    Ok(true)
}
fn copy_isoline_polylines(input: &StdPrsIsolinePolylines, output: &mut Vec<Vec<Point3>>) {
    for points in input.iter() {
        output.push(points.sequence(|sequence| sequence.iter().copied().collect()));
    }
}
