// Labeled and unlabeled AIS_Axis objects through the native interactive
// context.
#include <AIS_Axis.hxx>
#include <Geom_Axis1Placement.hxx>
#include <Geom_Axis2Placement.hxx>
#include <Prs3d_DatumAspect.hxx>
#include <UnitsAPI.hxx>
struct MartensiteOracleAxisScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_Axis> axis;
  static occ::handle<Geom_Axis2Placement> frame(const gp_Pnt &p) {
    return new Geom_Axis2Placement(gp_Ax2(p, gp_Dir(0, 0, 1)));
  }
  static occ::handle<AIS_Axis> create(const std::string &mode) {
    UnitsAPI::SetLocalSystem(UnitsAPI_SI);
    gp_Ax1 ray(gp_Pnt(-4, -3, 0), gp_Dir(1, 0, 0));
    if (mode == "axis-line")
      return new AIS_Axis(new Geom_Line(gp_Pnt(-3, -2, 0), gp_Dir(3, 2, 0)));
    if (mode == "axis-axis1")
      return new AIS_Axis(new Geom_Axis1Placement(ray));
    if (mode == "axis-ray" || mode == "axis-ray-default")
      return new AIS_Axis(ray, mode == "axis-ray" ? 6 : -1);
    auto kind = mode == "axis-y"
                    ? AIS_TOAX_YAxis
                    : (mode == "axis-z" ? AIS_TOAX_ZAxis : AIS_TOAX_XAxis);
    auto position = frame(ray.Location());
    occ::handle<AIS_Axis> axis = new AIS_Axis(position, kind);
    axis->Attributes()->DatumAspect()->SetAxisLength(6, 6, 6);
    axis->SetAxis2Placement(position, kind);
    return axis;
  }
  MartensiteOracleAxisScene(const occ::handle<V3d_Viewer> &viewer,
                            double offset, const std::string &mode) {
    context = new AIS_InteractiveContext(viewer);
    axis = create(mode);
    axis->Attributes()->SetMaximalParameterValue(12);
    gp_Trsf transform;
    transform.SetTranslation(gp_Vec(offset, 0, 0));
    axis->SetLocalTransformation(transform);
    context->Display(axis, false);
  }
  void transition(const std::string &mode, const occ::handle<V3d_View> &view) {
    if (mode == "axis-color")
      axis->SetColor(Quantity_NOC_RED);
    if (mode == "axis-width")
      axis->SetWidth(3);
    if (mode == "axis-unset-color") {
      axis->SetColor(Quantity_NOC_RED);
      axis->UnsetColor();
    }
    if (mode == "axis-unset-width") {
      axis->SetWidth(3);
      axis->UnsetWidth();
    }
    if (mode == "axis-display-aspect") {
      axis->SetDisplayAspect(
          new Prs3d_LineAspect(Quantity_NOC_GREEN, Aspect_TOL_DASH, 2));
      context->Redisplay(axis, false);
    }
    if (mode == "axis-moved") {
      axis->SetAxis2Placement(frame(gp_Pnt(-2, 1, 0)), AIS_TOAX_YAxis);
      context->Redisplay(axis, false);
    }
    if (mode == "axis-no-labels") {
      axis->Attributes()->DatumAspect()->SetDrawLabels(false);
      context->Redisplay(axis, false);
    }
    if (mode == "axis-erased")
      context->Erase(axis, false);
    if (mode == "axis-redisplayed") {
      context->Erase(axis, false);
      context->Display(axis, false);
    }
    if (mode == "axis-selected")
      context->SetSelected(axis, false);
    if (mode == "axis-detected") {
      auto offset = axis->Transformation().TranslationPart();
      int x, y;
      view->Convert(offset.X() - 1, offset.Y() - 3, offset.Z(), x, y);
      context->MoveTo(x, y, view, false);
      if (context->DetectedInteractive() != axis)
        throw std::runtime_error("expected detected AIS_Axis");
    }
  }
};
