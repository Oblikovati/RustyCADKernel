// PrsDim dimensions in a context of a 400x300 view with a recording driver: the measured
// geometry of each construction, the value, plane, label position, the traced presentation of
// every compute mode and the selection of every mode after property edits, the twin of
// crates/tkv3d/examples/v3_probe/prs_dim.rs. A request is
// `v3_prsdim Kind Variant Count (Op A B C)...`.
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <PrsDim_LengthDimension.hxx>
#include <PrsDim_RadiusDimension.hxx>
#include <PrsDim_DiameterDimension.hxx>
#include <PrsDim_AngleDimension.hxx>
#include <Prs3d_DimensionAspect.hxx>
#include <Aspect_NeutralWindow.hxx>
#include <Geom_Line.hxx>
#include <Geom_SurfaceOfRevolution.hxx>

static TopoDS_Edge v3pd_edge(double x1, double y1, double z1, double x2, double y2, double z2) {
  return BRepBuilderAPI_MakeEdge(gp_Pnt(x1, y1, z1), gp_Pnt(x2, y2, z2)).Edge();
}
static TopoDS_Face v3pd_face(double z, double size) {
  return BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, z), gp::DZ()), 0.0, size, 0.0, size).Face();
}
static gp_Pln v3pd_plane(int index) {
  switch (index) {
    case 1: return gp_Pln(gp::ZOX());
    case 2: return gp_Pln(gp::YOZ());
    default: return gp_Pln(gp::XOY());
  }
}
static occ::handle<PrsDim_Dimension> v3pd_length(int variant) {
  switch (variant) {
    case 0: return new PrsDim_LengthDimension(gp_Pnt(0, 0, 0), gp_Pnt(10, 0, 0), v3pd_plane(0));
    case 1: return new PrsDim_LengthDimension(gp_Pnt(1, 2, 0), gp_Pnt(1, 2, 0), v3pd_plane(0));
    case 2: return new PrsDim_LengthDimension(v3pd_edge(0, 0, 0, 5, 5, 0), v3pd_plane(0));
    case 3: return new PrsDim_LengthDimension(v3pd_edge(0, 0, 0, 10, 0, 0), v3pd_edge(0, 5, 0, 10, 5, 0), v3pd_plane(0));
    case 4: { occ::handle<PrsDim_LengthDimension> d = new PrsDim_LengthDimension();
      d->SetMeasuredShapes(v3pd_edge(0, 0, 0, 10, 0, 0), v3pd_edge(2, 5, 0, 12, 5, 0)); return d; }
    case 5: { occ::handle<PrsDim_LengthDimension> d = new PrsDim_LengthDimension();
      d->SetMeasuredShapes(v3pd_edge(0, 0, 0, 10, 0, 0), BRepBuilderAPI_MakeVertex(gp_Pnt(4, 6, 0)).Vertex()); return d; }
    case 6: return new PrsDim_LengthDimension(v3pd_face(0.0, 4.0), v3pd_face(5.0, 4.0));
    case 7: return new PrsDim_LengthDimension(v3pd_face(0.0, 4.0), v3pd_edge(1, 1, 3, 3, 1, 3));
    case 8: { occ::handle<PrsDim_LengthDimension> d = new PrsDim_LengthDimension();
      d->SetMeasuredShapes(BRepBuilderAPI_MakeVertex(gp_Pnt(0, 0, 0)).Vertex(), BRepBuilderAPI_MakeVertex(gp_Pnt(3, 4, 0)).Vertex()); return d; }
    case 9: { occ::handle<PrsDim_LengthDimension> d = new PrsDim_LengthDimension();
      d->SetMeasuredShapes(BRepBuilderAPI_MakeVertex(gp_Pnt(4, 6, 0)).Vertex(), v3pd_edge(0, 0, 0, 10, 0, 0)); return d; }
    default: return new PrsDim_LengthDimension(v3pd_edge(0, 0, 0, 10, 0, 0), v3pd_edge(0, 0, 3, 10, 0, 7), v3pd_plane(1));
  }
}
static gp_Circ v3pd_circle() { return gp_Circ(gp_Ax2(gp_Pnt(1, 2, 0), gp::DZ()), 3.0); }
static TopoDS_Edge v3pd_circle_edge(double first, double last) {
  return BRepBuilderAPI_MakeEdge(v3pd_circle(), first, last).Edge();
}
static TopoDS_Face v3pd_cylinder() {
  return BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(gp_Pnt(0, 0, 0), gp::DZ()), 2.0), 0.0, 2 * M_PI, 0.0, 4.0).Face();
}
static TopoDS_Face v3pd_disk() {
  return BRepBuilderAPI_MakeFace(BRepBuilderAPI_MakeWire(v3pd_circle_edge(0.0, 2 * M_PI)).Wire()).Face();
}
static occ::handle<PrsDim_Dimension> v3pd_radius(int variant) {
  switch (variant) {
    case 0: return new PrsDim_RadiusDimension(v3pd_circle());
    case 1: return new PrsDim_RadiusDimension(v3pd_circle(), gp_Pnt(1, 5, 0));
    case 2: return new PrsDim_RadiusDimension(v3pd_circle(), gp_Pnt(1, 2, 0));
    case 3: return new PrsDim_RadiusDimension(v3pd_circle_edge(0.0, 2 * M_PI));
    case 4: return new PrsDim_RadiusDimension(v3pd_circle_edge(0.5, 2.0));
    case 5: return new PrsDim_RadiusDimension(v3pd_disk());
    case 6: return new PrsDim_RadiusDimension(v3pd_cylinder());
    default: return new PrsDim_RadiusDimension(gp_Circ(gp_Ax2(gp_Pnt(1, 2, 0), gp::DZ()), 0.0));
  }
}
static occ::handle<PrsDim_Dimension> v3pd_diameter(int variant) {
  switch (variant) {
    case 0: return new PrsDim_DiameterDimension(v3pd_circle());
    case 1: return new PrsDim_DiameterDimension(v3pd_circle(), gp_Pln(gp_Pnt(1, 2, 0), gp::DZ()));
    case 2: return new PrsDim_DiameterDimension(v3pd_circle(), gp_Pln(gp_Pnt(1, 2, 0), gp::DY()));
    case 3: return new PrsDim_DiameterDimension(v3pd_circle_edge(0.0, 2 * M_PI));
    case 4: return new PrsDim_DiameterDimension(v3pd_circle_edge(0.5, 2.0));
    case 5: return new PrsDim_DiameterDimension(v3pd_cylinder());
    default: return new PrsDim_DiameterDimension(v3pd_circle_edge(0.0, 2 * M_PI), gp_Pln(gp_Pnt(1, 2, 0), gp::DX()));
  }
}
static TopoDS_Face v3pd_cone(double radius) {
  return BRepBuilderAPI_MakeFace(gp_Cone(gp_Ax3(gp_Pnt(0, 0, 0), gp::DZ()), M_PI / 6, radius), 0.0, 2 * M_PI, 0.0, 4.0).Face();
}
static TopoDS_Face v3pd_revolution() {
  occ::handle<Geom_Surface> s = new Geom_SurfaceOfRevolution(new Geom_Line(gp_Pnt(2, 0, 0), gp_Dir(1, 0, 1)), gp::OZ());
  return BRepBuilderAPI_MakeFace(s, 0.0, 2 * M_PI, 0.0, 3.0, 1.0e-7).Face();
}
static occ::handle<PrsDim_Dimension> v3pd_angle(int variant) {
  switch (variant) {
    case 0: return new PrsDim_AngleDimension(gp_Pnt(10, 0, 0), gp_Pnt(0, 0, 0), gp_Pnt(5, 5, 0));
    case 1: return new PrsDim_AngleDimension(gp_Pnt(10, 0, 0), gp_Pnt(0, 0, 0), gp_Pnt(-10, 0, 0));
    case 2: return new PrsDim_AngleDimension(gp_Pnt(0, 0, 0), gp_Pnt(0, 0, 0), gp_Pnt(5, 5, 0));
    case 3: return new PrsDim_AngleDimension(BRepBuilderAPI_MakeVertex(gp_Pnt(0, 5, 0)).Vertex(),
      BRepBuilderAPI_MakeVertex(gp_Pnt(0, 0, 0)).Vertex(), BRepBuilderAPI_MakeVertex(gp_Pnt(3, 0, 4)).Vertex());
    case 4: return new PrsDim_AngleDimension(v3pd_edge(2, 0, 0, 10, 0, 0), v3pd_edge(0, 3, 0, 0, 10, 0));
    case 5: return new PrsDim_AngleDimension(v3pd_edge(0, 0, 0, 10, 0, 0), v3pd_edge(10, 5, 0, 0, 5, 0));
    case 6: return new PrsDim_AngleDimension(v3pd_edge(0, 0, 0, 10, 0, 0), v3pd_edge(0, 0, 0, 10, 0, 0));
    case 7: return new PrsDim_AngleDimension(v3pd_face(0.0, 4.0), BRepBuilderAPI_MakeFace(gp_Pln(gp::ZOX()), 0.0, 4.0, 0.0, 4.0).Face());
    case 8: return new PrsDim_AngleDimension(v3pd_face(0.0, 4.0), BRepBuilderAPI_MakeFace(gp_Pln(gp::ZOX()), 0.0, 4.0, 0.0, 4.0).Face(), gp_Pnt(1, 3, 0));
    case 9: return new PrsDim_AngleDimension(v3pd_cone(2.0));
    case 10: return new PrsDim_AngleDimension(v3pd_revolution());
    case 11: return new PrsDim_AngleDimension(v3pd_cylinder(), v3pd_cone(1.0));
    default: return new PrsDim_AngleDimension(v3pd_circle_edge(0.0, 2.0), v3pd_edge(0, 0, 0, 10, 0, 0));
  }
}
static occ::handle<PrsDim_Dimension> v3pd_make(int kind, int variant) {
  switch (kind) {
    case 3: return v3pd_angle(variant);
    case 1: return v3pd_radius(variant);
    case 2: return v3pd_diameter(variant);
    default: return v3pd_length(variant);
  }
}
static void v3pd_point(const gp_Pnt& p) { ga_real(p.X()); ga_real(p.Y()); ga_real(p.Z()); }
static void v3pd_dir(const gp_Dir& d) { ga_real(d.X()); ga_real(d.Y()); ga_real(d.Z()); }
static void v3pd_state(const occ::handle<PrsDim_Dimension>& d) {
  ga_flag(d->IsValid()); ga_real(d->GetValue()); ga_real(d->GetFlyout());
  const gp_Pln& p = d->GetPlane(); v3pd_point(p.Location()); v3pd_dir(p.Axis().Direction()); v3pd_dir(p.XAxis().Direction());
  ga_int(d->GetGeometryType()); ga_int(d->KindOfDimension()); ga_flag(d->IsTextPositionCustom());
  // The 2D label width reads the context viewer: a null dereference without a context.
  ga_flag(d->HasInteractiveContext());
  if (d->HasInteractiveContext()) GA_TRY(v3pd_point(d->GetTextPosition()))
  ga_int(d->DisplaySpecialSymbol()); ga_int(d->SpecialSymbol());
}
static void v3pd_selection(const occ::handle<PrsDim_Dimension>& d) {
  for (int mode = 0; mode <= 3; ++mode) {
    occ::handle<SelectMgr_Selection> s = new SelectMgr_Selection(mode);
    GA_TRY({ static_cast<SelectMgr_SelectableObject*>(d.get())->ComputeSelection(s, mode);
      ga_int(s->Entities().Size());
      for (const auto& e : s->Entities()) {
        const occ::handle<Select3D_SensitiveEntity>& sens = e->BaseSensitive();
        ga_int(sens->NbSubElements()); ga_int(sens->OwnerId()->Priority());
        Select3D_BndBox3d b = sens->BoundingBox(); ga_flag(b.IsValid());
        if (b.IsValid()) for (double v : {b.CornerMin().x(), b.CornerMin().y(), b.CornerMin().z(), b.CornerMax().x(), b.CornerMax().y(), b.CornerMax().z()}) ga_real(v);
      } })
  }
}
void v3_prsdim(std::istream& input) {
  GaReader r{input};
  const int kind = r.integer(), variant = r.integer();
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<V3d_View> view = viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow(); window->SetSize(400, 300); view->SetWindow(window);
  occ::handle<AIS_InteractiveContext> ctx = new AIS_InteractiveContext(viewer);
  occ::handle<PrsDim_Dimension> d;
  GA_TRY(d = v3pd_make(kind, variant))
  if (d.IsNull()) { ga_end(); return; }
  v3pd_state(d);
  occ::handle<Prs3d_DimensionAspect> aspect = new Prs3d_DimensionAspect();
  for (int count = r.integer(); count > 0; --count) {
    const int op = r.integer(), a = r.integer(), b = r.integer(), c = r.integer();
    switch (op) {
      case 0: d->SetFlyout(a / 10.0); break;
      case 1: d->SetCustomValue(double(a)); break;
      case 2: d->SetComputedValue(); break;
      case 3: GA_TRY(d->SetTextPosition(gp_Pnt(a, b, c))) break;
      case 4: aspect->SetTextHorizontalPosition(Prs3d_DimensionTextHorizontalPosition(a)); d->SetDimensionAspect(aspect); break;
      case 5: aspect->SetTextVerticalPosition(Prs3d_DimensionTextVerticalPosition(a)); d->SetDimensionAspect(aspect); break;
      case 6: aspect->SetArrowOrientation(Prs3d_DimensionArrowOrientation(a)); d->SetDimensionAspect(aspect); break;
      case 7: aspect->MakeText3d(a != 0); d->SetDimensionAspect(aspect); break;
      case 8: aspect->MakeUnitsDisplayed(a != 0); d->SetDimensionAspect(aspect); break;
      case 9: d->SetCustomPlane(v3pd_plane(a)); break;
      case 10: { occ::handle<PrsDim_LengthDimension> l = occ::down_cast<PrsDim_LengthDimension>(d);
        if (!l.IsNull()) l->SetDirection(a == 0 ? gp::DX() : a == 1 ? gp::DY() : gp::DZ(), b != 0); break; }
      case 11: d->SetDisplaySpecialSymbol(PrsDim_DisplaySpecialSymbol(a)); d->SetSpecialSymbol(char16_t(b)); break;
      case 12: aspect->SetValueStringFormat(a == 0 ? "%g" : a == 1 ? "%.2f" : "%5.1e"); d->SetDimensionAspect(aspect); break;
      case 13: aspect->MakeArrows3d(a != 0); d->SetDimensionAspect(aspect); break;
      case 14: d->UnsetFixedTextPosition(); break;
      case 15: d->SetSelToleranceForText2d(a / 10.0); break;
      case 16: { trace.events.clear();
        GA_TRY({ if (ctx->IsDisplayed(d)) ctx->Redisplay(d, false, true); else ctx->Display(d, false);
          ga_int(d->Presentations().Size()); ga_int((int)trace.events.size()); for (double e : trace.events) ga_real(e); })
        trace.events.clear(); GA_TRY(v3pd_selection(d)) break; }
      case 17: aspect->SetExtensionSize(a / 10.0); aspect->SetArrowTailSize(b / 10.0); d->SetDimensionAspect(aspect); break;
      case 19: { occ::handle<PrsDim_AngleDimension> g = occ::down_cast<PrsDim_AngleDimension>(d);
        if (!g.IsNull()) g->SetType(PrsDim_TypeOfAngle(a)); break; }
      case 20: { occ::handle<PrsDim_AngleDimension> g = occ::down_cast<PrsDim_AngleDimension>(d);
        if (!g.IsNull()) g->SetArrowsVisibility(PrsDim_TypeOfAngleArrowVisibility(a)); break; }
      case 18: aspect->ArrowAspect()->SetLength(a / 10.0); aspect->ArrowAspect()->SetZoomable(b != 0); d->SetDimensionAspect(aspect); break;
    }
    v3pd_state(d);
  }
  ga_end();
}
