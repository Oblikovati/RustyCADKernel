use super::*;
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
    GeomSweptSurface,
};
use rustycad_tkmath::geom_abs::SurfaceType;

struct ClosureDistances {
    maximum: f64,
    minimum: f64,
    middle: f64,
}

impl ShapeAnalysisSurface {
    /// Tests U closure with cached native tolerance; `analysis.is_u_closed(1e-7)?`.
    pub fn is_u_closed(&self, tolerance: f64) -> Result<bool, Failure> {
        self.spatial_closure(true, tolerance)
    }

    /// Tests V closure with cached native tolerance; `analysis.is_v_closed(1e-7)?`.
    pub fn is_v_closed(&self, tolerance: f64) -> Result<bool, Failure> {
        self.spatial_closure(false, tolerance)
    }

    /// Returns cached U closure distance; `let gap = analysis.u_close_val();`.
    pub fn u_close_val(&self) -> f64 {
        self.fields.read().closure[0]
    }

    /// Returns cached V closure distance; `let gap = analysis.v_close_val();`.
    pub fn v_close_val(&self) -> f64 {
        self.fields.read().closure[1]
    }

    fn spatial_closure(&self, u_axis: bool, tolerance: f64) -> Result<bool, Failure> {
        let axis = usize::from(!u_axis);
        let cached = self.fields.read().closure[axis];
        if cached >= 0. {
            return Ok(cached <= std_max(tolerance, precision::confusion()));
        }
        let bounds = restricted_bounds(self.bounds());
        let interval = axis_interval(bounds, u_axis);
        self.fields.write().extension[axis] = (interval.1 - interval.0).abs() / 20.;
        if self.native_closed(u_axis)? {
            self.record_closed(axis);
            return Ok(true);
        }
        let distances = self.closure_distances(u_axis, bounds)?;
        self.finish_closure(axis, distances, tolerance)
    }

    fn finish_closure(
        &self,
        axis: usize,
        distances: ClosureDistances,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        self.limit_extension(axis, distances.minimum)?;
        let mut fields = self.fields.write();
        fields.gap = distances.maximum.sqrt();
        fields.closure[axis] = fields.gap;
        if distances.middle > 0. && fields.closure[axis] > distances.middle.sqrt() {
            fields.closure[axis] = f64::MAX;
            return Ok(false);
        }
        Ok(fields.closure[axis] <= std_max(tolerance, precision::confusion()))
    }

    fn native_closed(&self, u_axis: bool) -> Result<bool, Failure> {
        let surface = self.surface();
        if u_axis {
            surface.is_u_closed()
        } else {
            surface.is_v_closed()
        }
    }

    fn record_closed(&self, axis: usize) {
        let mut fields = self.fields.write();
        fields.closure[axis] = 0.;
        fields.extension[axis] = 0.;
        fields.gap = 0.;
    }

    fn limit_extension(&self, axis: usize, minimum: f64) -> Result<(), Failure> {
        if minimum < 0. {
            return Ok(());
        }
        let adaptor = self.adaptor3d();
        let resolution = if axis == 0 {
            adaptor.u_resolution(minimum.sqrt())?
        } else {
            adaptor.v_resolution(minimum.sqrt())?
        };
        let mut fields = self.fields.write();
        fields.extension[axis] = std_min(fields.extension[axis], 0.5 * resolution);
        Ok(())
    }

    fn closure_distances(
        &self,
        u_axis: bool,
        bounds: SurfaceBounds,
    ) -> Result<ClosureDistances, Failure> {
        let surface = self.surface();
        let kind = self.adaptor3d().get_type()?;
        if surface.is_kind_name("Geom_RectangularTrimmedSurface") {
            return self.sample_closure(u_axis, bounds);
        }
        if trivially_unclosed(kind, u_axis) {
            return Ok(ClosureDistances::unclosed());
        }
        if kind == SurfaceType::SurfaceOfExtrusion || kind == SurfaceType::SurfaceOfRevolution {
            return basis_curve_closure(&surface, u_axis);
        }
        self.pole_or_sample_closure(u_axis, bounds)
    }

    fn pole_or_sample_closure(
        &self,
        u_axis: bool,
        bounds: SurfaceBounds,
    ) -> Result<ClosureDistances, Failure> {
        let surface = self.surface();
        if let Some(spline) = surface.cast::<GeomBSplineSurface>().get() {
            return self.spline_closure(spline, u_axis, bounds);
        }
        if let Some(bezier) = surface.cast::<GeomBezierSurface>().get() {
            return pole_closure(bezier.nb_u_poles(), bezier.nb_v_poles(), u_axis, |i, j| {
                bezier.pole(i, j)
            });
        }
        self.sample_closure(u_axis, bounds)
    }

    fn sample_closure(
        &self,
        u_axis: bool,
        bounds: SurfaceBounds,
    ) -> Result<ClosureDistances, Failure> {
        let (first, last) = axis_interval(bounds, !u_axis);
        let parameters = (0..101).map(|i| first + (last - first) * f64::from(i) / 100.);
        self.parameter_closure(u_axis, bounds, parameters)
    }

    fn parameter_closure(
        &self,
        u_axis: bool,
        bounds: SurfaceBounds,
        parameters: impl Iterator<Item = f64>,
    ) -> Result<ClosureDistances, Failure> {
        let (first, last) = axis_interval(bounds, u_axis);
        let mut distances: Option<ClosureDistances> = None;
        for parameter in parameters {
            let eval = |t: f64| {
                if u_axis {
                    self.value(t, parameter)
                } else {
                    self.value(parameter, t)
                }
            };
            let a = eval(first)?;
            let squared = a.square_distance(&eval(last)?);
            accumulate_distances(&mut distances, squared, || {
                Ok(a.square_distance(&eval((first + last) / 2.)?))
            })?;
        }
        Ok(distances.unwrap())
    }

