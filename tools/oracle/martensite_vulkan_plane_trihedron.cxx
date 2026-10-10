// Native plane trihedron through AIS, sharing the Martensite camera/capture
// setup.
#include <AIS_PlaneTrihedron.hxx>
#include <Geom_Plane.hxx>
struct MartensiteOraclePlaneScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_PlaneTrihedron> object;
  MartensiteOraclePlaneScene(const occ::handle<V3d_Viewer> &viewer,
                             double offset, const std::string &mode) {
    UnitsAPI::SetLocalSystem(UnitsAPI_SI);
    context = new AIS_InteractiveContext(viewer);
    gp_Ax3 frame;
    frame.SetLocation(gp_Pnt(-3, -3, 0));
    if (mode == "plane-indirect")
      frame.YReverse();
    object = new AIS_PlaneTrihedron(new Geom_Plane(frame));
    object->SetLength(6);
    gp_Trsf transform;
    transform.SetTranslation(gp_Vec(offset, 0, 0));
    object->SetLocalTransformation(transform);
    context->Display(object, false);
  }
  void mutate(const std::string &mode) {
    if (mode == "plane-length")
      object->SetLength(4);
    if (mode == "plane-anisotropic")
      object->Attributes()->DatumAspect()->SetAxisLength(5, 3, 8);
    if (mode == "plane-labels") {
      object->SetXLabel("U");
      object->SetYLabel("V");
    }
    if (mode == "plane-empty-labels") {
      object->SetXLabel("");
      object->SetYLabel("");
    }
    if (mode == "plane-styles") {
      auto drawer = object->Attributes();
      drawer->DatumAspect()
          ->LineAspect(Prs3d_DatumParts_YAxis)
          ->SetColor(Quantity_NOC_GREEN);
      drawer->ArrowAspect()->SetColor(Quantity_NOC_RED);
      drawer->TextAspect()->SetColor(Quantity_NOC_CYAN);
    }
    if (mode == "plane-moved") {
      gp_Ax3 frame;
      frame.SetLocation(gp_Pnt(-2, 1, 0));
      object->SetComponent(new Geom_Plane(frame));
    }
    if (mode == "plane-mutated")
      object->Component()->SetLocation(gp_Pnt(-2, 1, 0));
  }
  void transition(const std::string &mode, const occ::handle<V3d_View> &view) {
    if (mode == "plane-color")
      object->SetColor(Quantity_NOC_RED);
    else if (mode == "plane-erased")
      context->Erase(object, false);
    else if (mode == "plane-redisplayed") {
      context->Erase(object, false);
      context->Display(object, false);
    } else if (mode == "plane-selected")
      context->SetSelected(object, false);
    else if (mode == "plane-detected") {
      auto offset = object->Transformation().TranslationPart();
      int x, y;
      view->Convert(offset.X(), offset.Y() - 3, offset.Z(), x, y);
      context->MoveTo(x, y, view, false);
      if (context->DetectedInteractive() != object)
        throw std::runtime_error("expected detected AIS_PlaneTrihedron");
    } else {
      mutate(mode);
      context->Redisplay(object, false);
    }
  }
};
