//! Sampled area screening followed by OCCT's exact mass-property check.
// ShapeAnalysis_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::*;
use rustycad_tkbrep::topods;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Point3, Xy, Xyz};
use rustycad_tktopalgo::brep_gprop::BRepGProp;

#[derive(Default)]
struct SampledWireArea {
    previous_point: Point3,
    previous_vector: Xyz,
    cross_sum: Xyz,
    length: f64,
}

impl ShapeAnalysisWire {
    /// Screen the loaded contour then measure `wire` on the current face.
    /// Example: `analyzer.check_small_area(&wire)?`.
    pub fn check_small_area(&self, wire: &Wire) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Fail1);
        let count = self.wire_data().nb_edges();
        if !self.is_ready() || count < 1 {
            return Ok(false);
        }
        self.set_last_status(ShapeExtendStatus::Ok);
        let Some(center) = self.sampled_area_center(count)? else {
            return Ok(false);
        };
        let center = self.surface().value(center.x(), center.y())?.xyz();
        let Some(sampled) = self.sample_wire_area(count, center)? else {
            return Ok(false);
        };
        if sampled.cross_sum.modulus() < sampled.length * self.precision()
            && self.exact_wire_area_is_small(wire)?
        {
            self.set_last_status(ShapeExtendStatus::Done1);
            return Ok(true);
        }
        Ok(false)
    }

    fn sampled_area_center(&self, count: i32) -> Result<Option<Xy>, Failure> {
        let mut center = Xy::default();
        let analysis = ShapeAnalysisEdge::new();
        for index in 1..=count {
            let (curve, first, last) =
                analysis.pcurve_on_face(&self.wire_data().edge(index)?, &self.face(), true)?;
            if curve.is_null() {
                self.set_last_status(ShapeExtendStatus::Fail2);
                return Ok(None);
            }
            for sample in 1..23 {
                center += curve.value(area_parameter(first, last, sample))?.xy();
            }
        }
        center *= 1. / f64::from(count * 22);
        Ok(Some(center))
    }

    fn sample_wire_area(
        &self,
        count: i32,
        center: Xyz,
    ) -> Result<Option<SampledWireArea>, Failure> {
        let mut sampled = SampledWireArea::default();
        let analysis = ShapeAnalysisEdge::new();
        for index in 1..=count {
            let (curve, first, last) = analysis.curve_3d(&self.wire_data().edge(index)?, true)?;
            let Some(curve) = curve.get() else {
                self.set_last_status(ShapeExtendStatus::Fail2);
                return Ok(None);
            };
            if precision::is_infinite(first) || precision::is_infinite(last) {
                continue;
            }
            sampled.append_curve(curve, first, last, center, index == 1)?;
        }
        Ok(Some(sampled))
    }

    fn exact_wire_area_is_small(&self, wire: &Wire) -> Result<bool, Failure> {
        let mut face = topods::face(&self.face().empty_copied()?)?;
        BRepBuilder::default().add(&mut face, &wire.clone().into_shape())?;
        let shape = face.into_shape();
        let mut area = GProps::new();
        let mut length = GProps::new();
        BRepGProp::surface_properties(&shape, &mut area, false, false)?;
        BRepGProp::linear_properties(&shape, &mut length, false, false)?;
        let tolerance = length.mass() * self.precision();
        Ok(area.mass().abs() < 0.5 * tolerance)
    }
}

impl SampledWireArea {
    fn append_curve(
        &mut self,
        curve: &dyn GeomCurve,
        first: f64,
        last: f64,
        center: Xyz,
        initial: bool,
    ) -> Result<(), Failure> {
        if initial {
            self.previous_point = curve.value(first)?;
            self.previous_vector = self.previous_point.xyz() - center;
        }
        for sample in i32::from(initial)..23 {
            let point = curve.value(area_parameter(first, last, sample))?;
            let vector = point.xyz() - center;
            self.cross_sum += self.previous_vector.crossed(vector);
            self.length += self.previous_point.distance(&point);
            self.previous_point = point;
            self.previous_vector = vector;
        }
        Ok(())
    }
}

fn area_parameter(first: f64, last: f64, sample: i32) -> f64 {
    (1. / 22.) * (f64::from(22 - sample) * first + f64::from(sample) * last)
}
