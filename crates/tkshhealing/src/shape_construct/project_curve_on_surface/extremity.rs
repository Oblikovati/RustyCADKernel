use super::sampling::CurveSamples;
use super::*;
use rustycad_tkgeomalgo::int_curve::IntConicConic;
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkmath::gp::{Direction2, Line2, Vector2};

struct ExtremityCorrection {
    coordinate: i32,
    finish: f64,
    first: Point2,
    second: Point2,
    first_parameter: f64,
    second_parameter: f64,
    finish_parameter: f64,
    endpoint: Point2,
    result: Point2,
}
impl ShapeConstructProjectCurveOnSurface {
    pub(super) fn correct_singular_ends(
        &self,
        curve: &Handle<dyn GeomCurve>,
        samples: &mut CurveSamples,
        tolerances: (f64, f64),
    ) -> Result<(), Failure> {
        let analysis = self.analysis();
        let mut index = 1;
        let tolerances = [tolerances.0, tolerances.1].map(endpoint_tolerance);
        while let Some(singularity) = analysis.singularity(index)? {
            if singularity.precision <= precision::confusion() {
                self.correct_singularity(curve, samples, tolerances, singularity)?;
            }
            index += 1;
        }
        Ok(())
    }

    fn correct_singularity(
        &self,
        curve: &Handle<dyn GeomCurve>,
        samples: &mut CurveSamples,
        tolerances: [f64; 2],
        singularity: crate::shape_analysis::SurfaceSingularity,
    ) -> Result<(), Failure> {
        for (end, tolerance) in tolerances.into_iter().enumerate() {
            let position = if end == 0 { 1 } else { samples.points.length() };
            if samples.points[position].distance(&singularity.point) <= tolerance {
                self.correct_extremity(
                    curve,
                    samples,
                    end == 0,
                    singularity.first_uv,
                    singularity.u_iso,
                )?;
            }
        }
        Ok(())
    }

    fn correct_extremity(
        &self,
        curve: &Handle<dyn GeomCurve>,
        samples: &mut CurveSamples,
        first: bool,
        singularity: Point2,
        u_iso: bool,
    ) -> Result<(), Failure> {
        let mut correction = ExtremityCorrection::new(samples, first, u_iso)?;
        if !correction.needs_correction(singularity)? {
            return Ok(());
        }
        correction.result = correction.fallback();
        while (correction.second.coord(3 - correction.coordinate)? - correction.finish).abs()
            > 2. * precision::p_confusion()
        {
            if !correction.intersect()? {
                break;
            }
            if !self.refine_extremity(curve, &mut correction, u_iso)? {
                break;
            }
        }
        let index = if first { 1 } else { samples.uv.length() };
        samples.uv[index] = correction.result;
        Ok(())
    }

    fn refine_extremity(
        &self,
        curve: &Handle<dyn GeomCurve>,
        correction: &mut ExtremityCorrection,
        u_iso: bool,
    ) -> Result<bool, Failure> {
        let previous = correction.first;
        if !correction.advance() {
            return Ok(false);
        }
        let analysis = self.analysis();
        let surface = analysis.surface();
        let point = curve.value(correction.second_parameter)?;
        correction.second = analysis.next_value_of_uv(
            correction.first,
            point,
            self.fields.read().precision,
            precision::confusion(),
        )?;
        correction.adjust_periodic(&surface, u_iso)?;
        correction.converging(previous)
    }
}
impl ExtremityCorrection {
    fn new(samples: &CurveSamples, first: bool, u_iso: bool) -> Result<Self, Failure> {
        let count = samples.uv.length();
        let (i, j, k) = if first {
            (3, 2, 1)
        } else {
            (count - 2, count - 1, count)
        };
        let coordinate = if u_iso { 2 } else { 1 };
        let endpoint = samples.uv[k];
        Ok(Self {
            coordinate,
            finish: endpoint.coord(3 - coordinate)?,
            first: samples.uv[i],
            second: samples.uv[j],
            first_parameter: samples.parameters[i],
            second_parameter: samples.parameters[j],
            finish_parameter: samples.parameters[k],
            endpoint,
            result: Point2::default(),
        })
    }

