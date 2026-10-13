#include "martensite_oit_fixture.hxx"
#include "martensite_oit_layers.hxx"
#include "martensite_oit_draw_trace.hxx"
#define main martensite_existing_cap_fixture_entry
#include "martensite_capping_texture_capture.cxx"
#undef main
static occ::handle<Graphic3d_Structure> oit_capture_geometry(const NativeTextPolicyProbe& probe,
                                                          bool reverse,const std::string& scene) {
  if(scene.rfind("cap-",0)!=0)return oit_native_geometry(probe.viewer,reverse,scene);
  const occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(probe.viewer->StructureManager());
  structure->SetZLayer(Graphic3d_ZLayerId_Default);
  textured_cap_scene(probe.view,structure,scene);return structure;
}
int main(int argc,char** argv) {
  if(argc!=7&&argc!=8&&argc!=9)throw Standard_Failure(("received argc="+std::to_string(argc)+"; expected mode, prefix, samples, factor, layers, reverse, optional fixed flag and scene").c_str());
  auto probe=create_policy_probe();texture_camera(probe.view);
  const std::string scene=argc==9?argv[8]:"two";
  if(scene.rfind("texture-",0)==0)texture_lights(probe.view);
  if(scene=="clipped")texture_view_fixture(probe.view,scene);
  if(argc>=8)probe.workspace->GetGlContext()->caps->ffpEnable=std::stoi(argv[7])!=0;
  oit_native_parameters(probe.view,argv[1],std::stoi(argv[3]),std::stof(argv[4]),std::stoi(argv[5]));
  const auto layered=scene.rfind("layer-",0)==0?oit_native_layers(probe.viewer,scene):std::vector<occ::handle<Graphic3d_Structure>>{};
  const auto structure=layered.empty()?oit_capture_geometry(probe,std::stoi(argv[6])!=0,scene):layered.front();
  probe.view->Redraw();
  if(scene.rfind("cap-",0)==0)cap_live_update(probe.view->ClipPlanes()->First(),scene);
  const auto context=probe.workspace->GetGlContext();
  std::cerr<<"OCCT_GL_VERSION="<<context->core11fwd->glGetString(GL_VERSION)<<std::endl;
  std::cerr<<"OCCT_GL_VENDOR="<<context->core11fwd->glGetString(GL_VENDOR)<<std::endl;
  NativeOitDrawTrace trace(context,argv[2]);
  if(layered.empty())texture_capture(probe.view,argv[2]);else oit_native_full_capture(probe.view,argv[2]);
  const auto glView=occ::handle<OpenGl_View>::DownCast(probe.view->View());
  const auto& params=glView->RenderingParams();
  std::cerr<<"MARTENSITE_OIT_PARAMETERS={\"mode\":"<<int(params.TransparencyMethod)
    <<",\"samples\":"<<params.NbMsaaSamples<<",\"factor\":"<<params.OitDepthFactor
    <<",\"layers\":"<<params.NbOitDepthPeelingLayers<<"}"<<std::endl;
  probe.view->Remove();return 0;
}
