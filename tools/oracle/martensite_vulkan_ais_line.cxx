// Built-in AIS_Line oracle scenes retain the native context and appearance
// behavior.
#include <AIS_Line.hxx>
#include <Geom_CartesianPoint.hxx>
#include <Geom_Line.hxx>
struct MartensiteOracleLineScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_Line> line;
  MartensiteOracleLineScene(const occ::handle<V3d_Viewer> &viewer,
                            double offset, bool infinite) {
    context = new AIS_InteractiveContext(viewer);
    if (infinite)
      line = new AIS_Line(new Geom_Line(gp_Pnt(-3, -2, 0), gp_Dir(3, 2, 0)));
    else
      line = new AIS_Line(new Geom_CartesianPoint(-3, -2, 0),
                          new Geom_CartesianPoint(3, 2, 0));
    line->Attributes()->SetMaximalParameterValue(12);
    context->Display(line, false);
    gp_Trsf transform;
    transform.SetTranslation(gp_Vec(offset, 0, 0));
    line->SetLocalTransformation(transform);
  }
  void transition(const std::string &mode, const occ::handle<V3d_View> &view) {
    if (mode == "datum-color")
      line->SetColor(Quantity_NOC_RED);
    if (mode == "datum-width")
      line->SetWidth(3);
    if (mode == "datum-unset-color") {
      line->SetWidth(3);
      line->SetColor(Quantity_NOC_RED);
      const auto retained = line->Attributes()->LineAspect();
      line->UnsetColor();
    }
    if (mode == "datum-unset-width") {
      line->SetColor(Quantity_NOC_RED);
      line->SetWidth(3);
      line->UnsetWidth();
    }
    if (mode == "datum-moved") {
      line->SetPoints(new Geom_CartesianPoint(-4, 2, 0),
                      new Geom_CartesianPoint(4, -1, 0));
      context->Redisplay(line, false);
    }
    if (mode == "datum-erased")
      context->Erase(line, false);
    if (mode == "datum-redisplayed") {
      context->Erase(line, false);
      context->Display(line, false);
    }
    if (mode == "datum-detected") {
      auto origin = line->Transformation().TranslationPart();
      int x, y;
      view->Convert(origin.X(), origin.Y(), origin.Z(), x, y);
      context->MoveTo(x, y, view, false);
      if (context->DetectedInteractive() != line)
        throw std::runtime_error("expected detected AIS_Line");
    }
    if (mode == "datum-selected")
      context->SetSelected(line, false);
  }
};
