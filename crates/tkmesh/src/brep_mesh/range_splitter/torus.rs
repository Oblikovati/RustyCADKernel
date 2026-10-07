// Rust translation of BRepMesh_TorusRangeSplitter.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::uv_param::IMapOfReal;
use super::*;
use rustycad_tkgeombase::gcpnts::TangentialDeflection;
use rustycad_tkmath::bvh::BvhScalar;

/// `BRepMesh_TorusRangeSplitter`: interior nodes of a torus from the filtered boundary
/// parameters; `TorusRangeSplitter::default()`.
#[derive(Default)]
pub struct TorusRangeSplitter {
    base: UVParamRangeSplitter,
}
impl std::ops::Deref for TorusRangeSplitter {
    type Target = UVParamRangeSplitter;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TorusRangeSplitter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

/// The torus grid: step counts and steps along U and V.
struct TorusGrid {
    nb_u: i32,
    nb_v: i32,
    du: f64,
    dv: f64,
}

impl TorusRangeSplitter {
    /// `GenerateSurfaceNodes`: the boundary parameters thinned by `fillParams` (or a
    /// regular U row when the major radius is below the minor one), kept strictly inside
    /// the range shrunk by a tenth of a step; a null list when both arc steps vanish;
    /// `splitter.generate_surface_nodes(&parameters)?`.
    pub fn generate_surface_nodes(
        &self,
        parameters: &MeshParameters,
    ) -> Result<SurfaceNodes, Failure> {
        let torus = self.get_surface().torus()?;
        let (minor, major) = (torus.minor_radius(), torus.major_radius());
        let Some(grid) = self.grid(parameters, minor, major)? else {
            return Ok(SurfaceNodes::default());
        };
        let params_u = if major < minor {
            (0..=grid.nb_u)
                .map(|i| self.range_u.0 + f64::from(i) * grid.du)
                .collect()
        } else {
            fill_params(self.get_parameters_u(), self.range_u, grid.nb_u, 0.5)
        };
        let params_v = fill_params(self.get_parameters_v(), self.range_v, grid.nb_v, 2.0 / 3.0);
        let mut nodes = new_surface_nodes();
        let range_u = (
            self.range_u.0 + grid.du * 0.1,
            self.range_u.1 - grid.du * 0.1,
        );
        let range_v = (
            self.range_v.0 + grid.dv * 0.1,
            self.range_v.1 - grid.dv * 0.1,
        );
        for u in params_u
            .into_iter()
            .filter(|u| *u >= range_u.0 && *u < range_u.1)
        {
            for v in params_v
                .iter()
                .filter(|v| **v >= range_v.0 && **v < range_v.1)
            {
                nodes.append(Point2::new(u, *v));
            }
        }
        Ok(Handle::new(Shared::new(nodes)))
    }

    /// The native step computation; `None` when `sqrt(Du^2 + oldDv^2)` is below
    /// gp::Resolution().
    fn grid(
        &self,
        parameters: &MeshParameters,
        minor: f64,
        major: f64,
    ) -> Result<Option<TorusGrid>, Failure> {
        let (diff_u, diff_v) = (
            self.range_u.1 - self.range_u.0,
            self.range_v.1 - self.range_v.0,
        );
        let old_dv = self.arc_step(minor, parameters)?;
        // Native `Dv = 0.9 * oldDv` is immediately overwritten by `Dv = oldDv`.
        let nb_v = std::cmp::max((diff_v / old_dv).truncate_to_int(), 2);
        let dv = diff_v / f64::from(nb_v.wrapping_add(1));
        let ru = major + minor;
        let mut du = dv;
        if ru > 1.0e-16 {
            du = self.arc_step(ru, parameters)?;
            let aa = (du * du + old_dv * old_dv).sqrt();
            if aa < rustycad_tkmath::gp::resolution() {
                return Ok(None);
            }
            du *= std_min(old_dv, du) / aa;
        }
        let mut nb_u = std::cmp::max((diff_u / du).truncate_to_int(), 2);
        let ratio = f64::from(nb_v) * diff_u * major / (diff_v * minor) / 5.0;
        nb_u = std::cmp::max(nb_u, ratio.truncate_to_int());
        let du = diff_u / f64::from(nb_u.wrapping_add(1));
        Ok(Some(TorusGrid { nb_u, nb_v, du, dv }))
    }

    fn arc_step(&self, radius: f64, parameters: &MeshParameters) -> Result<f64, Failure> {
        TangentialDeflection::arc_angular_step(
            radius,
            self.face.get_deflection(),
            parameters.angle,
            parameters.min_size,
        )
    }
}

/// `fillParams`: the parameters sorted by `FUN_CalcAverageDUV`, each kept when it is
/// farther than the scaled standard step from every kept one.
fn fill_params(params: &IMapOfReal, range: (f64, f64), steps_nb: i32, scale: f64) -> Vec<f64> {
    let length = params.extent();
    let mut array: Vec<f64> = (1..=length).map(|j| params[j]).collect();
    let diff = (range.1 - range.0).abs();
    let mut step = calc_average_duv(&mut array);
    step = std_max(step, diff / f64::from(steps_nb) / 2.0);
    let mut std_step = diff / length as f64;
    if step > std_step {
        std_step = step;
    }
    std_step *= scale;
    let mut kept: Vec<f64> = Vec::new();
    for value in array {
        if kept
            .iter()
            .all(|kept_value| (kept_value - value).abs() > std_step)
        {
            kept.push(value);
        }
    }
    kept
}

/// `FUN_CalcAverageDUV`: sorts `values` in place (the native exchange sort) and returns
/// the mean of the gaps above 1e-7 between neighbors, -1 without such a gap.
fn calc_average_duv(values: &mut [f64]) -> f64 {
    let (mut result, mut count) = (0.0, 0);
    for i in 0..values.len() {
        for j in i + 1..values.len() {
            if values[i] > values[j] {
                values.swap(i, j);
            }
        }
        if i != 0 {
            let gap = (values[i] - values[i - 1]).abs();
            if gap > 1.0e-7 {
                result += gap;
                count += 1;
            }
        }
    }
    if count != 0 {
        result / f64::from(count)
    } else {
        -1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_duv_sorts_and_skips_tiny_gaps() {
        let mut values = [3.0, 1.0, 1.0, 2.0];
        assert_eq!(calc_average_duv(&mut values), 1.0);
        assert_eq!(values, [1.0, 1.0, 2.0, 3.0]);
        assert_eq!(calc_average_duv(&mut [5.0]), -1.0);
    }

    #[test]
    fn fill_params_drops_values_within_the_scaled_step() {
        let mut params = IMapOfReal::new(1);
        for value in [0.0, 0.1, 1.0, 2.0] {
            params.add(value);
        }
        // avg gap 2/3, diff/steps/2 = 0.25, std = 2/4 -> step 2/3 scaled by 0.5.
        assert_eq!(
            fill_params(&params, (0.0, 2.0), 4, 0.5),
            vec![0.0, 1.0, 2.0]
        );
    }
}
