// Rust translation of BRepMesh_NURBSRangeSplitter.hxx/.cxx (AdjustRange, initParameters,
// the interval helpers, grabParamsOfEdges) and of the BoundaryParams, Extrusion and
// Undefined splitters at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::nurbs_filter::generate_nurbs_nodes;
use super::uv_param::IMapOfReal;
use super::*;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkgeombase::geom_lib::norm_estim;
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::Direction3;

/// `BRepMesh_NURBSRangeSplitter`: interior nodes on a grid of filtered continuity
/// interval and boundary parameters; `NURBSRangeSplitter::default()`.
pub struct NURBSRangeSplitter {
    base: UVParamRangeSplitter,
    surface_type: SurfaceType,
}
impl Default for NURBSRangeSplitter {
    fn default() -> Self {
        Self {
            base: UVParamRangeSplitter::default(),
            surface_type: SurfaceType::OtherSurface,
        }
    }
}
impl std::ops::Deref for NURBSRangeSplitter {
    type Target = UVParamRangeSplitter;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for NURBSRangeSplitter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl NURBSRangeSplitter {
    /// `mySurfaceType`, the surface type recorded by the last AdjustRange;
    /// `splitter.surface_type()`.
    pub fn surface_type(&self) -> SurfaceType {
        self.surface_type
    }
}

/// The virtual members of the NURBS splitter family (`initParameters`,
/// `getUndefinedIntervalNb`) over its shared state; the NURBS behavior of
/// AdjustRange and GenerateSurfaceNodes is in [`nurbs_adjust_range`] and
/// [`nurbs_generate_surface_nodes`].
pub trait NurbsRangeSplitterHooks: RangeSplitter {
    /// The NURBS splitter state; `splitter.nurbs_state().surface_type()`.
    fn nurbs_state(&self) -> &NURBSRangeSplitter;
    /// The NURBS splitter state, mutable; `splitter.nurbs_state_mut()`.
    fn nurbs_state_mut(&mut self) -> &mut NURBSRangeSplitter;
    /// `initParameters`: the U and V parameters from the CN intervals and the internal
    /// edges; false when a direction gets no parameter; `splitter.init_parameters()?`.
    fn init_parameters(&mut self) -> Result<bool, Failure> {
        nurbs_init_parameters(self)
    }
    /// `getUndefinedIntervalNb`: the number of poles less one;
    /// `splitter.get_undefined_interval_nb(&surface, true, Shape::CN)?`.
    fn get_undefined_interval_nb(
        &self,
        surface: &BRepAdaptorSurface,
        is_u: bool,
        _continuity: Shape,
    ) -> Result<i32, Failure> {
        let poles = if is_u {
            surface.nb_u_poles()?
        } else {
            surface.nb_v_poles()?
        };
        Ok(poles - 1)
    }
}

/// `AdjustRange`: the default adjustment, then the surface type; a Bezier surface is
/// valid only for a range within [-0.5, 1.5] in both directions.
pub fn nurbs_adjust_range<T: NurbsRangeSplitterHooks + ?Sized>(
    splitter: &mut T,
) -> Result<(), Failure> {
    default_adjust_range(splitter)?;
    let state = splitter.nurbs_state_mut();
    state.surface_type = state.get_surface().get_type()?;
    if state.surface_type == SurfaceType::BezierSurface {
        let (u, v) = (state.range_u, state.range_v);
        state.valid = u.0 >= -0.5 && u.1 <= 1.5 && v.0 >= -0.5 && v.1 <= 1.5;
    }
    Ok(())
}

/// `GenerateSurfaceNodes`: a null list when `initParameters` fails, else the grid.
pub fn nurbs_generate_surface_nodes<T: NurbsRangeSplitterHooks + ?Sized>(
    splitter: &mut T,
    parameters: &MeshParameters,
) -> Result<SurfaceNodes, Failure> {
    if !splitter.init_parameters()? {
        return Ok(SurfaceNodes::default());
    }
    generate_nurbs_nodes(splitter.nurbs_state(), parameters)
}

/// `initParameters` of BRepMesh_NURBSRangeSplitter.
fn nurbs_init_parameters<T: NurbsRangeSplitterHooks + ?Sized>(
    splitter: &mut T,
) -> Result<bool, Failure> {
    let surface = splitter.nurbs_state().get_surface();
    let (range_u, range_v) = (
        splitter.nurbs_state().range_u,
        splitter.nurbs_state().range_v,
    );
    let intervals_u = undefined_interval(splitter, &surface, true, range_u)?;
    let intervals_v = undefined_interval(splitter, &surface, false, range_v)?;
    let original = surface.transformed_surface().geom_surface_original();
    let split = to_split_intervals(&original, [&intervals_u, &intervals_v])?;
    let state = splitter.nurbs_state_mut();
    if !params_from_intervals(
        &intervals_u,
        range_u,
        split,
        state.base.change_parameters_u(),
    ) {
        return Ok(false);
    }
    if !params_from_intervals(
        &intervals_v,
        range_v,
        split,
        state.base.change_parameters_v(),
    ) {
        return Ok(false);
    }
    grab_internal_edge_params(state)
}

/// `getUndefinedInterval`: the CN intervals, or a regular subdivision of the range by
/// `getUndefinedIntervalNb` when the surface reports a single interval.
fn undefined_interval<T: NurbsRangeSplitterHooks + ?Sized>(
    splitter: &T,
    surface: &BRepAdaptorSurface,
    is_u: bool,
    range: (f64, f64),
) -> Result<Vec<f64>, Failure> {
    let mut count = if is_u {
        surface.nb_u_intervals(Shape::CN)?
    } else {
        surface.nb_v_intervals(Shape::CN)?
    };
    if count == 1 {
        count = splitter.get_undefined_interval_nb(surface, is_u, Shape::CN)?;
        if count > 1 {
            let diff = (range.1 - range.0) / f64::from(count);
            return Ok((1..count).map(|i| range.0 + f64::from(i) * diff).collect());
        }
    }
    let mut bounds: Array1<'_, f64> =
        Array1::with_bounds(1, count + 1).map_err(interval_failure)?;
    if is_u {
        surface.u_intervals(&mut bounds, Shape::CN)?;
    } else {
        surface.v_intervals(&mut bounds, Shape::CN)?;
    }
    Ok(bounds.iter().copied().collect())
}

fn interval_failure(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    Failure::with_message(
        "Standard_RangeError",
        &format!(
            "BRepMesh_NURBSRangeSplitter::getUndefinedInterval: {error}; expected a non-negative interval count"
        ),
    )
}

/// `toSplitIntervals`: true when the normal cannot be estimated at some finite pair
/// of interval bounds.
fn to_split_intervals(
    surface: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    intervals: [&Vec<f64>; 2],
) -> Result<bool, Failure> {
    for u in intervals[0].iter().filter(|u| !precision::is_infinite(**u)) {
        for v in intervals[1].iter().filter(|v| !precision::is_infinite(**v)) {
            let mut normal = Direction3::default();
            if norm_estim(
                surface,
                Point2::new(*u, *v),
                precision::confusion(),
                &mut normal,
            )? != 0
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// `initParamsFromIntervals`: each interval bound (and middle when splitting) inside
/// the range; whether any was added.
fn params_from_intervals(
    intervals: &[f64],
    range: (f64, f64),
    split: bool,
    params: &mut IMapOfReal,
) -> bool {
    let mut added = false;
    for (i, start) in intervals.iter().enumerate() {
        added |= add_param(*start, range, params);
        if split && i + 1 < intervals.len() {
            added |= add_param((start + intervals[i + 1]) / 2.0, range, params);
        }
    }
    added
}

/// `addParam`: adds a parameter inside the closed range.
fn add_param(param: f64, range: (f64, f64), params: &mut IMapOfReal) -> bool {
    if param < range.0 || param > range.1 {
        return false;
    }
    params.add(param);
    true
}

/// `grabParamsOfEdges(Edge_Internal, Param_U | Param_V)`: the points of the internal
/// pcurves of this face; always true.
fn grab_internal_edge_params(state: &mut NURBSRangeSplitter) -> Result<bool, Failure> {
    let face = state.get_dface().clone();
    for wire_index in 0..face.wires_nb() {
        let wire = face.get_wire(wire_index)?;
        for edge_index in 0..wire.edges_nb() {
            let edge = wire.get_edge(edge_index)?.upgrade();
            for pcurve_index in 0..edge.pcurves_nb() {
                let pcurve = edge.get_pcurve(pcurve_index)?;
                if pcurve.get_face().upgrade().object_address() != face.object_address()
                    || !pcurve.is_internal()
                {
                    continue;
                }
                for point_index in 0..pcurve.parameters_nb() {
                    let point = pcurve.get_point(point_index)?;
                    state.base.change_parameters_u().add(point.x());
                    state.base.change_parameters_v().add(point.y());
                }
            }
        }
    }
    Ok(true)
}

/// `BRepMesh_DefaultRangeSplitter::AdjustRange` on any splitter of the family.
fn default_adjust_range<T: RangeSplitter + ?Sized>(splitter: &mut T) -> Result<(), Failure> {
    let Some((length_u, length_v)) = splitter.range_state_mut().adjust_lengths()? else {
        return Ok(());
    };
    splitter.compute_tolerance(length_u, length_v)?;
    splitter.compute_delta(length_u, length_v);
    Ok(())
}

/// `BRepMesh_BoundaryParamsRangeSplitter`: the NURBS splitter over the boundary
/// parameters only (`initParameters` returns true at once);
/// `BoundaryParamsRangeSplitter::default()`.
#[derive(Default)]
pub struct BoundaryParamsRangeSplitter {
    base: NURBSRangeSplitter,
}

/// `BRepMesh_ExtrusionRangeSplitter`: the NURBS splitter whose undefined interval
/// count follows the extruded curve; `ExtrusionRangeSplitter::default()`.
#[derive(Default)]
pub struct ExtrusionRangeSplitter {
    base: NURBSRangeSplitter,
}

/// `BRepMesh_UndefinedRangeSplitter`: the NURBS splitter with a single undefined
/// interval; `UndefinedRangeSplitter::default()`.
#[derive(Default)]
pub struct UndefinedRangeSplitter {
    base: NURBSRangeSplitter,
}

macro_rules! nurbs_family_deref {
    ($($splitter:ty),*) => {$(
        impl std::ops::Deref for $splitter {
            type Target = NURBSRangeSplitter;
            fn deref(&self) -> &Self::Target {
                &self.base
            }
        }
        impl std::ops::DerefMut for $splitter {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.base
            }
        }
    )*};
}
nurbs_family_deref!(
    BoundaryParamsRangeSplitter,
    ExtrusionRangeSplitter,
    UndefinedRangeSplitter
);

/// `BRepMesh_ExtrusionRangeSplitter::getUndefinedIntervalNb`: the CN interval count of
/// the extruded curve, or its pole count less one for a single-interval Bezier or
/// B-spline curve.
pub(super) fn extrusion_interval_nb(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
    let curve = surface.basis_curve()?;
    let count = curve.nb_intervals(Shape::CN)?;
    if count == 1
        && matches!(
            curve.get_type()?,
            CurveType::BezierCurve | CurveType::BSplineCurve
        )
    {
        return Ok(curve.nb_poles()? - 1);
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_params_add_bounds_and_middles_inside_the_range() {
        let mut params = IMapOfReal::new(1);
        assert!(params_from_intervals(
            &[0.0, 1.0, 2.0],
            (0.5, 2.0),
            true,
            &mut params
        ));
        let values: Vec<f64> = (1..=params.extent()).map(|i| params[i]).collect();
        assert_eq!(values, vec![0.5, 1.0, 1.5, 2.0]);
        assert!(!params_from_intervals(
            &[5.0],
            (0.0, 1.0),
            false,
            &mut params
        ));
    }
}
