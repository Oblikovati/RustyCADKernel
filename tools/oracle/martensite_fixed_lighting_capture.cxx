// Actual configured fixed-function lighting, not a hand-written GL illumination approximation.
#include "martensite_surface_texture_probe.hxx"
#include <Graphic3d_TransformPers.hxx>
static occ::handle<Graphic3d_AspectFillArea3d> unlit_aspects() {
  const occ::handle<Graphic3d_AspectFillArea3d> style=new Graphic3d_AspectFillArea3d;
  style->SetInteriorStyle(Aspect_IS_SOLID);style->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  style->SetFaceCulling(Graphic3d_TypeOfBackfacingModel_DoubleSided);
  style->SetPolygonOffsets(Aspect_POM_Off,0,0);return style;
}
#include "martensite_vulkan_lighting.cxx"
#include "martensite_fixed_light_trace.hxx"
static void fixed_extra_fixture(const occ::handle<Graphic3d_Structure>& structure,
                                const occ::handle<V3d_View>& view,const std::string& mode) {
  if(mode=="lit-headlight-rotated")view->Camera()->SetDirection(gp_Dir(.4,0,-1));
  if(mode=="lit-clipped") {
    const occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane;
    planes->Append(new Graphic3d_ClipPlane(NCollection_Vec4<double>(1,0,0,0)));view->SetClipPlanes(planes);
  }
  if(mode=="lit-perspective")view->Camera()->SetProjectionType(Graphic3d_Camera::Projection_Perspective);
  if(mode.rfind("lit-persist-",0)==0) {
    structure->SetTransformPersistence(new Graphic3d_TransformPers(Graphic3d_TMF_ZoomPers,gp_Pnt()));
  }
  if(mode=="lit-group-scale") {
    gp_Trsf scale;scale.SetScale(gp_Pnt(),.75);structure->Groups().Last()->SetTransformation(scale);
  }
  if(mode=="lit-scale") {
    gp_Trsf scale;scale.SetScale(gp_Pnt(0,0,0),.75);
    structure->SetTransformation(new TopLoc_Datum3D(scale));
  }
  if(mode=="lit-rotation") {
    gp_Trsf rotation;rotation.SetRotation(gp_Ax1(gp_Pnt(),gp_Dir(0,1,0)),.35);
    structure->SetTransformation(new TopLoc_Datum3D(rotation));
  }
}
#include "martensite_fixed_lighting_arrays.hxx"
#include "martensite_fixed_lighting_markers.hxx"
#include "martensite_fixed_lighting_fixture.hxx"
#include "martensite_fixed_lighting_binding.hxx"
static void fixed_texture_capture(NativeTextPolicyProbe& probe,const std::string& mode,
                                  const std::string& prefix) {
  const bool lit=mode.rfind("lit-",0)==0;
  const std::string surfaceMode=lit?mode.substr(4):mode;
  const auto texture=texture_source(surfaceMode);
  texture_view_fixture(probe.view,surfaceMode);
  const auto structure=texture_geometry(probe.viewer,texture,surfaceMode);
  if(lit) {
    structure->Groups().Last()->Aspects()->SetShadingModel(Graphic3d_TypeOfShadingModel_Phong);
    lighting_lights(probe.view,"lit-basic");
  }
  probe.view->Redraw();
  NativeFixedLightingTrace trace(probe.workspace->GetGlContext(),prefix);
  texture_capture_mutation(probe.view,structure,texture,surfaceMode,prefix);
}
int main(int argc,char** argv) {
  if(argc!=3)throw Standard_Failure("fixed lighting arguments; expected mode and capture prefix");
  auto probe=create_policy_probe();texture_camera(probe.view);
  probe.workspace->GetGlContext()->caps->ffpEnable=true;
  if(std::string(argv[1]).rfind("texture-",0)==0) {
    fixed_texture_capture(probe,std::string(argv[1]).substr(8),argv[2]);
    probe.view->Remove();return 0;
  }
  const auto fixture=fixed_native_scene(probe,argv[1]);fixed_adjacent_groups(fixture,argv[1]);probe.view->Redraw();
  if(std::string(argv[1]).find("updated")!=std::string::npos||std::string(argv[1])=="lit-light-off")
    texture_capture(probe.view,std::string(argv[2])+"-initial");
  fixed_native_mutation(fixture,argv[1]);
  NativeFixedLightingTrace trace(probe.workspace->GetGlContext(),argv[2]);
  texture_capture(probe.view,argv[2]);probe.view->Remove();
}
