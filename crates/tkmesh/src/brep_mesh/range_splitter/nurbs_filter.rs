// Rust translation of BRepMesh_NURBSRangeSplitter::GenerateSurfaceNodes, its parameter
// filters and the AnalyticalFilter of BRepMesh_NURBSRangeSplitter.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::uv_param::IMapOfReal;
use super::*;
use crate::brep_mesh::GeomTool;
use rustycad_tkernel::ncollection::{Map, map_algo};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::gp::Vector3;

/// `IMeshData::MapOfReal`: the unordered set of parameters.
type MapOfReal = Map<f64>;

/// The body of `BRepMesh_NURBSRangeSplitter::GenerateSurfaceNodes` once the parameters
/// are initialized: the filtered U x V grid less the parameters the analytical filters
/// remove (minus those another iso line forbids removing).
pub(super) fn generate_nurbs_nodes(
    splitter: &NURBSRangeSplitter,
    parameters: &MeshParameters,
) -> Result<SurfaceNodes, Failure> {
    let surface = splitter.get_surface();
    let deflection = splitter.get_dface().get_deflection();
    let (range_u, range_v, delta) = (splitter.range_u, splitter.range_v, splitter.delta);
    let mut params_u = grain_and_filter(
        splitter,
        splitter.get_parameters_u(),
        (
            surface.u_resolution(deflection)?,
            range_u.1 - range_u.0,
            delta.0,
        ),
        parameters,
    )?;
    let mut params_v = grain_and_filter(
        splitter,
        splitter.get_parameters_v(),
        (
            surface.v_resolution(deflection)?,
            range_v.1 - range_v.0,
            delta.1,
        ),
        parameters,
    )?;
    let mut fixed = [MapOfReal::new(1), MapOfReal::new(1)];
    let original = surface.transformed_surface().geom_surface_original();
    let face = splitter.get_dface();
    let mut remove_u = AnalyticalFilter::new(face, &original, false, parameters)
        .control_params_to_remove(&params_v, &mut params_u, &mut fixed, [1, 0])?;
    let mut remove_v = AnalyticalFilter::new(face, &original, true, parameters)
        .control_params_to_remove(&params_u, &mut params_v, &mut fixed, [0, 1])?;
    map_algo::subtract(&mut remove_u, &fixed[0]);
    map_algo::subtract(&mut remove_v, &fixed[1]);
    Ok(grid_nodes(&params_u, &params_v, &remove_u, &remove_v))
}

/// The regular grid of the kept parameters, U major.
fn grid_nodes(
    params_u: &[f64],
    params_v: &[f64],
    remove_u: &MapOfReal,
    remove_v: &MapOfReal,
) -> SurfaceNodes {
    let mut nodes = new_surface_nodes();
    for u in params_u.iter().filter(|u| !remove_u.contains(u)) {
        for v in params_v.iter().filter(|v| !remove_v.contains(v)) {
            nodes.append(Point2::new(*u, *v));
        }
    }
    Handle::new(Shared::new(nodes))
}

/// `computeGrainAndFilterParameters(params, Tol2d, RangeDiff, Delta, ...)`.
fn grain_and_filter(
    splitter: &NURBSRangeSplitter,
    params: &IMapOfReal,
    (tol_2d, range_diff, delta): (f64, f64, f64),
    parameters: &MeshParameters,
) -> Result<Vec<f64>, Failure> {
    let mut min_diff = precision::p_confusion();
    if delta < 1.0 {
        min_diff /= delta;
    }
    let surface = splitter.get_surface();
    let min_size_2d = std_max(
        surface.u_resolution(parameters.min_size)?,
        surface.v_resolution(parameters.min_size)?,
    );
    min_diff = std_max(min_size_2d, min_diff);
    let diff_max = 0.1 * range_diff;
    let diff_min = std_max(0.005 * range_diff, 2.0 * tol_2d);
    let diff = std_max(min_size_2d, std_min(diff_max, diff_min));
    Ok(filter_parameters(params, min_diff, diff))
}

