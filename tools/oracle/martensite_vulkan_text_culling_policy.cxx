// Actual workspace/GL policy oracle for Martensite; OCCT 3d097a0.
#include "martensite_text_culling_probe.hxx"
static occ::handle<Graphic3d_AspectText3d> policy_aspect(int aspect, int alpha, bool hatch) {
  const occ::handle<Graphic3d_AspectText3d> style = new Graphic3d_AspectText3d;
  style->SetFaceCulling(Graphic3d_TypeOfBackfacingModel(aspect));
  const int modes[] = {0, 1, 2, 3, -1, -1};
  style->SetAlphaMode(Graphic3d_AlphaMode(modes[alpha]));
  Graphic3d_MaterialAspect front = style->FrontMaterial();
  front.SetTransparency(alpha == 5 ? 0.5f : 0.0f);
  style->SetFrontMaterial(front);
  style->SetInteriorStyle(hatch ? Aspect_IS_HATCH : Aspect_IS_SOLID);
  return style;
}
static void emit_policy(NativeTextPolicyProbe& probe, int index) {
  const int alpha = index % 6, hatch = (index / 6) % 2, allowed = (index / 12) % 2;
  const int aspect = (index / 24) % 4, viewMode = index / 96;
  probe.view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel(viewMode));
  const auto style = policy_aspect(aspect, alpha, hatch != 0);
  OpenGl_Aspects glStyle(style);
  probe.workspace->SetAllowFaceCulling(allowed != 0);
  probe.workspace->SetAspects(&glStyle);
  probe.workspace->ApplyAspects(false);
  const auto context = probe.workspace->GetGlContext();
  const bool enabled = context->core11fwd->glIsEnabled(GL_CULL_FACE) != 0;
  GLint culled = 0;
  context->core11fwd->glGetIntegerv(GL_CULL_FACE_MODE, &culled);
  std::cout << viewMode << ' ' << aspect << ' ' << allowed << ' ' << alpha << ' ' << hatch << ' ';
  std::cout << int(context->FaceCulling()) << ' ' << enabled << ' ' << culled << '\n';
}
int main() {
  auto probe = create_policy_probe();
  for (int index = 0; index < 384; ++index) emit_policy(probe, index);
  probe.view->Remove();
}
