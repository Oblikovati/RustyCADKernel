//! ShapeUpgrade_SplitSurfaceArea at OCCT 3d097a0328e71b8.
use super::{ShapeUpgradeSplitSurface, SplitSurface, SplitSurfaceData};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, real::real_to_int},
};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface, geom::GeomSurface, geom_adaptor::GeomAdaptorSurface,
};
use std::cmp::Ordering;

#[derive(Clone, Copy)]
struct AreaSplitSettings {
    parts: i32,
    u_splits: i32,
    v_splits: i32,
    into_squares: bool,
}

/// Split a surface into parameter-space regions sized from its physical aspect ratio.
/// Example: `let splitter = ShapeUpgradeSplitSurfaceArea::new();`.
pub struct ShapeUpgradeSplitSurfaceArea {
    parent: ShapeUpgradeSplitSurface,
    settings: SharedState<AreaSplitSettings>,
}

impl_transient!(ShapeUpgradeSplitSurfaceArea,"ShapeUpgrade_SplitSurfaceArea",ShapeUpgradeSplitSurface,parent,interfaces:[dyn SplitSurface]);

impl Default for ShapeUpgradeSplitSurfaceArea {
    fn default() -> Self {
        Self {
            parent: ShapeUpgradeSplitSurface::new(),
            settings: SharedState::new(AreaSplitSettings {
                parts: 1,
                u_splits: -1,
                v_splits: -1,
                into_squares: false,
            }),
        }
    }
}

impl ShapeUpgradeSplitSurfaceArea {
    /// Create an area criterion with OCCT's default one-part configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the requested number of regions; `assert_eq!(splitter.nb_parts(), 1)`.
    pub fn nb_parts(&self) -> i32 {
        self.settings.read().parts
    }

    /// Set the requested number of regions; `splitter.set_nb_parts(8)`.
    pub fn set_nb_parts(&self, parts: i32) {
        self.settings.write().parts = parts;
    }

    /// Select approximate square regions in physical space; `splitter.set_splitting_into_squares(true)`.
    pub fn set_splitting_into_squares(&self, enabled: bool) {
        self.settings.write().into_squares = enabled;
    }

    /// Set fixed U/V counts, which also set `NbParts` when both are positive.
    /// Example: `splitter.set_numbers_uv_splits(2, 3)?`.
    pub fn set_numbers_uv_splits(&self, u_splits: i32, v_splits: i32) -> Result<(), Failure> {
        self.settings.write().set_uv_counts(u_splits, v_splits)
    }

    /// Access common parent state shared with this criterion;
    /// `splitter.split_surface_data().with(|fields| fields.rows)`.
    pub fn split_surface_data(&self) -> &SplitSurfaceData {
        self.parent.split_surface_data()
    }

    fn compute_area_splits(&self) -> Result<(), Failure> {
        let settings = *self.settings.read();
        if settings.parts <= 1 {
            return Ok(());
        }
        let split = AreaSplitPlan::from_surface(self.split_surface_data(), settings)?;
        if let Some((u_splits, v_splits)) = split.generated_counts {
            let mut state = self.settings.write();
            state.u_splits = u_splits;
            state.v_splits = v_splits;
        }
        split.apply(self)
    }
}

impl AreaSplitSettings {
    fn set_uv_counts(&mut self, u_splits: i32, v_splits: i32) -> Result<(), Failure> {
        self.u_splits = u_splits;
        self.v_splits = v_splits;
        if u_splits > 0 && v_splits > 0 {
            self.parts = u_splits.checked_mul(v_splits).ok_or_else(|| {
                Failure::with_type(
                    "Standard_OutOfRange",
                    &format!("U/V split counts {u_splits} and {v_splits}; expected product within signed 32-bit range"),
                    "",
                )
            })?;
        }
        Ok(())
    }
}

impl SplitSurface for ShapeUpgradeSplitSurfaceArea {
    fn split_surface_data(&self) -> &SplitSurfaceData {
        self.parent.split_surface_data()
    }

    fn compute(&self, _segment: bool) -> Result<(), Failure> {
        self.compute_area_splits()
    }
}

struct AreaSplitPlan {
    first_u: bool,
    first_count: i32,
    second_count: i32,
    generated_counts: Option<(i32, i32)>,
}

