use super::sampling::CurveSamples;
use super::*;
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg3d::geom::GeomRectangularTrimmedSurface;
use rustycad_tkmath::bnd::BoundingBox3;

pub(super) struct IsoRecognition {
    pub recognized: bool,
    pub same_parameters: bool,
    pub u_iso: bool,
    pub endpoints: [Point2; 2],
    pub on_iso: [bool; 2],
    pub curve: Handle<dyn GeomCurve>,
    pub range: (f64, f64),
    pub projected: Array1<'static, f64>,
    matches: [i32; 2],
    parameters: [f64; 2],
    minimum: f64,
    endpoint_minimum: [f64; 2],
    value: f64,
}
struct BoundaryIso {
    curve: Handle<dyn GeomCurve>,
    bounds: BoundingBox3,
    u_iso: bool,
    value: f64,
    range: (f64, f64),
    ends: [Point3; 2],
}
struct IsoEndpointMatches {
    kind: [i32; 2],
    parameter: [f64; 2],
    squared: [f64; 2],
}

impl ShapeConstructProjectCurveOnSurface {
    pub(super) fn recognize_iso(&self, samples: &CurveSamples) -> IsoRecognition {
        let mut result = IsoRecognition::new(samples.points.length());
        if self.recognize_boundaries(samples, &mut result).is_err() {
            result.recognized = false;
        }
        result
    }

    fn recognize_boundaries(
        &self,
        samples: &CurveSamples,
        result: &mut IsoRecognition,
    ) -> Result<(), Failure> {
        let analysis = self.analysis();
        let surface = analysis.surface();
        let mut bounds = analysis.bounds();
        if !surface.cast::<GeomRectangularTrimmedSurface>().is_null() {
            bounds = surface.bounds();
        }
        for index in 0..4 {
            let Some(boundary) = BoundaryIso::new(&analysis, bounds, index)? else {
                continue;
            };
            if boundary.is_degenerate()? {
                continue;
            }
            let matches =
                boundary.match_ends(samples, surface.is_kind_name("Geom_SphericalSurface"))?;
            result.consider(&boundary, &matches);
        }
        result.check_interior(samples)
    }
}

impl IsoRecognition {
    fn new(count: i32) -> Self {
        let minimum = 4. * precision::confusion() * precision::confusion();
        Self {
            recognized: false,
            same_parameters: false,
            u_iso: false,
            endpoints: [Point2::default(); 2],
            on_iso: [false; 2],
            curve: Handle::default(),
            range: (0., 0.),
            projected: Array1::with_bounds(1, count).unwrap(),
            matches: [0; 2],
            parameters: [0.; 2],
            minimum,
            endpoint_minimum: [minimum; 2],
            value: 0.,
        }
    }

    fn consider(&mut self, boundary: &BoundaryIso, matches: &IsoEndpointMatches) {
        if matches.kind[0] > 0
            && matches.kind[1] > 0
            && (matches.parameter[0] - matches.parameter[1]).abs() < precision::p_confusion()
        {
            return;
        }
        self.consider_endpoints(boundary, matches);
        if matches.kind[0] <= 0 || matches.kind[1] <= 0 {
            return;
        }
        let minimum = matches.squared[0] + matches.squared[1];
        if self.minimum <= minimum {
            return;
        }
        self.select_boundary(boundary, matches, minimum);
    }
    fn select_boundary(
        &mut self,
        boundary: &BoundaryIso,
        matches: &IsoEndpointMatches,
        minimum: f64,
    ) {
        self.minimum = minimum;
        self.matches = matches.kind;
        self.parameters = matches.parameter;
        self.u_iso = boundary.u_iso;
        self.value = boundary.value;
        self.curve = boundary.curve.clone();
        self.range = boundary.range;
    }

    fn consider_endpoints(&mut self, boundary: &BoundaryIso, matches: &IsoEndpointMatches) {
        for index in 0..2 {
            if matches.kind[index] <= 0
                || (self.on_iso[index]
                    && matches.squared[index].partial_cmp(&self.endpoint_minimum[index])
                        != Some(std::cmp::Ordering::Less))
            {
                continue;
            }
            self.on_iso[index] = true;
            self.endpoint_minimum[index] = matches.squared[index];
            self.endpoints[index] = boundary.uv(matches.parameter[index]);
        }
    }

    fn check_interior(&mut self, samples: &CurveSamples) -> Result<(), Failure> {
        if self.matches[0] <= 0 || self.matches[1] <= 0 {
            return Ok(());
        }
        self.on_iso = [true; 2];
        self.endpoints = self
            .parameters
            .map(|parameter| iso_uv(self.u_iso, self.value, parameter));
        if self.matches[0] != 3 && self.matches[1] != 3 {
            self.same_parameters = self.check_parameter_identity(samples)?;
        }
        self.recognized = self.same_parameters || self.check_projected_interior(samples)?;
        Ok(())
    }

