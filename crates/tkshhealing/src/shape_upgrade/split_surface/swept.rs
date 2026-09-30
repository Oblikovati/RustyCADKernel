//! Split the generating curve, then trim the other sweep direction.
use super::super::{ShapeUpgradeSplitCurve3d, SplitCurve};
use super::*;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution};
use rustycad_tkmath::gp::{Axis1, Direction3};
#[derive(Clone, Copy)]
pub(super) enum SweepKind {
    Revolution(Axis1),
    Extrusion(Direction3),
}
impl SweepKind {
    fn is_revolution(self) -> bool {
        matches!(self, Self::Revolution(_))
    }
    fn surface(self, curve: &Handle<dyn GeomCurve>) -> Result<Handle<dyn GeomSurface>, Failure> {
        match self {
            Self::Revolution(axis) => {
                Ok(Handle::new(GeomSurfaceOfRevolution::new(curve, axis)?).cast())
            }
            Self::Extrusion(direction) => {
                Ok(Handle::new(GeomSurfaceOfLinearExtrusion::new(curve, direction)?).cast())
            }
        }
    }
}
impl SplitSurfaceData {
    pub(super) fn build_swept(
        &self,
        kind: SweepKind,
        basis: &Handle<dyn GeomCurve>,
        segment: bool,
    ) -> Result<(), Failure> {
        let curves = self.split_sweep_curve(kind, basis, segment)?;
        let along = split_snapshot(&self.values(kind.is_revolution()))?;
        let count = curves.curves().array1().len() as i32;
        let grid = self.allocate_sweep_grid(kind, count, (along.len() as i32 - 1).max(1))?;
        for index in 1..=count {
            self.fill_sweep_curve(&grid, kind, &curves, index, &along)?;
        }
        self.adopt_natural_grid(&grid)?;
        self.adopt_curve_status(&curves);
        Ok(())
    }
    fn adopt_curve_status(&self, curves: &ShapeUpgradeSplitCurve3d) {
        for status in [
            ShapeExtendStatus::Done1,
            ShapeExtendStatus::Done2,
            ShapeExtendStatus::Done3,
        ] {
            if curves.status_flag(status) {
                self.add_status(status);
            }
        }
    }
    fn split_sweep_curve(
        &self,
        kind: SweepKind,
        basis: &Handle<dyn GeomCurve>,
        segment: bool,
    ) -> Result<ShapeUpgradeSplitCurve3d, Failure> {
        let values = self.values(!kind.is_revolution());
        let bounds = split_endpoints(&values)?;
        let curves = ShapeUpgradeSplitCurve3d::new();
        curves.init_curve_range(basis, bounds.0, bounds.1)?;
        curves.set_split_values(&values)?;
        curves.build(segment)?;
        Ok(curves)
    }
    fn allocate_sweep_grid(
        &self,
        kind: SweepKind,
        count: i32,
        along: i32,
    ) -> Result<SurfaceGrid, Failure> {
        let (rows, columns) = if kind.is_revolution() {
            (along, count)
        } else {
            (count, along)
        };
        self.with_mut(|fields| {
            let (basis_count, along_count) = if kind.is_revolution() {
                (&mut fields.columns, &mut fields.rows)
            } else {
                (&mut fields.rows, &mut fields.columns)
            };
            *basis_count = count;
            if along > 1 {
                *along_count = along;
            }
        });
        new_grid(rows, columns)
    }
    fn fill_sweep_curve(
        &self,
        grid: &SurfaceGrid,
        kind: SweepKind,
        curves: &ShapeUpgradeSplitCurve3d,
        index: i32,
        along: &[f64],
    ) -> Result<(), Failure> {
        let curve = curves
            .curves()
            .array1()
            .value(index)
            .cloned()
            .map_err(|e| e.into_failure("ShapeUpgrade_SplitSurface::Build"))?;
        let surface = kind.surface(&curve)?;
        let ranges = sweep_ranges(along);
        for (slot, range) in ranges.into_iter().enumerate() {
            let patch = self.sweep_patch(kind, &surface, range, along.len() > 2)?;
            let (row, column) = if kind.is_revolution() {
                (slot as i32 + 1, index)
            } else {
                (index, slot as i32 + 1)
            };
            grid_put(grid, row, column, patch)?;
        }
        Ok(())
    }
    fn sweep_patch(
        &self,
        kind: SweepKind,
        surface: &Handle<dyn GeomSurface>,
        range: (f64, f64),
        multiple: bool,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let natural = surface.bounds();
        let same = if kind.is_revolution() {
            range == (natural.0, natural.1)
        } else {
            range == (natural.2, natural.3)
        };
        if !multiple && same {
            return Ok(surface.clone());
        }
        if kind.is_revolution() {
            return trimmed_surface(surface, (range.0, range.1, natural.2, natural.3));
        }
        if multiple {
            return trimmed_surface(surface, (natural.0, natural.1, range.0, range.1));
        }
        self.trim_extrusion_end(surface, natural, range)
    }
    fn trim_extrusion_end(
        &self,
        surface: &Handle<dyn GeomSurface>,
        natural: (f64, f64, f64, f64),
        range: (f64, f64),
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let u = split_snapshot(&self.values(true))?;
        trimmed_surface(
            surface,
            (
                natural.0.max(u[0]),
                u[u.len() - 1].min(natural.1),
                range.0.max(natural.2),
                range.1.min(natural.3),
            ),
        )
    }
}

fn sweep_ranges(along: &[f64]) -> Vec<(f64, f64)> {
    // OCCT allocates one cross-direction patch even for a one-value shared sequence.
    if along.len() <= 2 {
        return vec![(along[0], along[along.len() - 1])];
    }
    along.windows(2).map(|range| (range[0], range[1])).collect()
}