struct AreaSplitInput {
    surface: Handle<dyn GeomSurface>,
    bounds: ((f64, f64), (f64, f64)),
    measures: (f64, f64, f64),
}

impl AreaSplitPlan {
    fn from_surface(
        state: &SplitSurfaceData,
        settings: AreaSplitSettings,
    ) -> Result<Self, Failure> {
        let input = state.with(AreaSplitInput::read)?;
        Self::create(&input.surface, input.bounds, input.measures, settings)
    }

    fn create(
        surface: &Handle<dyn GeomSurface>,
        bounds: ((f64, f64), (f64, f64)),
        measures: (f64, f64, f64),
        settings: AreaSplitSettings,
    ) -> Result<Self, Failure> {
        let adaptor = GeomAdaptorSurface::with_bounds(
            surface,
            (bounds.0.0, bounds.0.1, bounds.1.0, bounds.1.1),
        )?;
        let (u_size, v_size) = physical_aspect(&adaptor, bounds)?;
        Self::choose_counts((u_size, v_size), measures, settings)
    }

    fn choose_counts(
        sizes: (f64, f64),
        measures: (f64, f64, f64),
        settings: AreaSplitSettings,
    ) -> Result<Self, Failure> {
        let ratio = sizes.0 / sizes.1;
        let native_u_first = ratio > 1.0;
        let first_u = ratio.partial_cmp(&1.0) != Some(Ordering::Less);
        let adjusted_ratio = if ratio < 1.0 { 1.0 / ratio } else { ratio };
        let fixed = settings.u_splits > 0 && settings.v_splits > 0;
        if settings.into_squares && settings.parts > 0 {
            return Self::square_plan(settings, measures, first_u, native_u_first, fixed);
        }
        let plan = Self::part_counts(settings.parts, adjusted_ratio, first_u);
        Ok(Self::adjust_second_count(plan, fixed))
    }

    fn square_plan(
        mut settings: AreaSplitSettings,
        measures: (f64, f64, f64),
        first_u: bool,
        native_u_first: bool,
        fixed: bool,
    ) -> Result<Self, Failure> {
        set_square_counts(&mut settings, (measures.1, measures.2), measures.0, fixed)?;
        let counts = (!fixed).then_some((settings.u_splits, settings.v_splits));
        let mut plan = Self::square_counts(settings, first_u, native_u_first);
        plan.generated_counts = counts;
        Ok(Self::adjust_second_count(plan, fixed))
    }

    fn square_counts(settings: AreaSplitSettings, first_u: bool, native_u_first: bool) -> Self {
        let (first_count, second_count) = if native_u_first {
            (settings.u_splits, settings.v_splits)
        } else {
            (settings.v_splits, settings.u_splits)
        };
        Self {
            first_u,
            first_count,
            second_count,
            generated_counts: None,
        }
    }

    fn part_counts(parts: i32, ratio: f64, first_u: bool) -> Self {
        let first_count = if ratio >= f64::from(parts) {
            parts
        } else {
            real_to_int((f64::from(parts) * ratio.ceil()).sqrt().ceil())
        };
        let second_count = if ratio >= f64::from(parts) {
            0
        } else {
            real_to_int((f64::from(parts) / f64::from(first_count)).ceil())
        };
        Self {
            first_u,
            first_count,
            second_count,
            generated_counts: None,
        }
    }

    fn adjust_second_count(mut plan: Self, fixed: bool) -> Self {
        if plan.second_count == 1 && !fixed {
            plan.second_count += 1;
        }
        plan
    }

    fn apply(&self, splitter: &impl SplitSurface) -> Result<(), Failure> {
        if self.first_count == 0 {
            return Ok(());
        }
        let (first, second) = if self.first_u {
            (splitter.u_split_values(), splitter.v_split_values())
        } else {
            (splitter.v_split_values(), splitter.u_split_values())
        };
        insert_even_values(&first, self.first_count)?;
        if self.second_count != 0 {
            insert_even_values(&second, self.second_count)?;
        }
        Ok(())
    }
}

