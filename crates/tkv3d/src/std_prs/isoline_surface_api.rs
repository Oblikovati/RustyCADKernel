//! Surface overloads retain original parameter lists and submit native U/V group policy.
use super::StdPrsIsolineSurfaceSource;
use super::isoline_surface_reference::IsolineSurfaceReference;
use super::{
    StdPrsIsolineDestination, StdPrsIsolines, isoline_parameters::face_isoline_parameters,
    isoline_surface_collect::collect_surface_isolines, isoline_triangle_api::add_isoline_groups,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
impl StdPrsIsolines {
    /// Append trimmed native surface isolines; `StdPrsIsolines::add_on_surface_polylines(&face,&drawer,0.1,output)?`.
    pub fn add_on_surface_polylines(
        face: &Face,
        drawer: &Handle<Prs3dDrawer>,
        deflection: f64,
        mut output: StdPrsIsolineDestination<'_>,
    ) -> Result<(), Failure> {
        let Some((surface, parameters)) = face_surface_isoline_inputs(face, drawer)? else {
            return Ok(());
        };
        let reference = IsolineSurfaceReference::Borrowed(&surface);
        collect_surface_isolines(
            &reference,
            drawer,
            deflection,
            parameters.each_ref(),
            &mut output,
        );
        Ok(())
    }
    /// Add separate U/V native surface groups; `StdPrsIsolines::add_on_surface(&prs,&face,&drawer,0.1)?`.
    pub fn add_on_surface(
        prs: &Handle<dyn Graphic3dStructureSource>,
        face: &Face,
        drawer: &Handle<Prs3dDrawer>,
        deflection: f64,
    ) -> Result<(), Failure> {
        let [mut u, mut v] = super::isoline_outputs::empty_isoline_lists();
        Self::add_on_surface_polylines(
            face,
            drawer,
            deflection,
            StdPrsIsolineDestination::distinct(&mut u, &mut v),
        )?;
        add_isoline_groups(prs, drawer, &mut u, &mut v)
    }
    /// Sample explicit parameters against the actual face adaptor; `StdPrsIsolines::add_on_surface_parameters(&prs,&surface,&drawer,0.1,parameters)?`.
    pub fn add_on_surface_parameters<
        S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
    >(
        prs: &Handle<dyn Graphic3dStructureSource>,
        surface: &Handle<S>,
        drawer: &Handle<Prs3dDrawer>,
        deflection: f64,
        parameters: [&SharedState<Sequence<f64>>; 2],
    ) -> Result<(), Failure> {
        let reference = IsolineSurfaceReference::Borrowed(surface);
        add_surface_reference_groups(prs, &reference, drawer, deflection, parameters)
    }
    /// Preserve a native live handle slot during callbacks; `StdPrsIsolines::add_on_surface_references(&prs,&surface_slot,&drawer,0.1,parameters)?`.
    pub fn add_on_surface_references<
        S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
    >(
        prs: &Handle<dyn Graphic3dStructureSource>,
        surface: &SharedState<Handle<S>>,
        drawer: &Handle<Prs3dDrawer>,
        deflection: f64,
        parameters: [&SharedState<Sequence<f64>>; 2],
    ) -> Result<(), Failure> {
        let reference = IsolineSurfaceReference::Live(surface);
        add_surface_reference_groups(prs, &reference, drawer, deflection, parameters)
    }
}

type FaceSurfaceIsolineInputs = (Handle<BRepAdaptorSurface>, [SharedState<Sequence<f64>>; 2]);
fn face_surface_isoline_inputs(
    face: &Face,
    drawer: &Prs3dDrawer,
) -> Result<Option<FaceSurfaceIsolineInputs>, Failure> {
    let counts = [
        drawer.u_iso_aspect().number(),
        drawer.v_iso_aspect().number(),
    ];
    if counts[0] < 1 && counts[1] < 1 {
        return Ok(None);
    }
    // Parameter generation and adaptor construction are outside native sampling's suppression boundary.
    let (parameters, _) = face_isoline_parameters(face, counts, drawer.maximal_parameter_value())?;
    let surface = Handle::new(BRepAdaptorSurface::with_face(face, true)?);
    Ok(Some((surface, parameters.map(SharedState::new))))
}

fn add_surface_reference_groups<
    S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
>(
    prs: &Handle<dyn Graphic3dStructureSource>,
    reference: &IsolineSurfaceReference<'_, S>,
    drawer: &Prs3dDrawer,
    deflection: f64,
    parameters: [&SharedState<Sequence<f64>>; 2],
) -> Result<(), Failure> {
    let [mut u, mut v] = super::isoline_outputs::empty_isoline_lists();
    let mut output = StdPrsIsolineDestination::distinct(&mut u, &mut v);
    collect_surface_isolines(reference, drawer, deflection, parameters, &mut output);
    add_isoline_groups(prs, drawer, &mut u, &mut v)
}
