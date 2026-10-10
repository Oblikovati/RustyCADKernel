// Native plane trihedron owner snapshots and live plane presentation.
#include <AIS_PlaneTrihedron.hxx>
#include <Geom_Plane.hxx>
static occ::handle<Geom_Plane> plane_trihedron_plane(int kind) {
  gp_Ax3 frame(gp_Pnt(1, 2, 3), gp_Dir(0, 0, 1), gp_Dir(1, 0, 0));
  if (kind == 1)
    frame.YReverse();
  if (kind == 2)
    frame = gp_Ax3(gp_Pnt(-2, 4, 6), gp_Dir(1, 2, 3), gp_Dir(2, -1, 0));
  return new Geom_Plane(frame);
}
static void
plane_child_state(const occ::handle<SelectMgr_SelectableObject> &child) {
  auto ais = occ::down_cast<AIS_InteractiveObject>(child);
  ga_int(ais->Signature());
  if (ais->Signature() == 1)
    ga_pnt(occ::down_cast<AIS_Point>(ais)->Component()->Pnt());
  else {
    auto axis = occ::down_cast<AIS_Line>(ais)->Line()->Position();
    ga_pnt(axis.Location());
    auto d = axis.Direction();
    ga_real(d.X());
    ga_real(d.Y());
    ga_real(d.Z());
  }
}
static void
plane_trihedron_state(const occ::handle<AIS_PlaneTrihedron> &object) {
  ga_int(object->Signature());
  ga_int(object->Type());
  ga_flag(object->IsInfinite());
  ga_flag(object->HasColor());
  Quantity_Color color;
  object->Color(color);
  ais_line_color(color);
  ga_real(object->GetLength());
  auto datum = object->Attributes()->DatumAspect();
  ga_int(datum->DatumAxes());
  for (auto part : {Prs3d_DatumParts_XAxis, Prs3d_DatumParts_YAxis,
                    Prs3d_DatumParts_ZAxis}) {
    ga_real(datum->AxisLength(part));
    ais_line_color(datum->LineAspect(part)->Aspect()->Color());
  }
  for (int mode : {-1, 0, 1, 2, 3})
    ga_flag(object->AcceptDisplayMode(mode));
  plane_child_state(object->Position());
  plane_child_state(object->XAxis());
  plane_child_state(object->YAxis());
  ga_flag(object->Position() == object->Position());
  ga_flag(object->XAxis() == object->XAxis());
}
static void
plane_trihedron_select(const occ::handle<AIS_PlaneTrihedron> &object,
                       int mode) {
  occ::handle<SelectMgr_Selection> selection = new SelectMgr_Selection(mode);
  auto base = static_cast<SelectMgr_SelectableObject *>(object.get());
  if (mode < 0 || mode > 2)
    base->ComputeSelection(selection, 0);
  base->ComputeSelection(selection, mode);
  ga_int(selection->Entities().Size());
  for (const auto &item : selection->Entities()) {
    auto sensitive = item->BaseSensitive();
    auto owner = sensitive->OwnerId();
    ga_int(owner->Priority());
    ga_flag(owner->Selectable() == object);
    if (mode == 1)
      ga_pnt(occ::down_cast<Select3D_SensitivePoint>(sensitive)->Point());
    else {
      auto segment = occ::down_cast<Select3D_SensitiveSegment>(sensitive);
      ga_pnt(segment->StartPoint());
      ga_pnt(segment->EndPoint());
    }
    if (mode == 1 || mode == 2)
      plane_child_state(owner->Selectable());
  }
}
static void plane_trihedron_operation(
    const occ::handle<AIS_PlaneTrihedron> &object,
    const occ::handle<PrsMgr_PresentationManager> &manager, int op,
    double length) {
  switch (op) {
  case 0:
  case 1:
  case 2:
    plane_trihedron_select(object, op);
    break;
  case 3:
    plane_trihedron_select(object, -1);
    break;
  case 4:
    plane_trihedron_select(object, 99);
    break;
  case 5:
    object->SetLength(length);
    break;
  case 6:
    object->SetComponent(plane_trihedron_plane(2));
    break;
  case 7:
    object->Component()->SetLocation(gp_Pnt(7, 8, 9));
    break;
  case 8:
    object->SetColor(Quantity_NOC_RED);
    break;
  case 9:
    object->SetXLabel(TCollection_AsciiString("U\0tail", 6));
    object->SetYLabel("V");
    break;
  case 10:
    object->SetXLabel("");
    object->SetYLabel("");
    break;
  case 11:
    object->Attributes()->DatumAspect()->SetAxisLength(2, 3, 4);
    object->Attributes()
        ->DatumAspect()
        ->LineAspect(Prs3d_DatumParts_YAxis)
        ->SetColor(Quantity_NOC_GREEN);
    break;
  default:
    if (manager->HasPresentation(object, 0))
      manager->Update(object, 0);
    else
      manager->Presentation(object, 0, true);
    break;
  }
}
void v3_plane_trihedron(std::istream &input) {
  GaReader r{input};
  int kind = r.integer(), system = r.integer();
  double length = r.real();
  int count = r.integer();
  UnitsAPI::SetLocalSystem(system == 0 ? UnitsAPI_SI : UnitsAPI_MDTV);
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager =
      new PrsMgr_PresentationManager(viewer->StructureManager());
  GA_TRY(
      occ::handle<AIS_PlaneTrihedron> object =
          new AIS_PlaneTrihedron(plane_trihedron_plane(kind));
      occ::handle<Prs3d_Drawer> linked = new Prs3d_Drawer();
      linked->SetupOwnDefaults(); object->Attributes()->SetLink(linked);
      plane_trihedron_state(object); for (int i = 0; i < count; ++i) {
        int op = r.integer();
        GA_TRY(plane_trihedron_operation(object, manager, op, length))
        plane_trihedron_state(object);
      } v3_display_trace(trace);
      ga_int(trace.texts.size()); for (const auto &text : trace.texts) {
        ga_real(text->Height());
        ga_pnt(text->Position());
        ga_int(text->Text().ToCString()[0]);
      })
  ga_end();
}