impl AreaSplitInput {
    fn read(fields: &super::SplitSurfaceFields) -> Result<Self, Failure> {
        let u = fields.u_values.get().ok_or_else(missing_split_values)?;
        let v = fields.v_values.get().ok_or_else(missing_split_values)?;
        let bounds = (
            u.sequence(|values| Ok((*values.value(1)?, *values.value(2)?)))?,
            v.sequence(|values| Ok((*values.value(1)?, *values.value(2)?)))?,
        );
        Ok(Self {
            surface: fields.surface.clone(),
            bounds,
            measures: (fields.area, fields.u_size, fields.v_size),
        })
    }
}

fn physical_aspect(
    surface: &GeomAdaptorSurface,
    bounds: ((f64, f64), (f64, f64)),
) -> Result<(f64, f64), Failure> {
    let mut u_resolution = surface.u_resolution(1.0)?;
    let mut v_resolution = surface.v_resolution(1.0)?;
    if u_resolution == 0.0 {
        u_resolution = 1.0;
    }
    if v_resolution == 0.0 {
        v_resolution = 1.0;
    }
    Ok((
        (bounds.0.1 - bounds.0.0).abs() / u_resolution,
        (bounds.1.1 - bounds.1.0).abs() / v_resolution,
    ))
}

fn set_square_counts(
    settings: &mut AreaSplitSettings,
    sizes: (f64, f64),
    area: f64,
    fixed: bool,
) -> Result<(), Failure> {
    if fixed {
        return Ok(());
    }
    let square_size = (area / f64::from(settings.parts)).sqrt();
    settings.u_splits = checked_native_int(sizes.0 / square_size, "U square count")?;
    settings.v_splits = checked_native_int(sizes.1 / square_size, "V square count")?;
    replace_zero_count(&mut settings.u_splits);
    replace_zero_count(&mut settings.v_splits);
    Ok(())
}

fn replace_zero_count(count: &mut i32) {
    if *count == 0 {
        *count = 1;
    }
}

fn checked_native_int(value: f64, label: &str) -> Result<i32, Failure> {
    let truncated = value.trunc();
    if !value.is_finite() || truncated < f64::from(i32::MIN) || truncated > f64::from(i32::MAX) {
        return Err(Failure::with_type(
            "Standard_OutOfRange",
            &format!(
                "{label} {value}; expected finite signed 32-bit value in native conversion domain"
            ),
            "",
        ));
    }
    Ok(truncated as i32)
}

fn insert_even_values(
    values: &Handle<rustycad_tkernel::ncollection::HSequence<f64>>,
    count: i32,
) -> Result<(), Failure> {
    let Some(sequence) = values.get() else {
        return Err(missing_split_values());
    };
    sequence.change_sequence(|items| {
        let (first, last) = (*items.value(1)?, *items.value(2)?);
        let step = (last - first) / f64::from(count);
        let mut previous = first;
        for index in 1..count {
            let next = previous + step;
            let position = usize::try_from(index + 1).map_err(|_| missing_split_values())?;
            items.insert_before(position, next)?;
            previous = next;
        }
        Ok::<_, Failure>(())
    })
}

fn missing_split_values() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_SplitSurfaceArea requires initialized split values with two bounds",
        "",
    )
}

#[cfg(test)]
mod tests {
    use super::{AreaSplitPlan, AreaSplitSettings, checked_native_int, replace_zero_count};

    #[test]
    fn native_truncation_accepts_fraction_above_int_max() {
        let value = f64::from(i32::MAX) + 0.5;
        assert_eq!(checked_native_int(value, "test count").unwrap(), i32::MAX);
    }

    #[test]
    fn native_truncation_rejects_next_integer() {
        let value = f64::from(i32::MAX) + 1.0;
        let failure = checked_native_int(value, "test count").unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    }

    #[test]
    fn zero_count_replacement_preserves_nonzero_negative_counts() {
        let mut count = -2;
        replace_zero_count(&mut count);
        assert_eq!(count, -2);
        let mut zero = 0;
        replace_zero_count(&mut zero);
        assert_eq!(zero, 1);
    }

    #[test]
    fn negative_native_aspect_ratio_keeps_v_as_first_direction() {
        let settings = AreaSplitSettings {
            parts: 4,
            u_splits: -1,
            v_splits: -1,
            into_squares: false,
        };
        let plan = AreaSplitPlan::choose_counts((-2., 1.), (0., 0., 0.), settings).unwrap();
        assert!(!plan.first_u);
    }
}
