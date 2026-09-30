//! Wrapper recursion and range synchronization preserve native mutation order.
use super::*;
impl<G: CurveGeometry + 'static> CurveSplitKernel<G> {
    pub(super) fn build_curves(&self, segment: bool) -> Result<(), Failure> {
        let values = self.split_values();
        let (first, last) = self.build_endpoints()?;
        let curve = self.state.read().curve.clone();
        require_curve::<G>(&curve)?;
        if let Some(basis) = G::trimmed_basis(&curve) {
            return self.build_trimmed(&basis, first, last, segment);
        }
        if let Some(basis) = G::offset_basis(&curve) {
            return self.build_offset(&curve, &basis, first, last, segment);
        }
        let (first, last) = self.build_bounds(&curve, first, last)?;
        self.allocate_curves(values.sequence(|s| s.len()) as i32 - 1)?;
        if self.fill_single(&curve, first, last, segment)? {
            return Ok(());
        }
        self.snap_spline_knots(&curve)?;
        self.build_pieces(&curve, segment)
    }
    fn build_endpoints(&self) -> Result<(f64, f64), Failure> {
        let values = self.split_values();
        let first = values.sequence(|s| s.value(1).copied())?;
        let last = values.sequence(|s| s.value(s.len()).copied())?;
        if values.sequence(|s| s.len()) > 2 {
            self.split_curve_data()
                .with_mut(|f| f.status = ShapeExtend::encode_status(ShapeExtendStatus::Done1));
        }
        Ok((first, last))
    }
    fn result_curve(&self, index: i32) -> Result<Handle<G::Curve>, Failure> {
        self.curves()
            .array1()
            .value(index)
            .cloned()
            .map_err(|error| error.into_failure(G::NAME))
    }
    fn nested_splitter(
        &self,
        basis: &Handle<G::Curve>,
        first: f64,
        last: f64,
        segment: bool,
    ) -> Result<Self, Failure> {
        let nested = Self::default();
        nested.init_curve_range(basis, first, last)?;
        nested.set_split_values(&self.split_values())?;
        nested.build(segment)?;
        self.allocate_curves(nested.curves().array1().len() as i32)?;
        Ok(nested)
    }
    fn inherit_status(&self, nested: &Self) {
        let status = nested.split_curve_data().with(|fields| fields.status);
        self.split_curve_data()
            .with_mut(|fields| fields.status |= status);
    }
    fn build_trimmed(
        &self,
        basis: &Handle<G::Curve>,
        first: f64,
        last: f64,
        segment: bool,
    ) -> Result<(), Failure> {
        let nested = self.nested_splitter(basis, first, last, segment)?;
        if nested.curves().array1().len() == 1 {
            let basis = nested.result_curve(1)?;
            self.set_curve(1, G::trim(&basis, first, last)?)?;
        } else {
            self.state.write().result = nested.curves();
        }
        self.inherit_status(&nested);
        Ok(())
    }
    fn build_offset(
        &self,
        curve: &Handle<G::Curve>,
        basis: &Handle<G::Curve>,
        first: f64,
        last: f64,
        segment: bool,
    ) -> Result<(), Failure> {
        let nested = self.nested_splitter(basis, first, last, segment)?;
        for index in 1..=nested.curves().array1().len() as i32 {
            let basis = nested.result_curve(index)?;
            self.set_curve(index, G::offset(curve, &basis)?)?;
        }
        self.inherit_status(&nested);
        Ok(())
    }
    fn build_bounds(
        &self,
        curve: &Handle<G::Curve>,
        mut first: f64,
        mut last: f64,
    ) -> Result<(f64, f64), Failure> {
        if !G::CLAMP_BUILD || G::periodic(curve)? {
            return Ok((first, last));
        }
        let (lower, upper) = G::bounds(curve);
        (first, last) = snap_bounds(first, last, lower, upper);
        let values = self.split_values();
        if first < lower {
            first = lower;
            values.change_sequence(|s| s.set_value(1, first))?;
        }
        if last > upper {
            last = upper;
            values.change_sequence(|s| s.set_value(s.len(), last))?;
        }
        Ok((first, last))
    }
}
