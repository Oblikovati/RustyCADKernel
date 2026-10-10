// Real native point lifecycle and standard geometric point adapter.
#include <AIS_Point.hxx>
#include <BRep_Tool.hxx>
#include <StdPrs_Point.hxx>
#include <StdPrs_ToolPoint.hxx>
#include <TopoDS_Vertex.hxx>
static void ais_point_color(const Quantity_Color &c) {
  ga_real(c.Red());
  ga_real(c.Green());
  ga_real(c.Blue());
}
static void ais_point_aspect(const occ::handle<Prs3d_Drawer> &d) {
  ga_flag(d->HasOwnPointAspect());
  auto a = d->PointAspect();
  ga_flag(!a.IsNull());
  if (a.IsNull())
    return;
  ais_point_color(a->Aspect()->Color());
  ga_int(a->Aspect()->Type());
  ga_real(a->Aspect()->Scale());
}
static void ais_point_state(const occ::handle<AIS_Point> &p) {
  ga_int(p->Signature());
  ga_int(p->Type());
  ga_flag(p->HasColor());
  ga_flag(p->HasWidth());
  ga_flag(p->HasMarker());
  ga_real(p->Width());
  Quantity_Color c;
  p->Color(c);
  ais_point_color(c);
  ga_pnt(p->Component()->Pnt());
  ais_point_aspect(p->Attributes());
  for (auto d : {p->HilightAttributes(), p->DynamicHilightAttributes()}) {
    ga_int(d->DisplayMode());
    ga_int(d->ZLayer());
    ais_point_color(d->Color());
    ais_point_aspect(d);
  }
  for (int m : {-99, -1, 0, 1, 3})
    ga_flag(p->AcceptDisplayMode(m));
}
static void ais_point_select(const occ::handle<AIS_Point> &p, int mode) {
  occ::handle<SelectMgr_Selection> s = new SelectMgr_Selection(mode);
  static_cast<SelectMgr_SelectableObject *>(p.get())->ComputeSelection(s, mode);
  ga_int(s->Entities().Size());
  for (const auto &e : s->Entities()) {
    auto q = occ::down_cast<Select3D_SensitivePoint>(e->BaseSensitive());
    ga_pnt(q->Point());
    ga_int(q->OwnerId()->Priority());
    ga_flag(q->OwnerId()->Selectable() == p);
  }
}
static void ais_point_drawer(const occ::handle<AIS_Point> &p, int link) {
  if (link == 0) {
    p->Attributes()->SetupOwnDefaults();
    return;
  }
  if (link == 2)
    return;
  occ::handle<Prs3d_Drawer> d = new Prs3d_Drawer();
  d->SetupOwnDefaults();
  d->PointAspect()->SetScale(2.5);
  d->PointAspect()->SetTypeOfMarker(Aspect_TOM_O);
  d->PointAspect()->SetColor(Quantity_NOC_GREEN);
  p->Attributes()->SetLink(d);
}
static void ais_point_present(const occ::handle<AIS_Point> &p,
                              const occ::handle<PrsMgr_PresentationManager> &m,
                              int mode) {
  if (mode == 0 && p->Attributes()->PointAspect().IsNull())
    return;
  if (m->HasPresentation(p, mode))
    m->Update(p, mode);
  else
    m->Presentation(p, mode, true);
}
void v3_ais_point(std::istream &input) {
  GaReader r{input};
  int link = r.integer();
  double width = r.real();
  auto kind = static_cast<Aspect_TypeOfMarker>(r.integer());
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager =
      new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<AIS_Point> p = new AIS_Point(new Geom_CartesianPoint(1, 2, 3));
  ais_point_drawer(p, link);
  ais_point_state(p);
  int count = r.integer();
  for (int i = 0; i < count; ++i) {
    int op = r.integer();
    GA_TRY(switch (op) {
      case 0:
        p->SetColor(Quantity_NOC_RED);
        break;
      case 1:
        p->UnsetColor();
        break;
      case 2:
        p->SetWidth(width);
        break;
      case 3:
        p->UnsetWidth();
        break;
      case 4:
        p->SetMarker(kind);
        break;
      case 5:
        p->UnsetMarker();
        break;
      case 6:
        p->SetComponent(new Geom_CartesianPoint(4, 5, 6));
        break;
      case 7:
        occ::down_cast<Geom_CartesianPoint>(p->Component())->SetCoord(-2, 3, 4);
        break;
      case 8:
        ais_point_present(p, manager, 0);
        break;
      case 9:
        ais_point_present(p, manager, -99);
        break;
      case 10:
        ais_point_present(p, manager, 1);
        break;
      case 11:
        ais_point_select(p, 0);
        break;
      case 12:
        ais_point_select(p, -99);
        break;
      case 13:
        ais_point_select(p, 7);
        break;
      default:
        ga_pnt(BRep_Tool::Pnt(p->Vertex()));
        break;
    })
    ais_point_state(p);
  }
  v3_display_trace(trace);
  ga_end();
}
void v3_standard_point(std::istream &input) {
  GaReader r{input};
  double x = r.real(), y = r.real(), z = r.real();
  occ::handle<Geom_Point> p = new Geom_CartesianPoint(x, y, z);
  StdPrs_ToolPoint::Coord(p, x, y, z);
  ga_real(x);
  ga_real(y);
  ga_real(z);
  x = r.real();
  y = r.real();
  z = r.real();
  double d = r.real();
  ga_flag(StdPrs_Point::Match(p, x, y, z, d));
  ga_end();
}
