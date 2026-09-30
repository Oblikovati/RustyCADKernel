//! Walking curves keep a reusable vertex buffer until line construction detaches it.
use super::{
    general_settings::GeneralSettings,
    line_constructor::construct_lines,
    math_helpers::transition_on_line,
    walking_endpoints::{EndpointBuilder, EndpointState},
    walking_internal::compute_walking_internal,
};
use crate::contap::{
    ContapLine, ContapSurfFunction, ContapTheIWLineOfTheIWalking, ContapTheIWalking,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_start::TheTopolTool;
pub(super) struct WalkingLines<'a> {
    pub function: &'a mut ContapSurfFunction,
    pub output: &'a mut Vec<ContapLine>,
    pub domain: &'a (dyn TheTopolTool + 'static),
    pub diagnostics: &'a mut dyn std::io::Write,
    pub settings: &'a GeneralSettings,
    pub builder: EndpointBuilder<'a>,
}
impl WalkingLines<'_> {
    pub fn append(
        &mut self,
        walk: &ContapTheIWalking,
        endpoints: &mut EndpointState,
    ) -> Result<(), Failure> {
        let mut line = ContapLine::new();
        for index in 1..=walk.nb_lines()? {
            let walked = walk.value(index)?;
            self.initialize_line(&mut line, walked)?;
            self.builder.append(&line, walked, endpoints)?;
            self.finish_line(&line)?;
            line.reset_seq_of_vertex();
        }
        Ok(())
    }
    fn initialize_line(
        &mut self,
        line: &mut ContapLine,
        walked: &ContapTheIWLineOfTheIWalking,
    ) -> Result<(), Failure> {
        line.set_line_on_2s(walked.line());
        let (tangent, rank) = walked.tangent_vector();
        let (u, v) = walked.line().value(rank)?.parameters_on_s2();
        line.set_transition_on_s(transition_on_line(self.function, u, v, tangent)?);
        Ok(())
    }
    fn finish_line(&mut self, line: &ContapLine) -> Result<(), Failure> {
        compute_walking_internal(
            line,
            self.function,
            self.settings.u_resolution,
            self.settings.v_resolution,
            self.diagnostics,
        )?;
        construct_lines(self.output, self.domain, line, self.builder.surface)
    }
}
