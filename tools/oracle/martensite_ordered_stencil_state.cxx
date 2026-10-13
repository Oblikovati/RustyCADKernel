// Actual OCCT3d097a0 element transitions; unrelated operation/write-mask state must survive.
#include "martensite_text_culling_probe.hxx"
#include <OpenGl_StencilTest.hxx>
static void stencil_state_row(const NativeTextPolicyProbe& probe, const char* stage) {
  const auto gl=probe.workspace->GetGlContext()->core11fwd;
  const GLenum queries[]={GL_STENCIL_FUNC,GL_STENCIL_REF,GL_STENCIL_VALUE_MASK,
    GL_STENCIL_WRITEMASK,GL_STENCIL_FAIL,GL_STENCIL_PASS_DEPTH_FAIL,GL_STENCIL_PASS_DEPTH_PASS};
  std::cout << stage << ' ' << int(gl->glIsEnabled(GL_STENCIL_TEST));
  for (const auto query:queries) { GLint value=0;gl->glGetIntegerv(query,&value);std::cout << ' ' << value; }
  std::cout << '\n';
}
static void retained_stencil_transitions(const NativeTextPolicyProbe& probe) {
  const auto context=probe.workspace->GetGlContext();const auto gl=context->core11fwd;
  gl->glDisable(GL_STENCIL_TEST);gl->glStencilFunc(GL_LESS,7,0x3F);
  gl->glStencilOp(GL_REPLACE,GL_INCR,GL_DECR);gl->glStencilMask(0x5A);
  stencil_state_row(probe,"initial");
  auto* element=new OpenGl_StencilTest;
  element->SetOptions(true);element->Render(probe.workspace);stencil_state_row(probe,"enabled");
  element->SetOptions(false);element->Render(probe.workspace);stencil_state_row(probe,"disabled");
  element->Release(context.get());stencil_state_row(probe,"released");
  OpenGl_Element::Destroy(context.get(),element);stencil_state_row(probe,"destroyed");
}
int main() {
  const auto probe=create_policy_probe();
  retained_stencil_transitions(probe);
  probe.view->Remove();return 0;
}
