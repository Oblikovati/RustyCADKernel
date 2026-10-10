// An upstream AIS custom object exercises the same triangle, view and lifecycle
// as Martensite.
#include <AIS_InteractiveContext.hxx>
#include <AIS_InteractiveObject.hxx>
#include <PrsMgr_Presentation.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_Selection.hxx>
class MartensiteOracleTriangle : public AIS_InteractiveObject {
  occ::handle<Graphic3d_ArrayOfTriangles> vertices = triangle_vertices();

public:
  bool AcceptDisplayMode(int mode) const override {
    return mode == 0 || mode == 1;
  }

protected:
  void Compute(const occ::handle<PrsMgr_PresentationManager> &,
               const occ::handle<Prs3d_Presentation> &presentation,
               int mode) override {
    if (!AcceptDisplayMode(mode))
      throw std::runtime_error("received display mode " + std::to_string(mode) +
                               "; expected 0 or 1");
    auto group = presentation->NewGroup();
    auto style = unlit_aspects();
    if (mode == 1)
      style->SetInteriorColor(Quantity_Color(0, 1, 0, Quantity_TOC_RGB));
    group->SetGroupPrimitivesAspect(style);
    group->AddPrimitiveArray(vertices);
  }
  void ComputeSelection(const occ::handle<SelectMgr_Selection> &selection,
                        int mode) override {
    selection->Add(new Select3D_SensitivePoint(
        new SelectMgr_EntityOwner(this, mode), gp_Pnt()));
  }
};
struct MartensiteOracleAisScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<MartensiteOracleTriangle> object;
  MartensiteOracleAisScene(const occ::handle<V3d_Viewer> &viewer,
                           double offset) {
    context = new AIS_InteractiveContext(viewer);
    object = new MartensiteOracleTriangle();
    context->Display(object, false);
    gp_Trsf transform;
    transform.SetTranslation(gp_Vec(offset, 0, 0));
    object->SetLocalTransformation(transform);
  }
  void transition(const std::string &mode, const occ::handle<V3d_View> &view) {
    if (mode == "ais-displayed")
      return;
    if (mode == "ais-erased") {
      context->Erase(object, false);
      return;
    }
    if (mode == "ais-redisplayed") {
      context->Erase(object, true);
      context->Display(object, true);
      return;
    }
    if (mode == "ais-mode1") {
      context->Display(object, 1, -1, false);
      return;
    }
    if (mode == "ais-affinity-off") {
      context->SetViewAffinity(object, view, false);
      return;
    }
    if (mode == "ais-cleared") {
      context->ClearPrs(object, 0, false);
      return;
    }
    if (mode == "ais-removed") {
      context->Remove(object, false);
      return;
    }
    if (mode == "ais-recreated") {
      context->Remove(object, false);
      context->Display(object, false);
      return;
    }
    if (mode == "ais-erased-all") {
      context->EraseAll(false);
      return;
    }
    if (mode == "ais-displayed-all") {
      context->EraseAll(false);
      context->DisplayAll(false);
      return;
    }
    if (mode == "ais-removed-all") {
      context->RemoveAll(false);
      return;
    }
    if (mode == "ais-erased-selected" || mode == "ais-displayed-selected") {
      context->SetAutomaticHilight(false);
      if (mode == "ais-displayed-selected")
        context->Erase(object, false);
      context->AddSelect(object);
      if (mode == "ais-displayed-selected")
        context->DisplaySelected(false);
      else
        context->EraseSelected(false);
      return;
    }
    throw std::runtime_error("received AIS transition " + mode +
                             "; expected a supported ais-* capture mode");
  }
};
