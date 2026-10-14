//! Operation-local presentation observations bracket native GUI submission.
use super::window_gui_contract::{NoPresentationProbe, WindowGuiContract, WindowPresentationProbe};
use super::{
    VulkanRenderer, WindowPresenter, frame::FrameResources, submission::CommandSubmission,
};
use crate::{
    FrameCapture, RenderError, RenderFrame, RenderGuiSurfacePhase, RenderGuiSurfacePolicy,
};
struct WindowGuiOperation<'a> {
    presenter: &'a mut WindowPresenter,
    contract: WindowGuiContract,
    probe: &'a mut dyn WindowPresentationProbe,
    phase: RenderGuiSurfacePhase,
}
impl WindowGuiOperation<'_> {
    fn new<'a>(
        presenter: &'a mut WindowPresenter,
        contract: WindowGuiContract,
        probe: &'a mut dyn WindowPresentationProbe,
    ) -> WindowGuiOperation<'a> {
        WindowGuiOperation {
            presenter,
            contract,
            probe,
            phase: RenderGuiSurfacePhase::Acquire,
        }
    }
    fn acquire(&mut self, extent: [u32; 2], vsync: bool) -> Result<u32, RenderError> {
        self.checkpoint(RenderGuiSurfacePhase::Acquire)?;
        let index = self.presenter.acquire(extent, vsync, self.contract)?;
        self.checkpoint(RenderGuiSurfacePhase::BeforeSubmit)?;
        Ok(index)
    }
    fn present(&mut self, queue: ash::vk::Queue, index: u32) -> Result<(), RenderError> {
        self.checkpoint(RenderGuiSurfacePhase::Present)?;
        self.presenter
            .chain
            .as_mut()
            .unwrap()
            .present(queue, index, self.contract)?;
        self.checkpoint(RenderGuiSurfacePhase::Publication)
    }
    fn checkpoint(&mut self, phase: RenderGuiSurfacePhase) -> Result<(), RenderError> {
        self.phase = phase;
        self.probe.observe(phase)?;
        self.contract.observe(self.presenter, phase)
    }
}
impl VulkanRenderer {
    pub(crate) fn render_window(
        &mut self,
        frame: &RenderFrame,
        presenter: &mut WindowPresenter,
        vsync: bool,
        policy: RenderGuiSurfacePolicy,
    ) -> Result<FrameCapture, RenderError> {
        let mut probe = NoPresentationProbe;
        self.render_window_with_probe(frame, presenter, vsync, policy, &mut probe)
    }
    pub(super) fn render_window_with_probe(
        &mut self,
        frame: &RenderFrame,
        presenter: &mut WindowPresenter,
        vsync: bool,
        policy: RenderGuiSurfacePolicy,
        probe: &mut dyn WindowPresentationProbe,
    ) -> Result<FrameCapture, RenderError> {
        let contract = WindowGuiContract::new(frame, policy);
        let mut operation = WindowGuiOperation::new(presenter, contract, probe);
        let result = self.submit_gui_window(frame, &mut operation, vsync);
        let result = contract.retain_fault(result, operation.presenter, operation.phase);
        if result.is_err() {
            operation.presenter.retire_failed_operation();
        }
        result
    }
    fn submit_gui_window(
        &mut self,
        frame: &RenderFrame,
        operation: &mut WindowGuiOperation<'_>,
        vsync: bool,
    ) -> Result<FrameCapture, RenderError> {
        let resources = self.frame_resources(frame)?;
        let commands = CommandSubmission::new(&self.runtime)?;
        let index = operation.acquire(frame.extent(), vsync)?;
        self.record_gui_window(frame, &commands, &resources, operation.presenter, index)?;
        self.commit_fragment_alpha(frame, resources.fragment_alpha.ending);
        self.commit_stencil(frame, resources.stencil.ending);
        let queue = self.runtime.queues[operation.presenter.present_family as usize];
        operation.present(queue, index)?;
        resources.capture()
    }
    fn record_gui_window(
        &self,
        frame: &RenderFrame,
        commands: &CommandSubmission,
        resources: &FrameResources,
        presenter: &mut WindowPresenter,
        index: u32,
    ) -> Result<(), RenderError> {
        let chain = presenter.chain.as_mut().expect("acquired swapchain");
        super::gui_native_extent::validate(frame, chain.config.extent)?;
        super::record::record_frame(&self.runtime, commands.command, resources, frame);
        super::window_blit::record(&self.runtime, commands.command, resources, chain, index);
        super::window_submission::finish(commands, chain)
    }
}
