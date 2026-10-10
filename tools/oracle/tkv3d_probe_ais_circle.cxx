// Circle state and actual retained presentation callbacks at OCCT 3d097a0.
#include <AIS_Circle.hxx>
#include <Geom_Circle.hxx>
static occ::handle<Geom_Circle> circle_component(double radius) {
  return new Geom_Circle(gp_Ax2(), radius);
}
static void circle_state(const occ::handle<AIS_Circle> &object) {
  ga_int(object->Signature());
  ga_int(object->Type());
  ga_flag(object->IsInfinite());
  ga_flag(object->HasColor());
  ga_flag(object->HasWidth());
  ga_real(object->Width());
  Quantity_Color color;
  object->Color(color);
  ais_line_color(color);
  auto drawer = object->Attributes();
  ga_flag(drawer->HasOwnLineAspect());
  auto aspect = drawer->LineAspect();
  ga_flag(!aspect.IsNull());
  if (!aspect.IsNull()) {
    ga_real(aspect->Aspect()->Width());
    ais_line_color(aspect->Aspect()->Color());
  }
  double first, last;
  object->Parameters(first, last);
  ga_real(first);
  ga_real(last);
  ga_flag(object->IsFilledCircleSens());
  ga_real(object->Circle()->Radius());
  ga_pnt(object->Circle()->Location());
  ga_real(drawer->DeviationCoefficient());
  ga_flag(drawer->HasOwnDeviationCoefficient());
  for (int mode : {-1, 0, 1, 3})
    ga_flag(object->AcceptDisplayMode(mode));
}
static void circle_select(const occ::handle<AIS_Circle> &object, int mode) {
  occ::handle<SelectMgr_Selection> selection = new SelectMgr_Selection(mode);
  static_cast<SelectMgr_SelectableObject *>(object.get())
      ->ComputeSelection(selection, mode);
  ga_int(selection->Entities().Size());
  for (const auto &item : selection->Entities()) {
    auto entity = item->BaseSensitive();
    ga_int(entity->OwnerId()->Priority());
    ga_int(entity->SensitivityFactor());
    ga_flag(entity->OwnerId()->Selectable() == object);
    auto circle = occ::down_cast<Select3D_SensitiveCircle>(entity);
    ga_flag(!circle.IsNull());
    if (!circle.IsNull()) {
      ga_real(circle->Radius());
      ga_pnt(circle->Circle().Location());
    } else {
      auto points = occ::down_cast<Select3D_SensitivePoly>(entity)->Points3D();
      ga_int(points->Length());
      for (const auto &point : *points)
        ga_pnt(point);
    }
  }
}
static void
circle_operation(const occ::handle<AIS_Circle> &object,
                 const occ::handle<PrsMgr_PresentationManager> &manager, int op,
                 double width) {
  switch (op) {
  case 0:
    object->SetColor(Quantity_NOC_RED);
    break;
  case 1:
    object->UnsetColor();
    break;
  case 2:
    object->SetWidth(width);
    break;
  case 3:
    object->UnsetWidth();
    break;
  case 4:
    object->SetFirstParam(.5);
    break;
  case 5:
    object->SetLastParam(2);
    break;
  case 6:
    object->SetCircle(circle_component(2));
    break;
  case 7:
    object->Circle()->SetRadius(4);
    break;
  case 9:
    circle_select(object, 0);
    break;
  case 10:
    circle_select(object, 99);
    break;
  case 11:
    object->SetFilledCircleSens(!object->IsFilledCircleSens());
    break;
  case 12:
    object->Circle()->SetLocation(gp_Pnt(1, 2, 3));
    break;
  default:
    if (!object->Attributes()->LineAspect().IsNull()) {
      if (manager->HasPresentation(object, 0))
        manager->Update(object, 0);
      else
        manager->Presentation(object, 0, true);
    }
  }
}
void v3_ais_circle(std::istream &input) {
  GaReader r{input};
  int variant = r.integer();
  bool filled = r.flag();
  int link = r.integer();
  double first = r.real(), last = r.real(), width = r.real();
  int count = r.integer();
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager =
      new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<AIS_Circle> object =
      variant == 0 ? new AIS_Circle(circle_component(3))
                   : new AIS_Circle(circle_component(3), first, last, filled);
  if (link == 0)
    object->Attributes()->SetupOwnDefaults();
  if (link == 1) {
    occ::handle<Prs3d_Drawer> drawer = new Prs3d_Drawer();
    drawer->SetupOwnDefaults();
    drawer->LineAspect()->SetWidth(2.5);
    drawer->LineAspect()->SetColor(Quantity_NOC_GREEN);
    object->Attributes()->SetLink(drawer);
  }
  circle_state(object);
  for (int i = 0; i < count; ++i) {
    int op = r.integer();
    GA_TRY(circle_operation(object, manager, op, width)) circle_state(object);
  }
  v3_display_trace(trace);
  ga_end();
}