    fn adjust_periodic(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        u_iso: bool,
    ) -> Result<(), Failure> {
        let periodic = if u_iso {
            surface.is_v_periodic()
        } else {
            surface.is_u_periodic()
        };
        if periodic {
            adjust_second(self.first, &mut self.second, surface)?;
        }
        Ok(())
    }

    fn advance(&mut self) -> bool {
        self.first = self.second;
        self.first_parameter = self.second_parameter;
        self.second_parameter = (self.first_parameter + self.finish_parameter) / 2.;
        (self.second_parameter - self.first_parameter).abs() > 2. * precision::p_confusion()
    }
    fn converging(&self, previous: Point2) -> Result<bool, Failure> {
        let old = (self.first.coord(self.coordinate)? - previous.coord(self.coordinate)?).abs();
        let new = (self.second.coord(self.coordinate)? - self.first.coord(self.coordinate)?).abs();
        Ok(new.partial_cmp(&(2. * old)) != Some(std::cmp::Ordering::Greater))
    }

    fn needs_correction(&self, singularity: Point2) -> Result<bool, Failure> {
        let singular = singularity.coord(3 - self.coordinate)?;
        let second = self.second.coord(3 - self.coordinate)?;
        if singular > self.finish && second > self.finish {
            return Ok(false);
        }
        if singular < self.finish && second < self.finish {
            return Ok(false);
        }
        let previous =
            (self.second.coord(self.coordinate)? - self.first.coord(self.coordinate)?).abs();
        let current =
            (self.endpoint.coord(self.coordinate)? - self.second.coord(self.coordinate)?).abs();
        Ok(current > 2. * previous)
    }

    fn fallback(&self) -> Point2 {
        if self.coordinate == 2 {
            Point2::new(self.finish, self.second.y())
        } else {
            Point2::new(self.second.x(), self.finish)
        }
    }

    fn intersect(&mut self) -> Result<bool, Failure> {
        let vector = Vector2::between(self.first, self.second);
        if vector.square_magnitude() <= 1e-32 {
            return Ok(false);
        }
        let direction = if self.coordinate == 2 {
            Direction2::new(0., 1.)?
        } else {
            Direction2::new(1., 0.)?
        };
        let iso = Line2::new(self.endpoint, direction);
        let line = Line2::new(self.first, Direction2::from_vector(vector)?);
        let intersection =
            IntConicConic::lines(iso, &Domain::new(), line, &Domain::new(), 1e-10, 1e-10)?;
        self.result = if intersection.is_done() && !intersection.is_empty()? {
            intersection.point(1)?.value()
        } else {
            self.fallback()
        };
        Ok(true)
    }
}
fn adjust_second(
    first: Point2,
    second: &mut Point2,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    for coordinate in 1..=2 {
        let periodic = if coordinate == 1 {
            surface.is_u_periodic()
        } else {
            surface.is_v_periodic()
        };
        if !periodic {
            continue;
        }
        let period = if coordinate == 1 {
            surface.u_period()?
        } else {
            surface.v_period()?
        };
        adjust_second_coordinate(first, second, coordinate, period)?;
    }
    Ok(())
}

fn endpoint_tolerance(value: f64) -> f64 {
    if value < 0. {
        return precision::confusion();
    }
    value
}

fn adjust_second_coordinate(
    first: Point2,
    second: &mut Point2,
    coordinate: i32,
    period: f64,
) -> Result<(), Failure> {
    let center = first.coord(coordinate)?;
    let value = rustycad_tkmath::elclib::in_period(
        second.coord(coordinate)?,
        center - period / 2.,
        center + period / 2.,
    );
    second.set_coord(coordinate, value)
}

#[cfg(test)]
mod tests {
    use super::super::regression_tests::{planar_line, plane_projector};
    use super::*;

    #[test]
    fn correct_extremity_uses_intersection_and_refines_the_endpoint() {
        let projector = plane_projector();
        let curve = planar_line();
        let mut samples = CurveSamples::generate(&curve, (0., 2.)).unwrap();
        samples.uv[1] = Point2::new(0., 10.);
        samples.uv[2] = Point2::new(1., 0.);
        samples.uv[3] = Point2::new(2., 0.);
        projector
            .correct_extremity(&curve, &mut samples, true, Point2::default(), true)
            .unwrap();
        assert_eq!(samples.uv[1], Point2::default());
    }
}
