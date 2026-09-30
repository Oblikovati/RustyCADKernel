use super::*;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomToroidalSurface};

/// Native singularity record, including its directed boundary endpoints.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceSingularity {
    pub precision: f64,
    pub point: Point3,
    pub first_uv: Point2,
    pub last_uv: Point2,
    pub first_parameter: f64,
    pub last_parameter: f64,
    pub u_iso: bool,
}

impl ShapeAnalysisSurface {
    /// Computes and orders singularities once; `analysis.compute_singularities()?`.
    pub fn compute_singularities(&self) -> Result<(), Failure> {
        if self.fields.read().singularities.is_some() {
            return Ok(());
        }
        let surface = self.surface();
        let mut singularities = surface_singularities(&surface, self.bounds())?;
        sort_singularities(&mut singularities);
        self.fields.write().singularities = Some(singularities);
        Ok(())
    }

    /// Counts singularities within precision; `let count = analysis.nb_singularities(1e-7)?`.
    pub fn nb_singularities(&self, precision: f64) -> Result<usize, Failure> {
        self.compute_singularities()?;
        Ok(self
            .fields
            .read()
            .singularities
            .as_ref()
            .unwrap()
            .iter()
            .filter(|entry| entry.precision <= precision)
            .count())
    }

    /// Tests whether any singularity meets precision; `analysis.has_singularities(1e-7)?`.
    pub fn has_singularities(&self, precision: f64) -> Result<bool, Failure> {
        Ok(self.nb_singularities(precision)? > 0)
    }

    /// Returns a one-based singularity, or none outside the range; `analysis.singularity(1)?`.
    pub fn singularity(&self, index: i32) -> Result<Option<SurfaceSingularity>, Failure> {
        self.compute_singularities()?;
        let fields = self.fields.read();
        Ok(index
            .checked_sub(1)
            .and_then(|rank| usize::try_from(rank).ok())
            .and_then(|rank| fields.singularities.as_ref().unwrap().get(rank))
            .copied())
    }

