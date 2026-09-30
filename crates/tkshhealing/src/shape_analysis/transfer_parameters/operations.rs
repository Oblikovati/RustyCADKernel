//! Initialization order and range arithmetic follow the pinned native implementation.
use super::*;
pub(super) fn initialize(
    state: &TransferParameterData,
    edge: &Edge,
    face: &Face,
) -> Result<(), Failure> {
    state.with_mut(|fields| {
        fields.scale = 1.;
        fields.shift = 0.;
        fields.edge = edge.clone();
    });
    let has_spatial = initialize_spatial(state, edge)?;
    let (planar, first_2d, last_2d) = if face.is_null() {
        (Handle::default(), 0., 0.)
    } else {
        ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?
    };
    set_coefficients(
        state,
        face,
        first_2d,
        last_2d,
        has_spatial && !planar.is_null(),
    );
    Ok(())
}
fn initialize_spatial(state: &TransferParameterData, edge: &Edge) -> Result<bool, Failure> {
    let (spatial, first, last) = ShapeAnalysisEdge::new().curve_3d(edge, false)?;
    state.with_mut(|fields| {
        fields.first = first;
        fields.last = last;
    });
    Ok(!spatial.is_null())
}
fn set_coefficients(
    state: &TransferParameterData,
    face: &Face,
    first_2d: f64,
    last_2d: f64,
    has_curves: bool,
) {
    state.with_mut(|fields| {
        fields.first_2d = first_2d;
        fields.last_2d = last_2d;
        fields.face = face.clone();
        if !has_curves {
            return;
        }
        let span = fields.last - fields.first;
        fields.scale = if span <= resolution() {
            1.
        } else {
            (last_2d - first_2d) / span
        };
        fields.shift = first_2d - fields.first * fields.scale;
    });
}

pub(super) fn transfer_sequence<T: TransferParameters + ?Sized>(
    tool: &T,
    parameters: &Handle<HSequence<f64>>,
    to_2d: bool,
) -> Result<Handle<HSequence<f64>>, Failure> {
    let Some(sequence) = parameters.get() else {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "ShapeAnalysis_TransferParameters: null parameters; expected HSequence<double>",
            "",
        ));
    };
    let values = sequence.sequence(|sequence| sequence.iter().copied().collect::<Vec<_>>());
    let result = Handle::new(HSequence::<f64>::default());
    for value in values {
        let transferred = tool.perform_parameter(value, to_2d)?;
        result.change_sequence(|sequence| {
            sequence.append(transferred);
        });
    }
    Ok(result)
}
pub(super) fn range_fractions(
    state: &TransferParameterData,
    previous: f64,
    current: f64,
    is_2d: bool,
) -> (Edge, f64, f64) {
    state.with(|fields| {
        let (low, high, first, last) = if is_2d {
            planar_bounds(fields, previous, current)
        } else {
            (previous, current, fields.first, fields.last)
        };
        let span = last - first;
        (
            fields.edge.clone(),
            (low - first) / span,
            (high - first) / span,
        )
    })
}
fn planar_bounds(
    fields: &TransferParameterFields,
    previous: f64,
    current: f64,
) -> (f64, f64, f64, f64) {
    // Only the native pcurve-driven overload orders its bounds.
    let (low, high) = if previous > current {
        (current, previous)
    } else {
        (previous, current)
    };
    (low, high, fields.first_2d, fields.last_2d)
}
