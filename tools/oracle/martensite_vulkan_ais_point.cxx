// Point datums exercise all native standard markers and live AIS interaction.
#include <AIS_Point.hxx>
#include <Graphic3d_ArrayOfPoints.hxx>
#include <Graphic3d_AspectMarker3d.hxx>
#include <Prs3d_PointAspect.hxx>
struct MartensiteOraclePointScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_Point> point;
  occ::handle<Graphic3d_Structure> batch;
  MartensiteOraclePointScene(const occ::handle<V3d_Viewer> &viewer,
                             double offset) {
    context = new AIS_InteractiveContext(viewer);
    point = new AIS_Point(new Geom_CartesianPoint(0, 0, 0));
    context->Display(point, false);
    gp_Trsf t;
    t.SetTranslation(gp_Vec(offset, 0, 0));
    point->SetLocalTransformation(t);
  }
  void transition(const std::string &mode, const occ::handle<V3d_View> &view) {
    const std::string markers[] = {
        "point-dot", "point-plus",  "point-star",   "point-x",
        "point-o",   "point-o-dot", "point-o-plus", "point-o-star",
        "point-o-x", "point-ring1", "point-ring2",  "point-ring3",
        "point-ball"};
    for (int i = 0; i < 13; ++i)
      if (mode == markers[i] && i != 1)
        point->SetMarker(Aspect_TypeOfMarker(i));
    if (mode == "point-empty")
      point->SetMarker(Aspect_TOM_EMPTY);
    if (mode == "point-color")
      point->SetColor(Quantity_NOC_RED);
    if (mode == "point-scale") {
      point->SetWidth(3);
      point->SetMarker(Aspect_TOM_PLUS);
    }
    if (mode == "point-unset") {
      point->SetMarker(Aspect_TOM_STAR);
      point->UnsetMarker();
    }
    if (mode == "point-moved") {
      point->SetComponent(new Geom_CartesianPoint(4, -2, 1));
      context->Redisplay(point, false);
    }
    if (mode == "point-erased")
      context->Erase(point, false);
    if (mode == "point-redisplayed") {
      context->Erase(point, false);
      context->Display(point, false);
    }
    if (mode == "point-selected")
      context->SetSelected(point, false);
    if (mode == "point-detected") {
      auto p = point->Transformation().TranslationPart();
      int x, y;
      view->Convert(p.X(), p.Y(), p.Z(), x, y);
      context->MoveTo(x, y, view, false);
      if (context->DetectedInteractive() != point)
        throw std::runtime_error("expected detected AIS_Point");
    }
    if (mode == "point-user") {
      occ::handle<NCollection_HArray1<uint8_t>> bits =
          new NCollection_HArray1<uint8_t>(1, 8);
      for (int i = 0; i < 8; ++i)
        bits->SetValue(i + 1, uint8_t(0xff << (7 - i)));
      point->Attributes()->SetPointAspect(
          new Prs3d_PointAspect(Quantity_NOC_RED, 8, 8, bits));
      context->Redisplay(point, false);
    }
    if (mode == "point-batch") {
      context->Erase(point, false);
      batch = new Graphic3d_Structure(view->Viewer()->StructureManager());
      auto group = batch->NewGroup();
      group->SetGroupPrimitivesAspect(new Graphic3d_AspectMarker3d(
          Aspect_TOM_PLUS, Quantity_NOC_YELLOW, 1));
      occ::handle<Graphic3d_ArrayOfPoints> vertices =
          new Graphic3d_ArrayOfPoints(1024);
      for (int i = 0; i < 1024; ++i)
        vertices->AddVertex(double(i % 32) * 0.5 - 8, double(i / 32) * 0.5 - 8,
                            0);
      group->AddPrimitiveArray(vertices);
      batch->SetTransformation(new TopLoc_Datum3D(point->Transformation()));
      batch->Display();
    }
  }
};