/// `filterParameters`: sorted, pre-filtered by the minimal distance, then thinned in
/// series by the filter distance; the first and last kept values always remain.
fn filter_parameters(params: &IMapOfReal, min_dist: f64, filter_dist: f64) -> Vec<f64> {
    let length = params.extent();
    if length < 1 {
        return Vec::new();
    }
    let mut values: Vec<f64> = (1..=length).map(|j| params[j]).collect();
    // Native std::sort of doubles; equal values are identical, so the order of ties is
    // unobservable except between 0.0 and -0.0 (kept in IEEE total order here).
    values.sort_by(f64::total_cmp);
    let count = prefilter(&mut values, min_dist);
    thin_series(&values[..count], filter_dist)
}

/// The mandatory pre-filtering: compacts the values farther than `min_dist` from the
/// last kept one to the front; returns their count.
fn prefilter(values: &mut [f64], min_dist: f64) -> usize {
    let mut length = 1;
    for j in 1..values.len() {
        if values[j] - values[length - 1] > min_dist {
            length += 1;
            if length - 1 < j {
                values[length - 1] = values[j];
            }
        }
    }
    length
}

/// The series filter: a value beyond `filter_dist` from the last added one adds the
/// pending candidate (and is examined again) or itself.
fn thin_series(values: &[f64], filter_dist: f64) -> Vec<f64> {
    let mut result = vec![values[0]];
    let (mut last_added, mut candidate) = (values[0], values[0]);
    let mut has_candidate = false;
    let mut j = 1;
    while j + 1 < values.len() {
        let value = values[j];
        j += 1;
        // Native `if (aVal - aLastAdded > theFilterDist)` else-branch, NaN included.
        if (value - last_added).partial_cmp(&filter_dist) != Some(std::cmp::Ordering::Greater) {
            (candidate, has_candidate) = (value, true);
            continue;
        }
        if has_candidate {
            (last_added, has_candidate) = (candidate, false);
            j -= 1;
        } else {
            last_added = value;
        }
        result.push(last_added);
    }
    result.push(values[values.len() - 1]);
    result
}

/// One control point on the current iso line: parameter, point and derivative.
#[derive(Clone, Copy, Default)]
struct ControlPoint {
    param: f64,
    point: Point3,
    vector: Vector3,
}

/// `AnalyticalFilter`: walks the control parameters along each iso line, inserting
/// middles where the deflection or angle is exceeded and collecting removable ones.
struct AnalyticalFilter<'a> {
    face: &'a crate::imesh_data::FaceHandle,
    surface: &'a Handle<dyn GeomSurface>,
    iso_u: bool,
    parameters: &'a MeshParameters,
    to_remove: MapOfReal,
    iso: Option<GeomAdaptorCurve>,
    curr_param: f64,
    prev: ControlPoint,
    curr: ControlPoint,
}

impl<'a> AnalyticalFilter<'a> {
    fn new(
        face: &'a crate::imesh_data::FaceHandle,
        surface: &'a Handle<dyn GeomSurface>,
        iso_u: bool,
        parameters: &'a MeshParameters,
    ) -> Self {
        Self {
            face,
            surface,
            iso_u,
            parameters,
            to_remove: MapOfReal::new(1),
            iso: None,
            curr_param: 0.0,
            prev: ControlPoint::default(),
            curr: ControlPoint::default(),
        }
    }

    /// `GetControlParametersToRemove`: `fixed[slots[0]]` is the map of the iso
    /// parameters, `fixed[slots[1]]` that of the control parameters.
    fn control_params_to_remove(
        mut self,
        params: &[f64],
        control: &mut Vec<f64>,
        fixed: &mut [MapOfReal; 2],
        slots: [usize; 2],
    ) -> Result<MapOfReal, Failure> {
        let (start, end) = if self.iso_u {
            (0, params.len())
        } else {
            (1, params.len().saturating_sub(1))
        };
        for param in params.iter().take(end).skip(start) {
            self.curr_param = *param;
            self.load_iso()?;
            self.prev = self.control_point(control[0])?;
            let mut j = 1;
            while j < control.len() {
                j += self.check_control_point_and_move_on(j, control, fixed, slots)?;
            }
        }
        Ok(self.to_remove)
    }

    fn load_iso(&mut self) -> Result<(), Failure> {
        let curve = if self.iso_u {
            self.surface.u_iso(self.curr_param)?
        } else {
            self.surface.v_iso(self.curr_param)?
        };
        self.iso = Some(GeomAdaptorCurve::with_curve(&curve)?);
        Ok(())
    }

