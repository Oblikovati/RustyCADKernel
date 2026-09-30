use super::closure::restricted_interval;
use super::*;
use crate::shape_analysis::{CurveProjection, ShapeAnalysisCurve};
use rustycad_tkernel::precision;
use rustycad_tkg3d::adaptor3d::Adaptor3dIsoCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::{IsoType, SurfaceType};

struct IsoSearch {
    point: Point3,
    tolerance: f64,
    uv: Point2,
    distance: f64,
    projected: CurveProjection,
}

impl ShapeAnalysisSurface {
    /// Refines UV with boundary and alternating isolines; `analysis.uv_from_iso(point, 1e-7, &mut uv)?`.
    pub fn uv_from_iso(
        &self,
        point: Point3,
        tolerance: f64,
        uv: &mut Point2,
    ) -> Result<f64, Failure> {
        let initial = self.value_uv(*uv)?.distance(&point);
        if initial < tolerance / 10. {
            return Ok(initial);
        }
        self.compute_boxes()?;
        if self.has_missing_iso() {
            return Ok(initial);
        }
        let mut search = IsoSearch::new(point, tolerance, *uv, initial);
        if self.search_isolines(*uv, &mut search).is_err() {
            return Ok(f64::MAX);
        }
        *uv = search.uv;
        Ok(search.distance)
    }

    fn has_missing_iso(&self) -> bool {
        let fields = self.fields.read();
        fields
            .bound_isos
            .as_ref()
            .unwrap()
            .iter()
            .any(Handle::is_null)
    }

    fn search_isolines(&self, initial: Point2, search: &mut IsoSearch) -> Result<(), Failure> {
        let (uf, ul, vf, vl) = self.bounds();
        let parameters = [uf, ul, initial.x(), vf, vl, initial.y()];
        let boxes = [Some(0), Some(1), None, Some(2), Some(3), None];
        let mut point_box = BoundingBox3::default();
        point_box.set_point(search.point);
        for index in 0..6 {
            if self.iso_box_too_far(boxes[index], &point_box, search.distance) {
                continue;
            }
            self.project_iso_step(index < 3, parameters[index], boxes[index], false, search)?;
        }
        self.alternate_isolines(initial, search)
    }

    fn iso_box_too_far(
        &self,
        boundary: Option<usize>,
        point_box: &BoundingBox3,
        distance: f64,
    ) -> bool {
        boundary.is_some_and(|index| self.fields.read().boxes[index].distance(point_box) > distance)
    }

    fn alternate_isolines(
        &self,
        mut previous: Point2,
        search: &mut IsoSearch,
    ) -> Result<(), Failure> {
        for _ in 0..5 {
            if (previous.x() == search.uv.x() && previous.y() == search.uv.y())
                || search.distance <= search.tolerance
            {
                break;
            }
            previous = search.uv;
            self.project_iso_step(false, search.uv.y(), None, false, search)?;
            self.project_iso_step(true, search.uv.x(), None, true, search)?;
        }
        Ok(())
    }

    fn project_iso_step(
        &self,
        u_iso: bool,
        parameter: f64,
        boundary: Option<usize>,
        direct: bool,
        search: &mut IsoSearch,
    ) -> Result<(), Failure> {
        let offset = self.adaptor3d().get_type()? == SurfaceType::OffsetSurface;
        let distance = if offset && boundary.is_none() {
            self.project_offset_iso(u_iso, parameter, direct, search)?
        } else {
            self.project_geometric_iso(u_iso, parameter, offset, search)?
        };
        search.record(distance, u_iso, parameter);
        Ok(())
    }

    fn project_geometric_iso(
        &self,
        u_iso: bool,
        parameter: f64,
        offset: bool,
        search: &mut IsoSearch,
    ) -> Result<f64, Failure> {
        let iso = if u_iso {
            self.u_iso(parameter)
        } else {
            self.v_iso(parameter)
        };
        if iso.is_null() || (!offset && precision::is_infinite(parameter)) {
            return Ok(f64::MAX);
        }
        if offset {
            return search.project_adaptor(&GeomAdaptorCurve::with_curve(&iso)?, false);
        }
        let range = restricted_interval(iso.first_parameter(), iso.last_parameter());
        search.project_geometric(&iso, range)
    }

    fn project_offset_iso(
        &self,
        u_iso: bool,
        parameter: f64,
        direct: bool,
        search: &mut IsoSearch,
    ) -> Result<f64, Failure> {
        let kind = if u_iso { IsoType::IsoU } else { IsoType::IsoV };
        let iso = Adaptor3dIsoCurve::with_iso(&self.adaptor3d().cast(), kind, parameter)?;
        search.project_adaptor(&iso, direct)
    }
}

impl IsoSearch {
    fn new(point: Point3, tolerance: f64, uv: Point2, distance: f64) -> Self {
        Self {
            point,
            tolerance,
            uv,
            distance,
            projected: CurveProjection::default(),
        }
    }

    fn record(&mut self, distance: f64, u_iso: bool, parameter: f64) {
        if distance.partial_cmp(&self.distance) != Some(std::cmp::Ordering::Less) {
            return;
        }
        self.distance = distance;
        self.uv = if u_iso {
            Point2::new(parameter, self.projected.parameter)
        } else {
            Point2::new(self.projected.parameter, parameter)
        };
    }

    fn project_geometric(
        &mut self,
        iso: &Handle<dyn GeomCurve>,
        range: (f64, f64),
    ) -> Result<f64, Failure> {
        ShapeAnalysisCurve.project_into(
            iso,
            self.point,
            self.tolerance,
            range,
            false,
            &mut self.projected,
        )
    }

    fn project_adaptor(
        &mut self,
        iso: &dyn rustycad_tkg3d::adaptor3d::Adaptor3dCurve,
        direct: bool,
    ) -> Result<f64, Failure> {
        if direct {
            return ShapeAnalysisCurve.project_act(
                iso,
                self.point,
                self.tolerance,
                &mut self.projected,
            );
        }
        ShapeAnalysisCurve.project_adaptor_into(
            iso,
            self.point,
            self.tolerance,
            true,
            &mut self.projected,
        )
    }
}
