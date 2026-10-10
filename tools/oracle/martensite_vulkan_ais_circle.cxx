// Native circular and arc datums with live appearance and sensitivity.
#include <AIS_Circle.hxx>
#include <Geom_Circle.hxx>
struct MartensiteOracleCircleScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_Circle> circle;
  static occ::handle<Geom_Circle> component(double radius) {
    return new Geom_Circle(gp_Ax2(), radius);
  }
  static occ::handle<AIS_Circle> create(const std::string &mode) {
    if (mode == "circle-arc" || mode == "circle-arc-filled")
      return new AIS_Circle(component(5), .25, 4.5,
                            mode == "circle-arc-filled");
    if (mode == "circle-reverse-full")
      return new AIS_Circle(component(5), 2 * M_PI, 0, false);
    if (mode == "circle-filled" || mode == "circle-filled-detected")
      return new AIS_Circle(component(5), 0, 2 * M_PI, true);
    return new AIS_Circle(component(5));
  }
  MartensiteOracleCircleScene(const occ::handle<V3d_Viewer> &viewer,
                              double offset, const std::string &mode) {
    context = new AIS_InteractiveContext(viewer);
    circle = create(mode);
    gp_Trsf trsf;
    trsf.SetTranslation(gp_Vec(offset, 0, 0));
    circle->SetLocalTransformation(trsf);
    context->Display(circle, false);
  }
  void transition(const std::string &mode, const occ::handle<V3d_View> &view) {
    if (mode == "circle-color")
      circle->SetColor(Quantity_NOC_RED);
    if (mode == "circle-width")
      circle->SetWidth(3);
    if (mode == "circle-unset-color") {
      circle->SetColor(Quantity_NOC_RED);
      circle->UnsetColor();
    }
    if (mode == "circle-unset-width") {
      circle->SetWidth(3);
      circle->UnsetWidth();
    }
    if (mode == "circle-replaced") {
      circle->SetCircle(component(3));
      context->Redisplay(circle, false);
    }
    if (mode == "circle-radius") {
      circle->Circle()->SetRadius(3);
      context->Redisplay(circle, false);
    }
    if (mode == "circle-moved") {
      circle->Circle()->SetLocation(gp_Pnt(-2, 1, 0));
      context->Redisplay(circle, false);
    }
    if (mode == "circle-trim") {
      circle->SetFirstParam(.5);
      circle->SetLastParam(3.5);
      context->Redisplay(circle, false);
    }
    if (mode == "circle-absolute") {
      circle->Attributes()->SetTypeOfDeflection(Aspect_TOD_ABSOLUTE);
      circle->Attributes()->SetMaximalChordialDeviation(.25);
      context->Redisplay(circle, false);
    }
    if (mode == "circle-erased")
      context->Erase(circle, false);
    if (mode == "circle-redisplayed") {
      context->Erase(circle, false);
      context->Display(circle, false);
    }
    if (mode == "circle-selected")
      context->SetSelected(circle, false);
    if (mode == "circle-detected" || mode == "circle-filled-detected") {
      auto offset = circle->Transformation().TranslationPart();
      int x, y;
      view->Convert(offset.X() + (mode == "circle-filled-detected" ? 0 : 5),
                    offset.Y(), offset.Z(), x, y);
      context->MoveTo(x, y, view, false);
      if (context->DetectedInteractive() != circle)
        throw std::runtime_error("expected detected AIS_Circle");
    }
  }
};