    fn check_parameter_identity(&self, samples: &CurveSamples) -> Result<bool, Failure> {
        for index in 2..samples.points.length() {
            let parameter = if self.parameters[1] > self.parameters[0] {
                samples.parameters[index]
            } else {
                self.range.0 + self.range.1 - samples.parameters[index]
            };
            if !samples.points[index]
                .is_equal(&self.curve.value(parameter)?, precision::confusion())
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn check_projected_interior(&mut self, samples: &CurveSamples) -> Result<bool, Failure> {
        let range = finite_curve_range(&self.curve);
        let mut previous = self.parameters[0];
        for index in 2..samples.points.length() {
            let (distance, _, parameter) = ShapeAnalysisCurve.next_project(
                previous,
                &self.curve,
                samples.points[index],
                precision::confusion(),
                range,
                false,
            )?;
            previous = parameter;
            self.projected[index] = parameter;
            if distance > precision::confusion() || parameter < range.0 || parameter > range.1 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl BoundaryIso {
    fn new(
        analysis: &ShapeAnalysisSurface,
        bounds: rustycad_tkg3d::geom::SurfaceBounds,
        index: usize,
    ) -> Result<Option<Self>, Failure> {
        let (uf, ul, vf, vl) = bounds;
        let values = [uf, ul, vf, vl];
        let value = values[index];
        if precision::is_infinite(value) {
            return Ok(None);
        }
        let u_iso = index < 2;
        let range = if u_iso { (vf, vl) } else { (uf, ul) };
        let curve = if u_iso {
            analysis.u_iso(value)
        } else {
            analysis.v_iso(value)
        };
        let bounds = iso_boundary_box(analysis, index)?;
        if curve.is_null() {
            return Ok(None);
        }
        Ok(Some(Self::with_curve(curve, bounds, u_iso, value, range)?))
    }
    fn with_curve(
        curve: Handle<dyn GeomCurve>,
        bounds: BoundingBox3,
        u_iso: bool,
        value: f64,
        range: (f64, f64),
    ) -> Result<Self, Failure> {
        let ends = [curve.value(range.0)?, curve.value(range.1)?];
        Ok(Self {
            curve,
            bounds,
            u_iso,
            value,
            range,
            ends,
        })
    }

    fn is_degenerate(&self) -> Result<bool, Failure> {
        let middle = self.curve.value((self.range.0 + self.range.1) / 2.)?;
        Ok(self.ends[0].is_equal(&self.ends[1], precision::confusion())
            && self.ends[0].is_equal(&middle, precision::confusion()))
    }

    fn uv(&self, parameter: f64) -> Point2 {
        iso_uv(self.u_iso, self.value, parameter)
    }

    fn match_ends(
        &self,
        samples: &CurveSamples,
        sphere: bool,
    ) -> Result<IsoEndpointMatches, Failure> {
        let mut result = IsoEndpointMatches {
            kind: [0; 2],
            parameter: [0.; 2],
            squared: [0.; 2],
        };
        let mut used = [false; 2];
        for (index, point) in [samples.points[1], samples.points[samples.points.length()]]
            .into_iter()
            .enumerate()
        {
            if self.match_corner(point, index, &mut used, &mut result) {
                continue;
            }
            if (sphere && !self.u_iso) || self.bounds.is_out_point(point) {
                continue;
            }
            self.match_projected(point, index, &mut result)?;
        }
        Ok(result)
    }

    fn match_projected(
        &self,
        point: Point3,
        index: usize,
        result: &mut IsoEndpointMatches,
    ) -> Result<(), Failure> {
        let range = finite_curve_range(&self.curve);
        let (distance, _, parameter) =
            ShapeAnalysisCurve.project(&self.curve, point, precision::confusion(), range, true)?;
        result.squared[index] = distance * distance;
        if distance <= precision::confusion() && parameter >= range.0 && parameter <= range.1 {
            result.kind[index] = 3;
            result.parameter[index] = parameter;
        }
        Ok(())
    }

    fn match_corner(
        &self,
        point: Point3,
        index: usize,
        used: &mut [bool; 2],
        result: &mut IsoEndpointMatches,
    ) -> bool {
        for (corner, used_corner) in used.iter_mut().enumerate() {
            result.squared[index] = point.square_distance(&self.ends[corner]);
            if result.squared[index] <= precision::confusion() * precision::confusion()
                && !*used_corner
            {
                result.kind[index] = corner as i32 + 1;
                result.parameter[index] = if corner == 0 {
                    self.range.0
                } else {
                    self.range.1
                };
                *used_corner = true;
                return true;
            }
        }
        false
    }
}

pub(super) fn finite_curve_range(curve: &Handle<dyn GeomCurve>) -> (f64, f64) {
    let mut first = curve.first_parameter();
    let mut last = curve.last_parameter();
    if precision::is_infinite(first) {
        first = -1000.;
    }
    if precision::is_infinite(last) {
        last = 1000.;
    }
    (first, last)
}
pub(super) fn iso_uv(u_iso: bool, value: f64, parameter: f64) -> Point2 {
    if u_iso {
        Point2::new(value, parameter)
    } else {
        Point2::new(parameter, value)
    }
}

fn iso_boundary_box(
    analysis: &ShapeAnalysisSurface,
    index: usize,
) -> Result<BoundingBox3, Failure> {
    match index {
        0 => analysis.get_box_uf(),
        1 => analysis.get_box_ul(),
        2 => analysis.get_box_vf(),
        _ => analysis.get_box_vl(),
    }
}
