// Native FreeType labels, corresponding to Martensite text_scene.rs.
#include <Graphic3d_AspectText3d.hxx>
#include <Graphic3d_Text.hxx>
struct MartensiteOracleTextScene {
  occ::handle<Graphic3d_Structure> structure;
  occ::handle<Graphic3d_Group> group;
  occ::handle<Graphic3d_Text> text;
  occ::handle<Graphic3d_AspectText3d> style;
  MartensiteOracleTextScene(const occ::handle<V3d_Viewer> &viewer,
                            double offset) {
    structure = new Graphic3d_Structure(viewer->StructureManager());
    group = structure->NewGroup();
    style = new Graphic3d_AspectText3d(Quantity_NOC_WHITE, "DejaVu Sans", 1, 0,
                                       Aspect_TOST_NORMAL, Aspect_TODT_NORMAL);
    text = new Graphic3d_Text(16);
    text->SetText("AV");
    text->SetPosition(gp_Pnt(-5, 0, 0));
    gp_Trsf tr;
    tr.SetTranslation(gp_Vec(offset, 0, 0));
    structure->SetTransformation(new TopLoc_Datum3D(tr));
    structure->Display();
  }
  void apply(const std::string &mode, const occ::handle<V3d_View> &view) {
    if (mode == "text-unicode")
      text->SetText("AΩБ");
    if (mode == "text-multiline")
      text->SetText("A\nV");
    if (mode == "text-tabs")
      text->SetText("A\tV");
    if (mode == "text-centered") {
      text->SetHorizontalAlignment(Graphic3d_HTA_CENTER);
      text->SetVerticalAlignment(Graphic3d_VTA_CENTER);
    }
    if (mode == "text-top") {
      text->SetHorizontalAlignment(Graphic3d_HTA_RIGHT);
      text->SetVerticalAlignment(Graphic3d_VTA_TOP);
    }
    if (mode == "text-rotated")
      style->SetTextAngle(30);
    if (mode == "text-zoomable")
      style->SetTextZoomable(true);
    if (mode == "text-color")
      style->SetColor(Quantity_NOC_RED);
    if (mode == "text-height")
      text->SetHeight(24);
    if (mode == "text-default-height")
      text->SetHeight(1);
    if (mode == "text-empty")
      text->SetText("");
    if (mode == "text-plane" || mode == "text-plane-local") {
      text->SetOrientation(gp_Ax2(gp_Pnt(-5, 0, 0), gp_Dir(0, 1, 1)));
      text->SetOwnAnchorPoint(mode == "text-plane");
    }
    if (mode == "text-depth" || mode == "text-annotation") {
      group->SetPrimitivesAspect(unlit_aspects());
      group->AddPrimitiveArray(triangle_vertices());
      text->SetPosition(gp_Pnt(-5, 0, -1));
      if (mode == "text-annotation")
        style->SetStyle(Aspect_TOST_ANNOTATION);
    }
    if (mode == "text-resolution")
      view->ChangeRenderingParams().Resolution = 144;
    if (mode == "text-highlight") {
      occ::handle<Graphic3d_PresentationAttributes> h =
          new Graphic3d_PresentationAttributes();
      h->SetColor(Quantity_NOC_GREEN);
      structure->Highlight(h);
    }
    group->SetPrimitivesAspect(style);
    group->AddText(text);
  }
};
