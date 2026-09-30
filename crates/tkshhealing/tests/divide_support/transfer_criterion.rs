//! A real specialized scalar criterion verifies inherited sequence dispatch.
use rustycad_tkernel::{impl_transient, standard::Failure};
use rustycad_tkshhealing::shape_analysis::{
    ShapeAnalysisTransferParameters, TransferParameterData, TransferParameters,
};
#[derive(Default)]
pub struct OffsetTransfer {
    parent: ShapeAnalysisTransferParameters,
}
impl_transient!(OffsetTransfer,"TransferParameterTest_Offset",ShapeAnalysisTransferParameters,parent,interfaces:[dyn TransferParameters]);
impl TransferParameters for OffsetTransfer {
    fn transfer_parameter_data(&self) -> &TransferParameterData {
        self.parent.transfer_parameter_data()
    }
    fn perform_parameter(&self, parameter: f64, to_2d: bool) -> Result<f64, Failure> {
        let base = self.parent.perform_parameter(parameter, to_2d)?;
        let direction = if to_2d { 1. } else { -1. };
        Ok(base + direction)
    }
}
