// Reuse the actual section resource/material fixture, changing only native FFP lighting policy.
#define main martensite_existing_cap_main
#include "martensite_capping_texture_capture.cxx"
#undef main
#include "martensite_fixed_light_trace.hxx"
#include "martensite_fixed_cap_lights.hxx"
int main(int argc,char** argv) {
  if(argc!=3)throw Standard_Failure("fixed cap arguments; expected mode and capture prefix");
  auto probe=create_policy_probe();texture_camera(probe.view);
  probe.workspace->GetGlContext()->caps->ffpEnable=true;
  const occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(probe.viewer->StructureManager());
  const auto plane=textured_cap_scene(probe.view,structure,argv[1]);
  fixed_cap_light_source(probe.view,argv[1]);
  probe.view->Redraw();cap_live_update(plane,argv[1]);
  NativeFixedLightingTrace trace(probe.workspace->GetGlContext(),argv[2]);
  texture_capture(probe.view,argv[2]);probe.view->Remove();
}
