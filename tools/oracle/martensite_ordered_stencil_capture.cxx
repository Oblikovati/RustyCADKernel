// Actual retained OCCT3d097a0 groups, native pass filtering, capping and context lifetime.
#include "martensite_oit_fixture.hxx"
#include "martensite_oit_layers.hxx"
#include "martensite_oit_draw_trace.hxx"
#define main martensite_existing_cap_fixture_entry
#include "martensite_capping_texture_capture.cxx"
#undef main
#include "martensite_stencil_seed.hxx"
#include "martensite_stencil_draw_trace.hxx"
static void stencil_cap_fixture(const NativeTextPolicyProbe& probe,
                               const occ::handle<Graphic3d_Structure>& structure,const std::string& mode) {
  auto group=structure->Groups().First();group->Clear(false);group->SetGroupPrimitivesAspect(unlit_aspects());
  cap_cube(group,0);group->SetStencilTestOptions(mode!="cap-disable");
  if(mode=="cap-empty-closed"){group->Clear(false);group->SetClosed(true);}
  auto plane=cap_plane(0,0,-1,0);
  if(mode=="cap-transparent") {
    auto aspect=plane->CappingAspect();aspect->SetInteriorColor(Quantity_ColorRGBA(0,1,0,.5f));
    auto material=aspect->FrontMaterial();material.SetTransparency(.5);aspect->SetFrontMaterial(material);
    aspect->SetAlphaMode(Graphic3d_AlphaMode_Blend,.5f);
  }
  const occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane;
  planes->Append(plane);probe.view->SetClipPlanes(planes);
}
static void stencil_group_fixture(const occ::handle<Graphic3d_Structure>& structure,const std::string& mode) {
  if(mode=="trailing-empty"){structure->NewGroup()->SetStencilTestOptions(true);return;}
  if(mode.rfind("layer-",0)==0){structure->Groups().First()->SetStencilTestOptions(true);return;}
  auto group=structure->Groups().First();group->Clear(false);group->SetGroupPrimitivesAspect(oit_native_aspect(0,"two"));
  group->SetStencilTestOptions(mode!="disable");
  if(mode!="leading-empty")group->AddPrimitiveArray(oit_native_quad(0,"two"));
  if(mode=="toggle")group->SetStencilTestOptions(false);
}
static void stencil_capture_state(const NativeTextPolicyProbe& probe,const std::string& prefix) {
  const auto gl=probe.workspace->GetGlContext()->core11fwd;
  std::ofstream output(prefix+".stencil-state");output<<int(gl->glIsEnabled(GL_STENCIL_TEST));
  for(const GLenum query:{GL_STENCIL_FUNC,GL_STENCIL_REF,GL_STENCIL_VALUE_MASK,GL_STENCIL_WRITEMASK,
    GL_STENCIL_FAIL,GL_STENCIL_PASS_DEPTH_FAIL,GL_STENCIL_PASS_DEPTH_PASS}) {
    GLint value=0;gl->glGetIntegerv(query,&value);output<<' '<<value;
  }
  output<<'\n';
}
static void stencil_gl_identity(const NativeTextPolicyProbe& probe) {
  const auto gl=probe.workspace->GetGlContext()->core11fwd;
  std::cerr<<"OCCT_GL_VERSION="<<gl->glGetString(GL_VERSION)<<'\n';
  std::cerr<<"OCCT_GL_VENDOR="<<gl->glGetString(GL_VENDOR)<<'\n';
}
static int stencil_capture_seed(int argc,char** argv) {
  const int seed=argc==5?std::stoi(argv[4]):0;
  if(seed<0||seed>255)throw Standard_Failure(("received seed="+std::to_string(seed)+"; expected0..255").c_str());
  return seed;
}
static void stencil_execute_capture(const NativeTextPolicyProbe& probe,char** argv) {
  const std::string mode=argv[1];
  oit_native_parameters(probe.view,argv[2],0,0,4);
  const auto layered=mode.rfind("layer-",0)==0?oit_native_layers(probe.viewer,mode):std::vector<occ::handle<Graphic3d_Structure>>{};
  const auto structure=layered.empty()?oit_native_geometry(probe.viewer,false,mode=="line"?"line-mask":mode=="point"?"sprite-opaque":mode=="text"?"text-colored":"two"):layered.front();
  if(mode.rfind("cap-",0)==0)stencil_cap_fixture(probe,structure,mode);else stencil_group_fixture(structure,mode);
  probe.view->Redraw();
  if(mode=="clear-executed")structure->Groups().First()->Clear(false);
  if(mode=="remove-executed")structure->Groups().First()->Remove();
  NativeOitDrawTrace trace(probe.workspace->GetGlContext(),argv[3]);
  if(layered.empty())texture_capture(probe.view,argv[3]);else oit_native_full_capture(probe.view,argv[3]);
  stencil_capture_state(probe,argv[3]);
}
#ifndef MARTENSITE_STENCIL_CAPTURE_NO_MAIN
int main(int argc,char** argv) {
  if(argc!=4&&argc!=5)throw Standard_Failure(("received argc="+std::to_string(argc)+"; expected mode method prefix and optional seed0..255").c_str());
  auto probe=create_policy_probe();stencil_gl_identity(probe);texture_camera(probe.view);
  const int seed=stencil_capture_seed(argc,argv);
  NativeStencilSeed seeded(probe.workspace->GetGlContext(),seed);
  NativeStencilDrawTrace stencil_trace(probe.workspace->GetGlContext(),argv[3]);
  stencil_execute_capture(probe,argv);
  probe.view->Remove();return 0;
}
#endif
