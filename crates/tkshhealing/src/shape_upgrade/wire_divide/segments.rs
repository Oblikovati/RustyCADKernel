//! Synchronize two parameter spaces before splitting, then independently rebuild reverse seams.
use super::split_values::Knots;
use super::*;
use rustycad_tkbrep::topods;
pub(super) struct DividedCurves {
    pub spatial: SplitCurves3d,
    pub planar: SplitCurves2d,
    pub reversed: SplitCurves2d,
    pub knots3: Knots,
    pub knots2: Knots,
    pub split3: Handle<ShapeUpgradeSplitCurve3d>,
    pub split2: Handle<ShapeUpgradeSplitCurve2d>,
    pub transfer: Handle<dyn TransferParameters>,
    pub was_same_range: bool,
    pub has3: bool,
    pub has2: bool,
    pub seam: bool,
}
pub(super) fn build<T: WireDivider + ?Sized>(
    tool: &T,
    edge: &Edge,
    face: &Face,
    criterion: &Handle<ShapeUpgradeEdgeDivide>,
) -> Result<DividedCurves, Failure> {
    let mut curves = prepare_curves(tool, edge, face, criterion)?;
    synchronize(&mut curves)?;
    curves.seam = !face.is_null() && BRepTool::is_closed_on_face(edge, face)?;
    if curves.has2 {
        curves.planar = curves.split2.curves();
    }
    if curves.seam {
        reverse_seam(&mut curves, edge, face)?;
    }
    if curves.has3 {
        curves.spatial = curves.split3.curves();
    }
    Ok(curves)
}
fn prepare_curves<T: WireDivider + ?Sized>(
    tool: &T,
    edge: &Edge,
    face: &Face,
    criterion: &Handle<ShapeUpgradeEdgeDivide>,
) -> Result<DividedCurves, Failure> {
    let transfer = tool.transfer_param_tool();
    state::require(&transfer, "parameter transfer")?;
    transfer.set_max_tolerance(tool.wire_divide_base().max_tolerance());
    let transfer = transfer.cast::<dyn TransferParameters>();
    transfer.init(edge, face)?;
    let virtual_criterion = criterion.cast::<dyn EdgeDivider>();
    Ok(DividedCurves {
        spatial: Handle::default(),
        planar: Handle::default(),
        reversed: Handle::default(),
        knots3: criterion.knots_3d(),
        knots2: criterion.knots_2d(),
        split3: virtual_criterion.split_curve_3d_tool(),
        split2: virtual_criterion.split_curve_2d_tool(),
        was_same_range: transfer.is_same_range(),
        transfer,
        has3: criterion.has_curve_3d(),
        has2: criterion.has_curve_2d(),
        seam: false,
    })
}
fn synchronize(curves: &mut DividedCurves) -> Result<(), Failure> {
    let (mut values2, mut values3) = (Knots::default(), Knots::default());
    if curves.has2 && curves.has3 {
        values2 = curves.transfer.perform(&curves.knots3, true)?;
        values3 = curves.transfer.perform(&curves.knots2, false)?;
        split_values::correct(&curves.knots3, &curves.knots2, &values2, &values3)?;
    }
    if curves.has2 {
        build_curve(&curves.split2.cast(), &curves.knots3, &values2)?;
    }
    if curves.has3 {
        build_curve(&curves.split3.cast(), &curves.knots2, &values3)?;
    }
    if curves.has2 {
        curves.knots2 = curves.split2.split_values();
    }
    if curves.has3 {
        curves.knots3 = curves.split3.split_values();
    }
    Ok(())
}
fn build_curve(
    tool: &Handle<dyn SplitCurve>,
    other: &Knots,
    values: &Knots,
) -> Result<(), Failure> {
    state::require(tool, "curve splitter")?;
    if !other.is_null() {
        state::require(values, "transferred split values")?;
        values.change_sequence(|sequence| sequence.remove(1))?;
        values.change_sequence(|sequence| sequence.remove(sequence.len()))?;
        tool.set_split_values(values)?;
    }
    tool.build(true)
}
fn reverse_seam(curves: &mut DividedCurves, edge: &Edge, face: &Face) -> Result<(), Failure> {
    let reversed = topods::edge(&edge.reversed())?;
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(&reversed, face, false)?;
    if curve.is_null() {
        curves.seam = false;
        return Ok(());
    }
    curves.split2.init_curve_range(&curve, first, last)?;
    let splitter = curves.split2.cast::<dyn SplitCurve>();
    if !curves.knots2.is_null() {
        splitter.set_split_values(&curves.knots2)?;
    }
    splitter.perform(true)?;
    if split_values::length(&splitter.split_values())? != split_values::length(&curves.knots2)? {
        curves.seam = false;
    } else {
        curves.reversed = curves.split2.curves();
    }
    Ok(())
}
impl DividedCurves {
    pub fn count(&self) -> usize {
        let spatial = self.spatial.get().map(|array| array.array1().len());
        let planar = self.planar.get().map(|array| array.array1().len());
        match (spatial, planar) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) | (None, Some(a)) => a,
            (None, None) => 0,
        }
    }
    pub fn is_done(&self) -> Result<bool, Failure> {
        if self.count() > 1 {
            return Ok(true);
        }
        state::require(&self.split3, "spatial splitter")?;
        if self.split3.status_flag(ShapeExtendStatus::Done) {
            return Ok(true);
        }
        state::require(&self.split2, "pcurve splitter")?;
        Ok(self.split2.status_flag(ShapeExtendStatus::Done))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn done_spatial_status_short_circuits_null_planar_tool() {
        let spatial = Handle::new(ShapeUpgradeSplitCurve3d::new());
        spatial
            .split_curve_data()
            .with_mut(|fields| fields.status = 1);
        let curves = short_circuit_fixture(&spatial);
        assert!(curves.is_done().unwrap());
        spatial
            .split_curve_data()
            .with_mut(|fields| fields.status = 0);
        assert_eq!(
            curves.is_done().unwrap_err().exception_type(),
            "Standard_NullObject"
        );
    }
    fn short_circuit_fixture(spatial: &Handle<ShapeUpgradeSplitCurve3d>) -> DividedCurves {
        DividedCurves {
            spatial: Handle::default(),
            planar: Handle::default(),
            reversed: Handle::default(),
            knots3: Handle::default(),
            knots2: Handle::default(),
            split3: spatial.clone(),
            split2: Handle::default(),
            transfer: Handle::default(),
            was_same_range: true,
            has3: true,
            has2: false,
            seam: false,
        }
    }
}
