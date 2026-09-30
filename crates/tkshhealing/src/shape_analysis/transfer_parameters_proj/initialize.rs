//! Preserve OCC2851 edge tolerance and the native early-return state.
use super::*;
pub(super) fn initialize(
    tool: &ShapeAnalysisTransferParametersProj,
    edge: &Edge,
    face: &Face,
) -> Result<(), Failure> {
    tool.state.write().initialized = false;
    tool.parent.init(edge, face)?;
    tool.state.write().precision = BRepTool::edge_tolerance(edge)?;
    let (spatial, first, last) = BRepTool::curve(edge)?;
    tool.transfer_parameter_data().with_mut(|fields| {
        fields.first = first;
        fields.last = last;
    });
    tool.state.write().spatial = spatial.clone();
    if spatial.is_null() {
        tool.transfer_parameter_data().with_mut(|fields| {
            fields.first = 0.;
            fields.last = 1.;
        });
        return Ok(());
    }
    if face.is_null() {
        return Ok(());
    }
    initialize_planar(tool, edge, face)
}
fn initialize_planar(
    tool: &ShapeAnalysisTransferParametersProj,
    edge: &Edge,
    face: &Face,
) -> Result<(), Failure> {
    let (planar, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    tool.state.write().planar = planar.clone();
    if planar.is_null() {
        return Ok(());
    }
    let curve = Handle::new(Geom2dAdaptorCurve::with_range(&planar, first, last)?).cast();
    let (surface, location) = BRepTool::surface_with_location(face)?;
    tool.state.write().location = location;
    let surface = Handle::new(GeomAdaptorSurface::with_surface(&surface)?).cast();
    let adaptor = Handle::new(Adaptor3dCurveOnSurface::with_curve(&curve, &surface)?);
    let mut state = tool.state.write();
    state.adaptor = adaptor;
    state.initialized = true;
    Ok(())
}
