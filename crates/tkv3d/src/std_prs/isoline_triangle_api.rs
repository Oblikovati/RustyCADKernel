//! Public native triangulation overloads preserve ordered output and separate U/V presentation groups.
use super::{
    StdPrsIsolineDestination, StdPrsIsolinePolylines, StdPrsIsolineTriangulationInputs,
    StdPrsIsolines, isoline_triangle_collect::collect_triangle_isolines,
    isoline_triangle_face::face_triangulation_isolines,
};
use crate::prs3d::{Prs3d, Prs3dDrawer};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle, catch_native_failure};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
impl StdPrsIsolines {
    /// Append genuine mesh crossings, retaining aliases; `StdPrsIsolines::add_on_triangulation_polylines(&face,&drawer,output)?`.
    pub fn add_on_triangulation_polylines(
        face: &Face,
        drawer: &Handle<Prs3dDrawer>,
        mut output: StdPrsIsolineDestination<'_>,
    ) -> Result<(), Failure> {
        // Surface callback failures propagate on this path; only the surface-sampling overload suppresses them.
        catch_native_failure(|| face_triangulation_isolines(face, drawer, &mut output))
    }
    /// Add separate native U/V groups for a face; `StdPrsIsolines::add_on_triangulation(&prs,&face,&drawer)?`.
    pub fn add_on_triangulation(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Face,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let [mut u, mut v] = super::isoline_outputs::empty_isoline_lists();
        Self::add_on_triangulation_polylines(
            face,
            drawer,
            StdPrsIsolineDestination::distinct(&mut u, &mut v),
        )?;
        add_isoline_groups(prs, drawer, &mut u, &mut v)
    }
    /// Evaluate explicit live native inputs before submitting groups; `StdPrsIsolines::add_on_triangulation_parameters(&prs,&drawer,inputs)?`.
    pub fn add_on_triangulation_parameters(
        prs: &Handle<dyn Graphic3dStructureSource>,
        drawer: &Handle<Prs3dDrawer>,
        inputs: StdPrsIsolineTriangulationInputs<'_>,
    ) -> Result<(), Failure> {
        let [mut u, mut v] = super::isoline_outputs::empty_isoline_lists();
        let mut output = StdPrsIsolineDestination::distinct(&mut u, &mut v);
        catch_native_failure(|| collect_triangle_isolines(&inputs, &mut output))?;
        add_isoline_groups(prs, drawer, &mut u, &mut v)
    }
}
pub(super) fn add_isoline_groups(
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &Prs3dDrawer,
    u: &mut StdPrsIsolinePolylines,
    v: &mut StdPrsIsolinePolylines,
) -> Result<(), Failure> {
    Prs3d::add_primitives_group(prs, &drawer.u_iso_aspect().cast(), u)?;
    Prs3d::add_primitives_group(prs, &drawer.v_iso_aspect().cast(), v)?;
    Ok(())
}
