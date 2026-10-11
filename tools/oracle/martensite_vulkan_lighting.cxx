// OCCT 3d097a0 shader lighting oracle; GPU state only exists in this test process.
#include <Graphic3d_CLight.hxx>
#include <Graphic3d_LightSet.hxx>
#include <Graphic3d_CView.hxx>
#include <Graphic3d_ArrayOfSegments.hxx>
static Quantity_Color lighting_rgb(float r, float g, float b) {
  return Quantity_Color(r,g,b,Quantity_TOC_RGB);
}
static Graphic3d_MaterialAspect lighting_material(bool back) {
  Graphic3d_MaterialAspect material(Graphic3d_NameOfMaterial_Plastified);
  material.SetMaterialType(Graphic3d_MATERIAL_ASPECT);
  material.SetAmbientColor(lighting_rgb(.125f,.25f,.375f));
  material.SetDiffuseColor(back ? lighting_rgb(.125f,.5f,.25f) : lighting_rgb(.5f,.25f,.125f));
  material.SetSpecularColor(lighting_rgb(.25f,.125f,.0625f));
  material.SetEmissiveColor(lighting_rgb(.015625f,.03125f,.0625f));
  material.SetShininess(.25f);
  return material;
}
static occ::handle<Graphic3d_AspectFillArea3d> lighting_aspects(const std::string &mode) {
  auto aspect = unlit_aspects();
  aspect->SetShadingModel(mode == "lit-facet" ? Graphic3d_TypeOfShadingModel_PhongFacet : mode == "lit-gouraud" ? Graphic3d_TypeOfShadingModel_Gouraud : mode == "lit-default" ? Graphic3d_TypeOfShadingModel_DEFAULT : Graphic3d_TypeOfShadingModel_Phong);
  aspect->SetInteriorColor(lighting_rgb(.75f,.5f,.25f));
  aspect->SetFrontMaterial(lighting_material(false));
  aspect->SetBackMaterial(lighting_material(true));
  aspect->SetDistinguish(mode == "lit-distinct-back");
  aspect->SetBackInteriorColor(lighting_rgb(.25f,.75f,.5f));
  aspect->SetUseVertexColorForBackFaces(mode != "lit-front-color-only");
  return aspect;
}
static occ::handle<Graphic3d_ArrayOfPrimitives> lighting_vertices(const std::string &mode, bool line) {
  const bool colored = mode == "lit-colored" || mode == "lit-front-color-only";
  const auto flags = static_cast<Graphic3d_ArrayFlags>((mode != "lit-no-normals" ? Graphic3d_ArrayFlags_VertexNormal : 0) | (colored ? Graphic3d_ArrayFlags_VertexColor : 0) | (mode == "lit-normal-updated" ? Graphic3d_ArrayFlags_AttribsMutable : 0));
  occ::handle<Graphic3d_ArrayOfPrimitives> vertices;
  if (line) vertices = new Graphic3d_ArrayOfSegments(3,6,flags);
  else vertices = new Graphic3d_ArrayOfTriangles(3,3,flags);
  const float points[3][3] = {{-8,-8,0},{8,-8,0},{0,8,0}};
  const float normals[3][3] = {{0,0,1},{.6f,0,.8f},{0,.6f,.8f}};
  for (int i=0;i<3;++i) {
    // Mutable buffers start with their full capacity; fill slots instead of appending.
    vertices->SetVertice(i+1,points[i][0],points[i][1],points[i][2]);
    if (mode != "lit-no-normals") vertices->SetVertexNormal(i+1,normals[i][0],normals[i][1],normals[i][2]);
    if (colored) vertices->SetVertexColor(i+1,lighting_rgb(.5f,.75f,.25f));
  }
  if (line) { for (int index : {1,2,2,3,3,1}) vertices->AddEdge(index); return vertices; }
  const bool back = mode == "lit-back" || mode == "lit-distinct-back" || mode == "lit-front-color-only";
  vertices->AddEdge(1); vertices->AddEdge(back ? 3 : 2); vertices->AddEdge(back ? 2 : 3);
  return vertices;
}
static occ::handle<Graphic3d_CLight> lighting_lights(const occ::handle<V3d_View> &view, const std::string &mode) {
  const auto kind = mode == "lit-point" || mode == "lit-range" ? Graphic3d_TypeOfLightSource_Positional : mode == "lit-spot" ? Graphic3d_TypeOfLightSource_Spot : Graphic3d_TypeOfLightSource_Directional;
  occ::handle<Graphic3d_CLight> light = new Graphic3d_CLight(kind);
  light->SetColor(lighting_rgb(.75f,.5f,.25f));
  light->SetIntensity(mode == "lit-intensity" ? 7.f : 1.f);
  if (kind != Graphic3d_TypeOfLightSource_Positional) light->SetDirection(gp_Dir(0,0,-1));
  if (kind != Graphic3d_TypeOfLightSource_Directional) { light->SetPosition(gp_Pnt(0,0,12)); light->SetAttenuation(.5f,.125f); }
  if (mode == "lit-range") light->SetRange(14.f);
  if (mode == "lit-spot") { light->SetAngle(.75f); light->SetConcentration(.125f); }
  if (mode.find("lit-headlight") == 0) light->SetHeadlight(true);
  occ::handle<Graphic3d_CLight> ambient = new Graphic3d_CLight(Graphic3d_TypeOfLightSource_Ambient);
  ambient->SetColor(lighting_rgb(.125f,.25f,.5f)); ambient->SetIntensity(2.f);
  occ::handle<Graphic3d_LightSet> lights = new Graphic3d_LightSet();
  lights->Add(ambient); if (mode != "lit-ambient") lights->Add(light);
  view->View()->SetLights(lights);
  return light;
}
static void lighting_scene(const occ::handle<Graphic3d_Structure> &structure, const occ::handle<V3d_View> &view, const std::string &input_mode) {
  const bool line = input_mode.find("lit-line-") == 0;
  const std::string mode = line ? "lit-" + input_mode.substr(9) : input_mode;
  structure->Clear();
  view->ChangeRenderingParams().ShadingModel = Graphic3d_TypeOfShadingModel_Phong;
  auto group = structure->NewGroup();
  auto aspect = lighting_aspects(mode);
  if (line) {
    aspect->SetLineWidth(3.f);
    if (mode == "lit-patterned") aspect->SetLineType(Aspect_TOL_DASH);
  }
  auto vertices = lighting_vertices(mode,line);
  group->SetGroupPrimitivesAspect(aspect);
  group->AddPrimitiveArray(vertices);
  auto light = lighting_lights(view,mode);
  if (mode == "lit-headlight-rotated") view->Camera()->SetDirection(gp_Dir(.4,0,-1));
  if (mode == "lit-clipped") {
    occ::handle<Graphic3d_SequenceOfHClipPlane> planes = new Graphic3d_SequenceOfHClipPlane();
    planes->Append(new Graphic3d_ClipPlane(NCollection_Vec4<double>(1,0,0,0)));
    view->SetClipPlanes(planes);
  }
  view->Redraw();
  // Static VBOs are immutable; native callers must invalidate a mutable attribute buffer.
  if (mode == "lit-normal-updated") { vertices->SetVertexNormal(2,1.,0.,0.); vertices->Attributes()->Invalidate(); }
  if (mode == "lit-material-updated") aspect->SetFrontMaterial(lighting_material(true));
  if (mode == "lit-light-updated") light->SetDirection(gp_Dir(-1,-1,-1));
  if (mode == "lit-light-off") light->SetEnabled(false);
}
