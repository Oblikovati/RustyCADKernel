// Real native AIS_Line state and presentation buffers, matching the Rust
// fixture.
#include <AIS_Line.hxx>
#include <Geom_CartesianPoint.hxx>
#include <Geom_Line.hxx>
#include <UnitsAPI.hxx>
static occ::handle<Geom_Line> ais_line_component() {
  return new Geom_Line(gp_Pnt(-3, -2, 0), gp_Dir(3, 2, 0));
}
static void ais_line_color(const Quantity_Color &color) {
  ga_real(color.Red());
  ga_real(color.Green());
  ga_real(color.Blue());
}
static void ais_line_state(const occ::handle<AIS_Line> &line) {
  ga_int(line->Signature());
  ga_int(line->Type());
  ga_flag(line->IsInfinite());
  ga_flag(line->HasColor());
  ga_flag(line->HasWidth());
  ga_real(line->Width());
  Quantity_Color color;
  line->Color(color);
  ais_line_color(color);
  auto drawer = line->Attributes();
  ga_flag(drawer->HasOwnLineAspect());
  auto aspect = drawer->LineAspect();
  ga_flag(!aspect.IsNull());
  if (!aspect.IsNull()) {
    ga_real(aspect->Aspect()->Width());
    ais_line_color(aspect->Aspect()->Color());
  }
  auto component = line->Line();
  ga_flag(!component.IsNull());
  if (!component.IsNull()) {
    ga_pnt(component->Position().Location());
    auto d = component->Position().Direction();
    ga_real(d.X());
    ga_real(d.Y());
    ga_real(d.Z());
  }
  auto a = line->StartPoint(), b = line->EndPoint();
  ga_flag(!a.IsNull());
  if (!a.IsNull())
    ga_pnt(a->Pnt());
  ga_flag(!b.IsNull());
  if (!b.IsNull())
    ga_pnt(b->Pnt());
}
static void ais_line_select(const occ::handle<AIS_Line> &line, int mode) {
  occ::handle<SelectMgr_Selection> selection = new SelectMgr_Selection(mode);
  static_cast<SelectMgr_SelectableObject *>(line.get())
      ->ComputeSelection(selection, mode);
  ga_int(selection->Entities().Size());
  for (const auto &sensitive : selection->Entities()) {
    auto segment =
        occ::down_cast<Select3D_SensitiveSegment>(sensitive->BaseSensitive());
    ga_pnt(segment->StartPoint());
    ga_pnt(segment->EndPoint());
    ga_int(segment->OwnerId()->Priority());
    ga_flag(segment->OwnerId()->Selectable() == line);
  }
}
static void ais_line_drawer(const occ::handle<AIS_Line> &line, int link) {
  auto own = line->Attributes();
  own->SetMaximalParameterValue(12);
  if (link == 0) {
    own->SetupOwnDefaults();
    return;
  }
  if (link == 2)
    return;
  occ::handle<Prs3d_Drawer> drawer = new Prs3d_Drawer();
  drawer->SetupOwnDefaults();
  drawer->LineAspect()->SetWidth(2.5);
  drawer->LineAspect()->SetColor(Quantity_NOC_GREEN);
  own->SetLink(drawer);
}
void v3_ais_line(std::istream &input) {
  GaReader r{input};
  bool segment = r.flag();
  int system = r.integer(), link = r.integer();
  double width = r.real();
  UnitsAPI::SetLocalSystem(system == 0 ? UnitsAPI_SI : UnitsAPI_MDTV);
  // Presentations retained by the line must die before their recording driver
  // trace.
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager =
      new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<Geom_CartesianPoint> a = new Geom_CartesianPoint(-3, -2, 0),
                                   b = new Geom_CartesianPoint(3, 2, 0);
  occ::handle<AIS_Line> line =
      segment ? new AIS_Line(a, b) : new AIS_Line(ais_line_component());
  // SetPoints receives a separate pair, as in the Rust retained-handle fixture.
  a = new Geom_CartesianPoint(-3, -2, 0);
  b = new Geom_CartesianPoint(3, 2, 0);
  ais_line_drawer(line, link);
  ais_line_state(line);
  int count = r.integer();
  for (int i = 0; i < count; ++i) {
    int op = r.integer();
    // OCCT 3d097a0 replaceWithNewLineAspect borrows a handle inside the
    // replaced wrapper. Retain that wrapper across the call so native
    // aspect-map construction cannot use freed storage.
    const auto aspectLifetime = line->Attributes()->LineAspect();
    GA_TRY(switch (op) {
      case 0:
        line->SetColor(Quantity_NOC_RED);
        break;
      case 1:
        line->UnsetColor();
        break;
      case 2:
        line->SetWidth(width);
        break;
      case 3:
        line->UnsetWidth();
        break;
      case 4:
        line->SetLine(ais_line_component());
        break;
      case 5:
        line->SetPoints(a, b);
        break;
      case 6:
        b->SetCoord(5, 1, 2);
        break;
      case 7:
        UnitsAPI::SetLocalSystem(UnitsAPI::LocalSystem() == UnitsAPI_SI
                                     ? UnitsAPI_MDTV
                                     : UnitsAPI_SI);
        break;
      case 8:
        if (!line->Attributes()->LineAspect().IsNull()) {
          if (manager->HasPresentation(line, 0))
            manager->Update(line, 0);
          else
            manager->Presentation(line, 0, true);
        }
        break;
      case 9:
        ais_line_select(line, 0);
        break;
      case 10:
        ais_line_select(line, 1);
        break;
      default:
        line->SetPoints(a, a);
        break;
    })
    ais_line_state(line);
  }
  v3_display_trace(trace);
  ga_end();
}
