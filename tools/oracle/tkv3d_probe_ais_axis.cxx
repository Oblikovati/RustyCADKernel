// Native AIS_Axis state and cached selection, including both units systems.
#include <AIS_Axis.hxx>
#include <DsgPrs_XYZAxisPresentation.hxx>
#include <Geom_Axis1Placement.hxx>
#include <Geom_Axis2Placement.hxx>
static void ais_axis_state(const occ::handle<AIS_Axis> &axis) {
  ga_int(axis->Signature());
  ga_int(axis->Type());
  ga_int(axis->TypeOfAxis());
  ga_flag(axis->IsXYZAxis());
  ga_flag(axis->IsInfinite());
  ga_flag(axis->HasColor());
  ga_flag(axis->HasWidth());
  ga_real(axis->Width());
  Quantity_Color color;
  axis->Color(color);
  ais_line_color(color);
  ga_flag(!axis->Axis2Placement().IsNull());
  auto component = axis->Component()->Position();
  ga_pnt(component.Location());
  auto d = component.Direction();
  ga_real(d.X());
  ga_real(d.Y());
  ga_real(d.Z());
  auto drawer = axis->Attributes();
  ga_flag(drawer->HasOwnLineAspect());
  auto line = drawer->LineAspect()->Aspect();
  ais_line_color(line->Color());
  ga_real(line->Width());
  ga_int(line->Type());
  for (auto part : {Prs3d_DatumParts_XAxis, Prs3d_DatumParts_YAxis,
                    Prs3d_DatumParts_ZAxis}) {
    auto datum = drawer->DatumAspect();
    ga_real(datum->AxisLength(part));
    auto style = datum->LineAspect(part)->Aspect();
    ais_line_color(style->Color());
    ga_real(style->Width());
  }
  ga_flag(drawer->DatumAspect()->ToDrawLabels());
  for (int mode : {-1, 0, 1, 3})
    ga_flag(axis->AcceptDisplayMode(mode));
}
static occ::handle<Geom_Axis2Placement> ais_axis_frame() {
  return new Geom_Axis2Placement(gp_Ax2());
}
static occ::handle<AIS_Axis> ais_axis_create(int kind, double length) {
  if (kind == 0)
    return new AIS_Axis(ais_line_component());
  if (kind == 4)
    return new AIS_Axis(new Geom_Axis1Placement(gp_Ax1()));
  if (kind == 5)
    return new AIS_Axis(gp_Ax1(), length);
  return new AIS_Axis(ais_axis_frame(), AIS_TypeOfAxis(kind));
}
static void ais_axis_select(const occ::handle<AIS_Axis> &axis, int mode) {
  occ::handle<SelectMgr_Selection> selection = new SelectMgr_Selection(mode);
  static_cast<SelectMgr_SelectableObject *>(axis.get())
      ->ComputeSelection(selection, mode);
  ga_int(selection->Entities().Size());
  for (const auto &item : selection->Entities()) {
    auto segment =
        occ::down_cast<Select3D_SensitiveSegment>(item->BaseSensitive());
    ga_pnt(segment->StartPoint());
    ga_pnt(segment->EndPoint());
    ga_int(segment->OwnerId()->Priority());
    ga_flag(segment->OwnerId()->Selectable() == axis);
  }
}
static void
ais_axis_operation(const occ::handle<AIS_Axis> &axis,
                   const occ::handle<PrsMgr_PresentationManager> &manager,
                   int op, double width) {
  switch (op) {
  case 0:
    axis->SetColor(Quantity_NOC_BLUE);
    break;
  case 1:
    axis->UnsetColor();
    break;
  case 2:
    axis->SetWidth(width);
    break;
  case 3:
    axis->UnsetWidth();
    break;
  case 4:
    axis->SetComponent(ais_line_component());
    break;
  case 5:
    axis->SetAxis1Placement(new Geom_Axis1Placement(gp_Ax1()));
    break;
  case 6:
  case 7:
  case 8:
    axis->SetAxis2Placement(ais_axis_frame(), AIS_TypeOfAxis(op - 5));
    break;
  case 9:
    axis->SetTypeOfAxis(AIS_TOAX_Unknown);
    break;
  case 10:
    ais_axis_select(axis, 0);
    break;
  case 11:
    ais_axis_select(axis, 3);
    break;
  case 12:
    UnitsAPI::SetLocalSystem(
        UnitsAPI::LocalSystem() == UnitsAPI_SI ? UnitsAPI_MDTV : UnitsAPI_SI);
    break;
  case 13:
    if (manager->HasPresentation(axis, 0))
      manager->Update(axis, 0);
    else
      manager->Presentation(axis, 0, true);
    break;
  case 14:
    axis->Attributes()->DatumAspect()->SetDrawLabels(false);
    break;
  case 15:
    axis->SetDisplayAspect(
        new Prs3d_LineAspect(Quantity_NOC_GREEN, Aspect_TOL_DASH, 2));
    break;
  case 17:
    axis->SetAxis2Placement(ais_axis_frame(), AIS_TOAX_Unknown);
    break;
  default:
    axis->Component()->SetLocation(gp_Pnt(6, 7, 8));
    break;
  }
}
void v3_ais_axis(std::istream &input) {
  GaReader r{input};
  int kind = r.integer(), system = r.integer();
  double length = r.real(), width = r.real();
  int count = r.integer();
  UnitsAPI::SetLocalSystem(system == 0 ? UnitsAPI_SI : UnitsAPI_MDTV);
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager =
      new PrsMgr_PresentationManager(viewer->StructureManager());
  GA_TRY(
      auto axis = ais_axis_create(kind, length);
      occ::handle<Prs3d_Drawer> linked = new Prs3d_Drawer();
      linked->SetupOwnDefaults(); axis->Attributes()->SetLink(linked);
      axis->Attributes()->SetMaximalParameterValue(12); ais_axis_state(axis);
      for (int i = 0; i < count; ++i) {
        int op = r.integer();
        GA_TRY(ais_axis_operation(axis, manager, op, width))
        ais_axis_state(axis);
      } v3_display_trace(trace);
      ga_int(trace.texts.size()); for (const auto &text : trace.texts) {
        ga_real(text->Height());
        ga_pnt(text->Position());
        ga_int(text->Text().ToCString()[0]);
      })
  ga_end();
}
void v3_xyz_axis(std::istream &input) {
  GaReader r{input};
  bool styled = r.flag(), label = r.flag();
  double length = r.real(), angle = r.real();
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<Prs3d_Presentation> prs =
      new Prs3d_Presentation(viewer->StructureManager());
  GA_TRY(occ::handle<Prs3d_LineAspect> line =
             new Prs3d_LineAspect(Quantity_NOC_RED, Aspect_TOL_SOLID, 2);
         occ::handle<Prs3d_ArrowAspect> arrow = new Prs3d_ArrowAspect();
         arrow->SetAngle(angle); arrow->SetColor(Quantity_NOC_BLUE);
         occ::handle<Prs3d_TextAspect> text = new Prs3d_TextAspect();
         text->SetColor(Quantity_NOC_GREEN); gp_Dir direction(1, 2, 3);
         gp_Pnt last(direction.XYZ() * length); if (styled)
             DsgPrs_XYZAxisPresentation::Add(prs, line, arrow, text, direction,
                                             length, label ? "X" : "", gp_Pnt(),
                                             last);
         else DsgPrs_XYZAxisPresentation::Add(
             prs, line, direction, length, label ? "X" : "", gp_Pnt(), last);)
  v3_display_trace(trace);
  ga_int(trace.texts.size());
  for (const auto &text : trace.texts) {
    ga_real(text->Height());
    ga_pnt(text->Position());
    ga_int(text->Text().ToCString()[0]);
  }
  ga_end();
}
