#include "martensite_vulkan_derived_clipping.cxx"
// Native retained clipping counterpart of tests/support/clipping_scene.rs.
#include <Graphic3d_ClipPlane.hxx>
#include <Graphic3d_ArrayOfPoints.hxx>
#include <Graphic3d_ArrayOfSegments.hxx>
#include <Graphic3d_SequenceOfHClipPlane.hxx>
static occ::handle<Graphic3d_ClipPlane> clip_plane(double x, double y, double w) {
  return new Graphic3d_ClipPlane(NCollection_Vec4<double>(x,y,0,w));
}
static void clipping_chains(const occ::handle<Graphic3d_SequenceOfHClipPlane>& planes,
                            const occ::handle<Graphic3d_ClipPlane>& first,
                            const std::string& mode) {
  if (mode == "clip-intersection") planes->Append(clip_plane(0,1,0));
  if (mode == "clip-union" || mode == "clip-two-chains" || mode == "clip-child-off") {
    auto child = clip_plane(0,1,0);
    child->SetOn(mode != "clip-child-off");
    first->SetChainNextPlane(child);
  }
  if (mode == "clip-two-chains") {
    auto second = clip_plane(-1,0,5);
    second->SetChainNextPlane(clip_plane(0,-1,5));
    planes->Append(second);
  }
  if (mode == "clip-head-off") first->SetOn(false);
  if (mode == "clip-outside") first->SetEquation(NCollection_Vec4<double>(1,0,0,-20));
  if (mode == "clip-inside") first->SetEquation(NCollection_Vec4<double>(1,0,0,20));
}
static void clipping_local(const occ::handle<Graphic3d_Structure>& structure,
                           const occ::handle<V3d_View>& view, const std::string& mode) {
  if (mode != "clip-local" && mode != "clip-override" && mode != "clip-empty-override") return;
  occ::handle<Graphic3d_SequenceOfHClipPlane> local = new Graphic3d_SequenceOfHClipPlane();
  local->SetOverrideGlobal(mode != "clip-local");
  if (mode != "clip-empty-override") local->Append(clip_plane(0,1,0));
  if (mode == "clip-local") view->SetClipPlanes(occ::handle<Graphic3d_SequenceOfHClipPlane>());
  structure->SetClipPlanes(local);
}
static void clipping_geometry(const occ::handle<Graphic3d_Structure>& structure,
                              const std::string& mode) {
  auto group = structure->Groups().First();
  if (mode == "clip-group-transform") {
    gp_Trsf transform; transform.SetTranslation(gp_Vec(2,3,0));
    group->SetTransformation(transform);
  }
  if (mode != "clip-lines" && mode != "clip-points") return;
  group->Clear(false);
  auto aspects = unlit_aspects();
  aspects->SetMarkerType(Aspect_TOM_PLUS);
  aspects->SetMarkerScale(2);
  group->SetGroupPrimitivesAspect(aspects);
  occ::handle<Graphic3d_ArrayOfPrimitives> vertices;
  if (mode == "clip-lines") vertices = new Graphic3d_ArrayOfSegments(6);
  else vertices = new Graphic3d_ArrayOfPoints(6);
  for (float y : {-3.f,3.f}) for (float x : {-8.f,0.f,8.f}) vertices->AddVertex(x,y,0.f);
  group->AddPrimitiveArray(vertices);
}
static void clipping_persistence(const occ::handle<Graphic3d_Structure>& structure,
                                 const std::string& mode) {
  if (mode.find("pers") == std::string::npos) return;
  double x = (mode == "clip-pers-out" || mode == "clip-pers-override") ? -1 : 1;
  occ::handle<Graphic3d_TransformPers> zoom =
    new Graphic3d_TransformPers(Graphic3d_TMF_ZoomPers, gp_Pnt(x,0,0));
  if (mode == "clip-group-pers" || mode == "clip-nested-pers") {
    auto group = structure->Groups().First();
    group->SetTransformPersistence(zoom);
    gp_Trsf transform; transform.SetTranslation(gp_Vec(2,3,0));
    group->SetTransformation(transform);
  } else structure->SetTransformPersistence(zoom);
  if (mode == "clip-pers-2d" || mode == "clip-nested-pers")
    structure->SetTransformPersistence(persistence_corner(Graphic3d_TMF_2d));
  if (mode == "clip-pers-local" || mode == "clip-pers-override" || mode == "clip-nested-pers") {
    occ::handle<Graphic3d_SequenceOfHClipPlane> local = new Graphic3d_SequenceOfHClipPlane();
    local->Append(clip_plane(0,1,0));
    local->SetOverrideGlobal(mode == "clip-pers-override");
    structure->SetClipPlanes(local);
  }
}
static void clipping_text(const occ::handle<Graphic3d_Structure>& structure,
                           const occ::handle<V3d_View>& view, const std::string& mode) {
  if (mode.rfind("clip-text-",0) != 0) return;
  auto group = structure->Groups().First();
  occ::handle<Graphic3d_AspectText3d> style = new Graphic3d_AspectText3d(
    Quantity_NOC_WHITE, "DejaVu Sans", 1, 0, Aspect_TOST_NORMAL, Aspect_TODT_NORMAL);
  occ::handle<Graphic3d_Text> text = new Graphic3d_Text(16);
  text->SetText("AV"); text->SetPosition(gp_Pnt(-5,0,1));
  group->SetPrimitivesAspect(style); group->AddText(text);
  if (mode == "clip-text-baseline") view->SetClipPlanes(occ::handle<Graphic3d_SequenceOfHClipPlane>());
  else if (mode != "clip-text-cut") {
    occ::handle<Graphic3d_SequenceOfHClipPlane> planes = new Graphic3d_SequenceOfHClipPlane();
    planes->Append(clip_plane(1,0,-20)); view->SetClipPlanes(planes);
  }
  if (mode == "clip-text-no-box") structure->CStructure()->SetBndBoxClipCheck(false);
}

static void update_clipping_planes(const occ::handle<V3d_View>& view,
                                   const occ::handle<Graphic3d_ClipPlane>& first,
                                   const std::string& mode) {
  if (mode == "clip-equation-updated" || mode == "clip-cleared") view->Redraw();
  if (mode == "clip-equation-updated") first->SetEquation(NCollection_Vec4<double>(1,0,0,-2));
  if (mode == "clip-cleared") view->SetClipPlanes(occ::handle<Graphic3d_SequenceOfHClipPlane>());
}
static void clipping_scene(const occ::handle<Graphic3d_Structure>& structure,
                           const occ::handle<V3d_View>& view, const std::string& mode) {
  if (mode.rfind("clip-derived-", 0) == 0) {
    configure_derived_clipping(view, mode);
    return;
  }
  auto first = clip_plane(1,0,0);
  occ::handle<Graphic3d_SequenceOfHClipPlane> global = new Graphic3d_SequenceOfHClipPlane();
  global->Append(first);
  clipping_chains(global,first,mode);
  view->SetClipPlanes(global);
  clipping_local(structure,view,mode);
  clipping_geometry(structure,mode);
  clipping_persistence(structure,mode);
  clipping_text(structure,view,mode);
  structure->CalculateBoundBox();
  update_clipping_planes(view, first, mode);
}