    /// Tests proximity to a singularity and updates gap; `analysis.is_degenerated(point, 1e-7)?`.
    pub fn is_degenerated(&self, point: Point3, precision: f64) -> Result<bool, Failure> {
        self.compute_singularities()?;
        let entries = self.fields.read().singularities.clone().unwrap();
        for entry in entries
            .iter()
            .take_while(|entry| entry.precision <= precision)
        {
            let gap = entry.point.distance(&point);
            self.fields.write().gap = gap;
            if gap <= precision {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn surface_singularities(
    surface: &Handle<dyn GeomSurface>,
    bounds: SurfaceBounds,
) -> Result<Vec<SurfaceSingularity>, Failure> {
    if let Some(cone) = surface.cast::<GeomConicalSurface>().get() {
        let apex_v = -cone.ref_radius() / cone.semi_angle().sin();
        return Ok(vec![v_singularity(cone.apex(), 0., bounds, apex_v, false)]);
    }
    if let Some(torus) = surface.cast::<GeomToroidalSurface>().get() {
        return torus_singularities(surface, torus, bounds);
    }
    if surface.is_kind_name("Geom_SphericalSurface") {
        return sphere_singularities(surface, bounds);
    }
    bounded_singularities(surface, bounds)
}

fn sphere_singularities(
    surface: &Handle<dyn GeomSurface>,
    bounds: SurfaceBounds,
) -> Result<Vec<SurfaceSingularity>, Failure> {
    let north = surface.value(bounds.0, bounds.3)?;
    let south = surface.value(bounds.0, bounds.2)?;
    Ok(vec![
        v_singularity(north, 0., bounds, bounds.3, true),
        v_singularity(south, 0., bounds, bounds.2, false),
    ])
}

fn bounded_singularities(
    surface: &Handle<dyn GeomSurface>,
    bounds: SurfaceBounds,
) -> Result<Vec<SurfaceSingularity>, Failure> {
    let kinds = [
        "Geom_BoundedSurface",
        "Geom_SurfaceOfRevolution",
        "Geom_OffsetSurface",
    ];
    if kinds.iter().any(|name| surface.is_kind_name(name)) {
        return boundary_singularities(&GeomAdaptorSurface::with_surface(surface)?, bounds);
    }
    Ok(Vec::new())
}

fn v_singularity(
    point: Point3,
    precision: f64,
    bounds: SurfaceBounds,
    v: f64,
    reverse: bool,
) -> SurfaceSingularity {
    let (first, last) = if reverse {
        (bounds.1, bounds.0)
    } else {
        (bounds.0, bounds.1)
    };
    singularity_record(
        point,
        precision,
        (Point2::new(first, v), Point2::new(last, v)),
        (bounds.0, bounds.1),
        false,
    )
}

fn torus_singularities(
    surface: &Handle<dyn GeomSurface>,
    torus: &GeomToroidalSurface,
    bounds: SurfaceBounds,
) -> Result<Vec<SurfaceSingularity>, Failure> {
    let (minor, major) = (torus.minor_radius(), torus.major_radius());
    let angle = std_min(1., major / minor).acos();
    let precision = std_max(0., major - minor);
    let north = std::f64::consts::PI - angle;
    let south = std::f64::consts::PI + angle;
    let first = surface.value(0., north)?;
    let mut entries = vec![v_singularity(first, precision, bounds, north, false)];
    if major <= minor {
        let second = surface.value(0., south)?;
        entries.push(v_singularity(second, precision, bounds, south, true));
    }
    Ok(entries)
}

fn boundary_singularities(
    surface: &GeomAdaptorSurface,
    bounds: SurfaceBounds,
) -> Result<Vec<SurfaceSingularity>, Failure> {
    let (uf, ul, vf, vl) = bounds;
    let pairs = [
        (Point2::new(uf, vl), Point2::new(uf, vf)),
        (Point2::new(ul, vf), Point2::new(ul, vl)),
        (Point2::new(uf, vf), Point2::new(ul, vf)),
        (Point2::new(ul, vl), Point2::new(uf, vl)),
    ];
    pairs
        .iter()
        .enumerate()
        .map(|(index, &(first, last))| {
            boundary_singularity(surface, first, last, index < 2, bounds)
        })
        .collect()
}

fn boundary_singularity(
    surface: &GeomAdaptorSurface,
    first: Point2,
    last: Point2,
    u_iso: bool,
    bounds: SurfaceBounds,
) -> Result<SurfaceSingularity, Failure> {
    let point = surface.value(0.5 * (first.x() + last.x()), 0.5 * (first.y() + last.y()))?;
    let a = surface.value(first.x(), first.y())?;
    let b = surface.value(last.x(), last.y())?;
    let precision = std_max(
        a.distance(&b),
        std_max(point.distance(&a), point.distance(&b)),
    );
    let (first_parameter, last_parameter) = singularity_range(bounds, u_iso);
    Ok(singularity_record(
        point,
        precision,
        (first, last),
        (first_parameter, last_parameter),
        u_iso,
    ))
}

fn sort_singularities(entries: &mut [SurfaceSingularity]) {
    // Selection order preserves OCCT ties, including their directed endpoints.
    for index in 0..entries.len().saturating_sub(1) {
        let mut minimum = index;
        for next in index + 1..entries.len() {
            if entries[minimum].precision > entries[next].precision {
                minimum = next;
            }
        }
        entries.swap(index, minimum);
    }
}

fn singularity_record(
    point: Point3,
    precision: f64,
    endpoints: (Point2, Point2),
    range: (f64, f64),
    u_iso: bool,
) -> SurfaceSingularity {
    SurfaceSingularity {
        point,
        precision,
        first_uv: endpoints.0,
        last_uv: endpoints.1,
        first_parameter: range.0,
        last_parameter: range.1,
        u_iso,
    }
}

fn singularity_range(bounds: SurfaceBounds, u_iso: bool) -> (f64, f64) {
    if u_iso {
        (bounds.2, bounds.3)
    } else {
        (bounds.0, bounds.1)
    }
}