    fn iso(&self) -> Result<&GeomAdaptorCurve, Failure> {
        self.iso.as_ref().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "AnalyticalFilter: iso curve not loaded; expected a loaded iso line",
            )
        })
    }

    fn control_point(&self, param: f64) -> Result<ControlPoint, Failure> {
        let (point, vector) = self.iso()?.d1(param)?;
        Ok(ControlPoint {
            param,
            point,
            vector,
        })
    }

    /// `checkControlPointAndMoveOn(theIndex)` with a zero-based `index`.
    fn check_control_point_and_move_on(
        &mut self,
        index: usize,
        control: &mut Vec<f64>,
        fixed: &mut [MapOfReal; 2],
        slots: [usize; 2],
    ) -> Result<usize, Failure> {
        self.curr = self.control_point(control[index])?;
        let mid_param = 0.5 * (self.prev.param + self.curr.param);
        let mid = self.iso()?.value(mid_param)?;
        let sq_dist =
            GeomTool::square_deflection_of_segment(self.prev.point, self.curr.point, mid)?;
        let angle = self.angle(self.prev.vector, self.curr.vector)?;
        let sq_max = self.face.get_deflection() * self.face.get_deflection();
        let sq_min = self.parameters.min_size * self.parameters.min_size;
        if (sq_dist > sq_max || angle > self.parameters.angle_interior) && sq_dist > sq_min {
            control.insert(index, mid_param);
            return Ok(0);
        }
        let mut steps = 0;
        let thinnable = sq_dist < sq_max || angle < self.parameters.angle_interior;
        if thinnable
            && control.len() > 3
            && index + 1 < control.len()
            && self.check_parameter_for_deflection(control[index + 1], fixed, slots)?
        {
            steps += 1;
        }
        self.prev = self.curr;
        Ok(steps + 1)
    }

    /// The angle between two derivatives, 0 when either is below the square confusion.
    fn angle(&self, first: Vector3, second: Vector3) -> Result<f64, Failure> {
        let tolerance = precision::square_confusion();
        if first.square_magnitude() > tolerance && second.square_magnitude() > tolerance {
            return first.angle(second);
        }
        Ok(0.0)
    }

    /// `checkParameterForDeflectionAndUpdateCache`: removes the current control
    /// parameter when the next one still fits; forbids it when only the angle fails.
    fn check_parameter_for_deflection(
        &mut self,
        param: f64,
        fixed: &mut [MapOfReal; 2],
        slots: [usize; 2],
    ) -> Result<bool, Failure> {
        let next = self.control_point(param)?;
        let mid = self.iso()?.value(0.5 * (self.prev.param + param))?;
        let sq_dist = GeomTool::square_deflection_of_segment(self.prev.point, next.point, mid)?;
        let sq_max = self.face.get_deflection() * self.face.get_deflection();
        if sq_dist.partial_cmp(&sq_max) != Some(std::cmp::Ordering::Less) {
            return Ok(false);
        }
        let resolution = rustycad_tkmath::gp::resolution();
        if self.prev.vector.square_magnitude() < resolution
            || next.vector.square_magnitude() < resolution
            || self.prev.vector.angle(next.vector)? < self.parameters.angle_interior
        {
            self.to_remove.add(self.curr.param);
            self.curr = next;
            return Ok(true);
        }
        fixed[slots[0]].add(self.curr_param);
        fixed[slots[1]].add(self.curr.param);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefilter_compacts_values_beyond_the_minimal_distance() {
        let mut values = vec![0.0, 0.05, 0.2, 0.21, 1.0];
        let count = prefilter(&mut values, 0.1);
        assert_eq!(&values[..count], &[0.0, 0.2, 1.0]);
    }

    #[test]
    fn thin_series_keeps_ends_and_pending_candidates() {
        assert_eq!(
            thin_series(&[0.0, 0.1, 0.5, 0.6, 2.0], 0.3),
            vec![0.0, 0.1, 0.5, 2.0]
        );
        assert_eq!(thin_series(&[1.0], 0.3), vec![1.0, 1.0]);
    }
}
