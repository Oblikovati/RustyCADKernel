#pragma once
// Real retained OCCT groups and source objects, with updates after the first visible redraw.
struct NativeFixedLightScene {
  occ::handle<Graphic3d_Structure> structure;
  occ::handle<Graphic3d_AspectFillArea3d> aspect;
  occ::handle<Graphic3d_ArrayOfPrimitives> vertices;
  occ::handle<Graphic3d_CLight> light;
};
static std::string fixed_lighting_mode(const std::string& mode) {
  if(mode.rfind("lit-adjacent-",0)==0)return "lit-"+mode.substr(13);
  if(mode.rfind("lit-persist-",0)==0)return "lit-"+mode.substr(12);
  if(mode.rfind("lit-facet-",0)==0)return "lit-facet";
  if(mode.rfind("lit-strip-",0)==0)return "lit-"+mode.substr(10);
  if(mode.rfind("lit-line-",0)==0)return "lit-"+mode.substr(9);
  if(mode.rfind("lit-alpha-",0)==0)return "lit-"+mode.substr(10);
  return mode;
}
static NativeFixedLightScene fixed_native_scene(const NativeTextPolicyProbe& probe,const std::string& input) {
  const std::string mode=fixed_lighting_mode(input);
  const occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(probe.viewer->StructureManager());
  const auto aspect=lighting_aspects(mode);const bool line=input.rfind("lit-line-",0)==0||input.rfind("lit-strip-",0)==0;
  if(input=="lit-unlit")aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  const bool marker=input.rfind("lit-marker-",0)==0;
  if(marker)fixed_marker_aspect(aspect,input);
  if(input.rfind("lit-alpha-",0)==0)lighting_opacity_aspects(aspect,mode);
  if(line) {aspect->SetLineWidth(3);if(mode=="lit-patterned")aspect->SetLineType(Aspect_TOL_DASH);}
  auto vertices=marker?fixed_marker_array(input):fixed_array_override(input);if(vertices.IsNull())vertices=lighting_vertices(mode,line);const auto group=structure->NewGroup();
  group->SetGroupPrimitivesAspect(aspect);group->AddPrimitiveArray(vertices);
  probe.view->ChangeRenderingParams().ShadingModel=Graphic3d_TypeOfShadingModel_Phong;
  const auto light=lighting_lights(probe.view,mode);
  fixed_extra_fixture(structure,probe.view,input);structure->Display();
  return {structure,aspect,vertices,light};
}
static void fixed_native_mutation(const NativeFixedLightScene& fixture,const std::string& input) {
  const std::string mode=fixed_lighting_mode(input);
  if(mode=="lit-normal-updated") {
    fixture.vertices->SetVertexNormal(2,1,0,0);fixture.vertices->Attributes()->Invalidate();
  }
  if(mode=="lit-material-updated")fixture.aspect->SetFrontMaterial(lighting_material(true));
  if(mode=="lit-light-updated")fixture.light->SetDirection(gp_Dir(-1,-1,-1));
  if(mode=="lit-light-off")fixture.light->SetEnabled(false);
  if(input.rfind("lit-alpha-",0)==0)lighting_opacity_mutation(fixture.aspect,mode);
}