    fn spline_closure(
        &self,
        spline: &GeomBSplineSurface,
        u_axis: bool,
        bounds: SurfaceBounds,
    ) -> Result<ClosureDistances, Failure> {
        let count = if u_axis {
            spline.nb_u_poles()
        } else {
            spline.nb_v_poles()
        };
        if count < 3 {
            return Ok(ClosureDistances::unclosed());
        }
        if spline_needs_samples(spline, u_axis)? {
            let parameters = spline_closure_parameters(spline, u_axis);
            return self.parameter_closure(u_axis, bounds, parameters.into_iter());
        }
        pole_closure(spline.nb_u_poles(), spline.nb_v_poles(), u_axis, |i, j| {
            spline.pole(i, j)
        })
    }
}

fn spline_needs_samples(spline: &GeomBSplineSurface, u_axis: bool) -> Result<bool, Failure> {
    if u_axis {
        Ok(spline.is_u_rational()
            || spline.u_multiplicity(1)? != spline.u_degree() + 1
            || spline.u_multiplicity(spline.nb_u_knots())? != spline.u_degree() + 1)
    } else {
        Ok(spline.is_v_rational()
            || spline.v_multiplicity(1)? != spline.v_degree() + 1
            || spline.v_multiplicity(spline.nb_v_knots())? != spline.v_degree() + 1)
    }
}

fn pole_closure(
    u_count: i32,
    v_count: i32,
    u_axis: bool,
    pole: impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<ClosureDistances, Failure> {
    let (count, other_count) = if u_axis {
        (u_count, v_count)
    } else {
        (v_count, u_count)
    };
    if count < 3 {
        return Ok(ClosureDistances::unclosed());
    }
    let mut distances: Option<ClosureDistances> = None;
    for i in 1..=other_count {
        let eval = |rank: i32| if u_axis { pole(rank, i) } else { pole(i, rank) };
        let first = eval(1)?;
        let squared = first.square_distance(&eval(count)?);
        accumulate_distances(&mut distances, squared, || {
            Ok(first.square_distance(&eval(count / 2 + 1)?))
        })?;
    }
    Ok(distances.unwrap())
}

fn accumulate_distances(
    distances: &mut Option<ClosureDistances>,
    squared: f64,
    middle: impl FnOnce() -> Result<f64, Failure>,
) -> Result<(), Failure> {
    let Some(current) = distances else {
        *distances = Some(ClosureDistances {
            maximum: squared,
            minimum: squared,
            middle: middle()?,
        });
        return Ok(());
    };
    if squared > current.maximum {
        current.maximum = squared;
        current.middle = middle()?;
    } else {
        current.minimum = std_min(current.minimum, squared);
    }
    Ok(())
}

fn basis_curve_closure(
    surface: &Handle<dyn GeomSurface>,
    u_axis: bool,
) -> Result<ClosureDistances, Failure> {
    let curve = closure_basis_curve(surface, u_axis);
    let (first, last) = (curve.first_parameter(), curve.last_parameter());
    if u_axis && (precision::is_infinite(first) || precision::is_infinite(last)) {
        return Ok(ClosureDistances::unclosed());
    }
    let a = curve.value(first)?;
    let maximum = a.square_distance(&curve.value(last)?);
    let middle = if u_axis {
        a.square_distance(&curve.value((first + last) / 2.)?)
    } else {
        -1.
    };
    Ok(ClosureDistances {
        maximum,
        middle,
        minimum: -1.,
    })
}

impl ClosureDistances {
    fn unclosed() -> Self {
        Self {
            maximum: f64::MAX,
            minimum: -1.,
            middle: -1.,
        }
    }
}

pub(super) fn restricted_bounds(bounds: SurfaceBounds) -> SurfaceBounds {
    let (uf, ul) = restricted_interval(bounds.0, bounds.1);
    let (vf, vl) = restricted_interval(bounds.2, bounds.3);
    (uf, ul, vf, vl)
}

pub(super) fn restricted_interval(first: f64, last: f64) -> (f64, f64) {
    match (
        precision::is_negative_infinite(first),
        precision::is_positive_infinite(last),
    ) {
        (true, true) => (-1000., 1000.),
        (true, false) => (last - 2000., last),
        (false, true) => (first, first + 2000.),
        _ => (first, last),
    }
}

fn axis_interval(bounds: SurfaceBounds, u_axis: bool) -> (f64, f64) {
    if u_axis {
        (bounds.0, bounds.1)
    } else {
        (bounds.2, bounds.3)
    }
}

fn trivially_unclosed(kind: SurfaceType, u_axis: bool) -> bool {
    kind == SurfaceType::Plane
        || (!u_axis
            && matches!(
                kind,
                SurfaceType::Cone
                    | SurfaceType::Cylinder
                    | SurfaceType::Sphere
                    | SurfaceType::SurfaceOfExtrusion
            ))
}

fn spline_closure_parameters(spline: &GeomBSplineSurface, u_axis: bool) -> Vec<f64> {
    let knots = if u_axis {
        spline.v_knots()
    } else {
        spline.u_knots()
    };
    let mut parameters = vec![knots[1]];
    for index in 2..=knots.length() {
        parameters.push(0.5 * (knots[index - 1] + knots[index]));
    }
    parameters
}

fn closure_basis_curve(surface: &Handle<dyn GeomSurface>, u_axis: bool) -> Handle<dyn GeomCurve> {
    if u_axis {
        surface.cast::<GeomSurfaceOfLinearExtrusion>().basis_curve()
    } else {
        surface.cast::<GeomSurfaceOfRevolution>().basis_curve()
    }
}
