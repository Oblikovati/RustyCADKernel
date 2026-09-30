use super::{Frpr, MultipleVarFunctionWithGradient};
impl Frpr {
    pub(super) fn minimize_direction(
        &mut self,
        function: &mut impl MultipleVarFunctionWithGradient,
    ) -> bool {
        super::super::optimization::line::minimize(
            function,
            &mut self.result.location,
            &mut self.result.gradient,
            &mut self.result.minimum,
        )
    }
}
