// PrsDim relations in a context of a 400x300 view with a recording driver: the state of each
// construction, then property edits (position, arrow size, symbol, color, bounding box, value,
// automatic position, extension shape, text) each followed by the traced presentation and the
// selection, the twin of crates/tkv3d/examples/v3_probe/prs_rel.rs. A request is
// `v3_prsrel Kind Variant Count (Op A B C)...`.
#include <PrsDim_Chamf2dDimension.hxx>
#include <PrsDim_Chamf3dDimension.hxx>
#include <Geom_Plane.hxx>
#include <PrsDim_EqualRadiusRelation.hxx>
#include <PrsDim_ConcentricRelation.hxx>
#include <PrsDim_PerpendicularRelation.hxx>
#include <PrsDim_ParallelRelation.hxx>
#include <PrsDim_MaxRadiusDimension.hxx>
#include <PrsDim_MinRadiusDimension.hxx>
#include <Geom_Ellipse.hxx>
#include <PrsDim_OffsetDimension.hxx>
#include <PrsDim_TangentRelation.hxx>
#include <PrsDim_FixRelation.hxx>
#include <PrsDim_SymmetricRelation.hxx>
#include <PrsDim_MidPointRelation.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>

static TopoDS_Edge v3pr_edge(double x1, double y1, double z1, double x2, double y2, double z2) {
  return BRepBuilderAPI_MakeEdge(gp_Pnt(x1, y1, z1), gp_Pnt(x2, y2, z2)).Edge();
}
static TopoDS_Face v3pr_square(double z, double size) {
  return BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, z), gp::DZ()), 0.0, size, 0.0, size).Face();
}
static TopoDS_Face v3pr_cylinder() {
  return BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(gp_Pnt(0, 0, 0), gp::DZ()), 2.0), 0.0, M_PI, 0.0, 4.0).Face();
}
static occ::handle<Geom_Plane> v3pr_plane() { return new Geom_Plane(gp_Pln(gp::XOY())); }
static TCollection_ExtendedString v3pr_text(int index) {
  return TCollection_ExtendedString(index == 0 ? "" : index == 1 ? "C2" : "chamfer");
}
static occ::handle<PrsDim_Relation> v3pr_chamf2d(int variant) {
  const TopoDS_Edge e = v3pr_edge(0, 0, 0, 6, 8, 0);
  switch (variant) {
    case 0: return new PrsDim_Chamf2dDimension(e, v3pr_plane(), 10.0, v3pr_text(1));
    case 1: return new PrsDim_Chamf2dDimension(e, v3pr_plane(), 120.0, v3pr_text(2));
    case 2: return new PrsDim_Chamf2dDimension(e, v3pr_plane(), 10.0, v3pr_text(1), gp_Pnt(20, 1, 0), DsgPrs_AS_BOTHAR, 3.0);
    default: return new PrsDim_Chamf2dDimension(e, v3pr_plane(), 10.0, v3pr_text(0), gp_Pnt(3, 4, 0), DsgPrs_AS_FIRSTPT, 0.5);
  }
}
static occ::handle<PrsDim_Relation> v3pr_chamf3d(int variant) {
  switch (variant) {
    case 0: return new PrsDim_Chamf3dDimension(v3pr_square(1.0, 4.0), 10.0, v3pr_text(1));
    case 1: return new PrsDim_Chamf3dDimension(v3pr_cylinder(), 60.0, v3pr_text(2));
    case 2: return new PrsDim_Chamf3dDimension(v3pr_square(1.0, 4.0), 10.0, v3pr_text(1), gp_Pnt(2, 2, 30), DsgPrs_AS_LASTPT, 2.0);
    default: return new PrsDim_Chamf3dDimension(v3pr_cylinder(), 10.0, v3pr_text(0), gp_Pnt(9, 9, 9), DsgPrs_AS_NONE, 12.0);
  }
}
static TopoDS_Edge v3pr_arc(double x, double z, double radius, double first, double last) {
  return BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(x, 0, z), gp::DZ()), radius), first, last).Edge();
}
static TopoDS_Vertex v3pr_vertex(double x, double y, double z) {
  return BRepBuilderAPI_MakeVertex(gp_Pnt(x, y, z)).Vertex();
}
static occ::handle<PrsDim_Relation> v3pr_equal_radius(int variant) {
  switch (variant) {
    case 0: return new PrsDim_EqualRadiusRelation(v3pr_arc(0, 0, 3, 0, M_PI / 2), v3pr_arc(10, 0, 3, 0, M_PI), v3pr_plane());
    case 1: return new PrsDim_EqualRadiusRelation(v3pr_arc(0, 0, 3, 0, M_PI / 2), v3pr_arc(10, 2, 3, 1, 4), v3pr_plane());
    default: return new PrsDim_EqualRadiusRelation(v3pr_arc(0, 0, 4, 0, 2 * M_PI), v3pr_arc(-9, 0, 4, 0, 2 * M_PI), v3pr_plane());
  }
}
static occ::handle<PrsDim_Relation> v3pr_concentric(int variant) {
  switch (variant) {
    case 0: return new PrsDim_ConcentricRelation(v3pr_arc(1, 0, 3, 0, M_PI), v3pr_arc(1, 0, 5, 1, 3), v3pr_plane());
    case 1: return new PrsDim_ConcentricRelation(v3pr_arc(1, 0, 3, 0, M_PI), v3pr_vertex(1, 0, 0), v3pr_plane());
    case 2: return new PrsDim_ConcentricRelation(v3pr_vertex(1, 0, 0), v3pr_arc(1, 0, 90, 0, 2 * M_PI), v3pr_plane());
    case 3: return new PrsDim_ConcentricRelation(v3pr_vertex(1, 0, 0), v3pr_vertex(1, 0, 0), v3pr_plane());
    case 4: return new PrsDim_ConcentricRelation(v3pr_arc(1, 0, 3, 0, M_PI), v3pr_arc(1, 4, 5, 1, 3), v3pr_plane());
    case 5: return new PrsDim_ConcentricRelation(v3pr_arc(1, 0, 3, 0, M_PI), v3pr_vertex(1, 0, 6), v3pr_plane());
    default: return new PrsDim_ConcentricRelation(v3pr_vertex(1, 2, 3), v3pr_vertex(1, 0, 0), v3pr_plane());
  }
}
static TopoDS_Edge v3pr_ellipse() {
  return BRepBuilderAPI_MakeEdge(gp_Elips(gp_Ax2(gp_Pnt(0, 4, 0), gp::DZ(), gp::DX()), 5.0, 3.0)).Edge();
}
static TopoDS_Edge v3pr_infinite(double y) {
  return BRepBuilderAPI_MakeEdge(gp_Lin(gp_Pnt(0, y, 0), gp::DX())).Edge();
}
static occ::handle<PrsDim_Relation> v3pr_perpendicular(int variant) {
  switch (variant) {
    case 0: return new PrsDim_PerpendicularRelation(v3pr_edge(0, 0, 0, 10, 0, 0), v3pr_edge(3, -2, 0, 3, 8, 0), v3pr_plane());
    case 1: return new PrsDim_PerpendicularRelation(v3pr_edge(0, 0, 0, 5, 0, 0), v3pr_edge(8, 1, 0, 8, 9, 0), v3pr_plane());
    case 2: return new PrsDim_PerpendicularRelation(v3pr_ellipse(), v3pr_edge(2, -3, 0, 2, 12, 0), v3pr_plane());
    case 3: return new PrsDim_PerpendicularRelation(v3pr_edge(0, 0, 0, 10, 0, 0), v3pr_edge(3, -2, 3, 3, 8, 3), v3pr_plane());
    case 4: return new PrsDim_PerpendicularRelation(v3pr_square(0, 4), v3pr_square(1, 4), v3pr_plane());
    case 5: return new PrsDim_PerpendicularRelation(v3pr_edge(0, 0, 0, 10, 0, 0), v3pr_edge(3, -2, 0, 3, 8, 0));
    default: return new PrsDim_PerpendicularRelation(v3pr_infinite(1), v3pr_edge(3, -2, 0, 3, 8, 0), v3pr_plane());
  }
}
static occ::handle<PrsDim_Relation> v3pr_parallel(int variant) {
  switch (variant) {
    case 0: return new PrsDim_ParallelRelation(v3pr_edge(0, 0, 0, 10, 0, 0), v3pr_edge(2, 5, 0, 12, 5, 0), v3pr_plane());
    case 1: return new PrsDim_ParallelRelation(v3pr_edge(0, 0, 0, 10, 0, 0), v3pr_edge(2, 5, 0, 12, 5, 0), v3pr_plane(), gp_Pnt(5, 9, 0), DsgPrs_AS_BOTHPT, 2.0);
    case 2: return new PrsDim_ParallelRelation(v3pr_ellipse(), v3pr_edge(-6, -3, 0, 7, -3, 0), v3pr_plane());
    case 3: return new PrsDim_ParallelRelation(v3pr_edge(0, 0, 0, 10, 0, 0), v3pr_edge(2, 5, 3, 12, 5, 3), v3pr_plane());
    case 4: return new PrsDim_ParallelRelation(v3pr_infinite(-2), v3pr_edge(2, 5, 0, 12, 5, 0), v3pr_plane());
    case 5: return new PrsDim_ParallelRelation(v3pr_square(0, 4), v3pr_square(1, 4), v3pr_plane());
    default: return new PrsDim_ParallelRelation(v3pr_edge(0, 0, 0, 10, 0, 0), v3pr_edge(12, 0, 0, 20, 0, 0), v3pr_plane());
  }
}
static gp_Elips v3pr_elips() { return gp_Elips(gp_Ax2(gp_Pnt(1, 2, 0), gp::DZ(), gp::DX()), 5.0, 3.0); }
static TopoDS_Shape v3pr_ellipse_shape(int index) {
  switch (index) {
    case 0: return BRepBuilderAPI_MakeEdge(v3pr_elips()).Edge();
    case 1: return BRepBuilderAPI_MakeEdge(v3pr_elips(), 0.5, 2.5).Edge();
    case 2: return BRepBuilderAPI_MakeFace(BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(v3pr_elips()).Edge()).Wire()).Face();
    default: {
      occ::handle<Geom_Surface> s = new Geom_SurfaceOfLinearExtrusion(new Geom_Ellipse(v3pr_elips()), gp::DZ());
      return BRepBuilderAPI_MakeFace(s, 0.0, 2 * M_PI, 0.0, 4.0, 1.0e-7).Face();
    }
  }
}
template <class T> static occ::handle<PrsDim_Relation> v3pr_radius(int variant, double value) {
  switch (variant) {
    case 0: return new T(v3pr_ellipse_shape(0), value, v3pr_text(1));
    case 1: return new T(v3pr_ellipse_shape(1), value, v3pr_text(1));
    case 2: return new T(v3pr_ellipse_shape(1), value, v3pr_text(2), gp_Pnt(2, 9, 0), DsgPrs_AS_BOTHAR, 2.0);
    case 3: return new T(v3pr_ellipse_shape(2), value, v3pr_text(1));
    case 4: return new T(v3pr_ellipse_shape(3), value, v3pr_text(1));
    case 5: return new T(v3pr_ellipse_shape(0), value, v3pr_text(0), gp_Pnt(0, 9, 0), DsgPrs_AS_FIRSTAR, 1.5);
    default: return new T(v3pr_ellipse_shape(0), 7.0 - value, v3pr_text(1));
  }
}
static TopoDS_Face v3pr_cylinder_at(double x, double radius) {
  return BRepBuilderAPI_MakeFace(gp_Cylinder(gp_Ax3(gp_Pnt(x, 0, 0), gp::DZ()), radius), 0.0, 2 * M_PI, 0.0, 4.0).Face();
}
static occ::handle<PrsDim_Relation> v3pr_offset(int variant) {
  switch (variant) {
    case 0: return new PrsDim_OffsetDimension(v3pr_square(0, 4), v3pr_square(5, 4), 5.0, v3pr_text(1));
    case 1: return new PrsDim_OffsetDimension(v3pr_square(0, 4), v3pr_square(5, 4), 0.0, v3pr_text(2));
    case 2: return new PrsDim_OffsetDimension(v3pr_cylinder_at(0, 2), v3pr_cylinder_at(3, 1), 3.0, v3pr_text(1));
    case 3: return new PrsDim_OffsetDimension(v3pr_cylinder_at(0, 2), v3pr_square(5, 4), 3.0, v3pr_text(1));
    default: return new PrsDim_OffsetDimension(v3pr_square(0, 4), v3pr_square(0, 2), 400.0, v3pr_text(1));
  }
}
static TopoDS_Edge v3pr_circle_edge(double x, double y, double radius) {
  return BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(x, y, 0), gp::DZ()), radius)).Edge();
}
static TopoDS_Edge v3pr_ellipse_at(double y) {
  return BRepBuilderAPI_MakeEdge(gp_Elips(gp_Ax2(gp_Pnt(0, y, 0), gp::DZ(), gp::DX()), 5.0, 3.0)).Edge();
}
static occ::handle<PrsDim_Relation> v3pr_tangent(int variant) {
  switch (variant) {
    case 0: return new PrsDim_TangentRelation(v3pr_circle_edge(0, 0, 3), v3pr_edge(-5, 3, 0, 5, 3, 0), v3pr_plane(), 0);
    case 1: return new PrsDim_TangentRelation(v3pr_edge(-5, 3, 0, 5, 3, 0), v3pr_circle_edge(0, 0, 3), v3pr_plane(), 1);
    case 2: return new PrsDim_TangentRelation(v3pr_circle_edge(0, 0, 3), v3pr_circle_edge(5, 0, 2), v3pr_plane(), 0);
    case 3: return new PrsDim_TangentRelation(v3pr_circle_edge(0, 0, 3), v3pr_circle_edge(0, 0, 5), v3pr_plane(), 0);
    case 4: return new PrsDim_TangentRelation(v3pr_edge(-6, 1, 0, 6, 1, 0), v3pr_ellipse_at(4), v3pr_plane(), 0);
    case 5: return new PrsDim_TangentRelation(v3pr_ellipse_at(4), v3pr_edge(-6, 1, 0, 6, 1, 0), v3pr_plane(), 0);
    case 6: return new PrsDim_TangentRelation(v3pr_circle_edge(0, 8, 1), v3pr_ellipse_at(4), v3pr_plane(), 0);
    case 7: return new PrsDim_TangentRelation(v3pr_ellipse_at(4), v3pr_circle_edge(0, 8, 1), v3pr_plane(), 0);
    case 8: return new PrsDim_TangentRelation(v3pr_ellipse_at(4), v3pr_ellipse_at(-2), v3pr_plane(), 0);
    default: {
      TopoDS_Vertex v0 = v3pr_vertex(0, 0, 0), v1 = v3pr_vertex(5, 0, 0), v2 = v3pr_vertex(8, 3, 0);
      TopoDS_Edge line = BRepBuilderAPI_MakeEdge(v0, v1).Edge();
      TopoDS_Edge arc = BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(5, 3, 0), gp::DZ()), 3.0), v1, v2).Edge();
      return new PrsDim_TangentRelation(line, arc, v3pr_plane(), 0);
    }
  }
}
static occ::handle<PrsDim_Relation> v3pr_fix(int variant) {
  const TopoDS_Edge line = v3pr_edge(0, 0, 0, 10, 0, 0);
  const TopoDS_Edge arc = v3pr_arc(1, 0, 3, 1, 4);
  switch (variant) {
    case 0: return new PrsDim_FixRelation(v3pr_vertex(1, 2, 0), v3pr_plane());
    case 1: return new PrsDim_FixRelation(v3pr_vertex(1, 2, 0), v3pr_plane(), gp_Pnt(3, 4, 0), 3.0);
    case 2: return new PrsDim_FixRelation(line, v3pr_plane());
    case 3: return new PrsDim_FixRelation(line, v3pr_plane(), gp_Pnt(5, 5, 0), 2.0);
    case 4: return new PrsDim_FixRelation(line, v3pr_plane(), gp_Pnt(15, 3, 0), 2.0);
    case 5: return new PrsDim_FixRelation(arc, v3pr_plane());
    case 6: return new PrsDim_FixRelation(arc, v3pr_plane(), gp_Pnt(1, -6, 0), 4.0);
    case 7: return new PrsDim_FixRelation(v3pr_arc(1, 0, 3, 0, 2 * M_PI), v3pr_plane());
    default: return new PrsDim_FixRelation(v3pr_square(0, 4), v3pr_plane());
  }
}
static occ::handle<PrsDim_Relation> v3pr_symmetric(int variant) {
  const TopoDS_Edge axis = v3pr_edge(0, -10, 0, 0, 10, 0);
  switch (variant) {
    case 0: return new PrsDim_SymmetricRelation(axis, v3pr_edge(2, 0, 0, 5, 4, 0), v3pr_edge(-2, 0, 0, -5, 4, 0), v3pr_plane());
    case 1: return new PrsDim_SymmetricRelation(axis, v3pr_edge(3, 0, 0, 0, 6, 0), v3pr_edge(-3, 0, 0, 0, 6, 0), v3pr_plane());
    case 2: return new PrsDim_SymmetricRelation(axis, v3pr_arc(4, 0, 1, 0, M_PI), v3pr_arc(-4, 0, 1, 0, M_PI), v3pr_plane());
    case 3: return new PrsDim_SymmetricRelation(axis, v3pr_vertex(3, 2, 0), v3pr_vertex(-3, 2, 0), v3pr_plane());
    case 4: return new PrsDim_SymmetricRelation(axis, v3pr_vertex(3, 2, 0), v3pr_vertex(-3, 2, 5), v3pr_plane());
    case 5: return new PrsDim_SymmetricRelation(axis, v3pr_vertex(3, 2, 0), v3pr_vertex(3, 2, 0), v3pr_plane());
    case 6: return new PrsDim_SymmetricRelation(axis, v3pr_edge(2, 0, 0, 5, 4, 0), v3pr_edge(-2, 0, 2, -5, 4, 2), v3pr_plane());
    case 7: return new PrsDim_SymmetricRelation(axis, BRepBuilderAPI_MakeEdge(gp_Lin(gp_Pnt(2, 0, 0), gp::DY())).Edge(), v3pr_edge(-2, -3, 0, -2, 3, 0), v3pr_plane());
    default: return new PrsDim_SymmetricRelation(axis, v3pr_square(0, 4), v3pr_square(1, 4), v3pr_plane());
  }
}
static occ::handle<PrsDim_Relation> v3pr_midpoint(int variant) {
  const TopoDS_Vertex tool = v3pr_vertex(0, 0, 0);
  switch (variant) {
    case 0: return new PrsDim_MidPointRelation(tool, v3pr_vertex(3, 1, 0), v3pr_vertex(-3, -1, 0), v3pr_plane());
    case 1: return new PrsDim_MidPointRelation(tool, v3pr_edge(2, -3, 0, 2, 3, 0), v3pr_edge(-2, -3, 0, -2, 3, 0), v3pr_plane());
    case 2: return new PrsDim_MidPointRelation(tool, v3pr_edge(2, 4, 0, 2, 8, 0), v3pr_edge(-2, -8, 0, -2, -4, 0), v3pr_plane());
    case 3: return new PrsDim_MidPointRelation(tool, BRepBuilderAPI_MakeEdge(gp_Lin(gp_Pnt(2, 0, 0), gp::DY())).Edge(),
      BRepBuilderAPI_MakeEdge(gp_Lin(gp_Pnt(-2, 0, 0), gp::DY())).Edge(), v3pr_plane());
    case 4: return new PrsDim_MidPointRelation(tool, v3pr_arc(5, 0, 3, 2.0, 4.5), v3pr_arc(-5, 0, 3, 5.5, 7.0), v3pr_plane());
    case 5: return new PrsDim_MidPointRelation(tool, v3pr_arc(5, 0, 3, 0, 2 * M_PI), v3pr_arc(-5, 0, 3, 0, 2 * M_PI), v3pr_plane());
    case 6: return new PrsDim_MidPointRelation(tool, v3pr_ellipse_at(4), BRepBuilderAPI_MakeEdge(gp_Elips(gp_Ax2(gp_Pnt(0, -4, 0), gp::DZ(), gp::DX()), 5.0, 3.0), 0.5, 2.5).Edge(), v3pr_plane());
    case 7: return new PrsDim_MidPointRelation(v3pr_vertex(0, 0, 3), v3pr_vertex(3, 1, 0), v3pr_vertex(-3, -1, 0), v3pr_plane());
    case 8: return new PrsDim_MidPointRelation(tool, v3pr_edge(2, -3, 2, 2, 3, 2), v3pr_vertex(-3, -1, 4), v3pr_plane());
    default: return new PrsDim_MidPointRelation(v3pr_edge(0, 0, 0, 1, 0, 0), v3pr_vertex(3, 1, 0), v3pr_vertex(-3, -1, 0), v3pr_plane());
  }
}
static occ::handle<PrsDim_Relation> v3pr_make(int kind, int variant) {
  switch (kind) {
    case 12: return v3pr_midpoint(variant);
    case 11: return v3pr_symmetric(variant);
    case 10: return v3pr_fix(variant);
    case 8: return v3pr_offset(variant);
    case 9: return v3pr_tangent(variant);
    case 6: return v3pr_radius<PrsDim_MaxRadiusDimension>(variant, 5.0);
    case 7: return v3pr_radius<PrsDim_MinRadiusDimension>(variant, 3.0);
    case 4: return v3pr_perpendicular(variant);
    case 5: return v3pr_parallel(variant);
    case 2: return v3pr_equal_radius(variant);
    case 3: return v3pr_concentric(variant);
    case 1: return v3pr_chamf3d(variant);
    default: return v3pr_chamf2d(variant);
  }
}
static void v3pr_state(const occ::handle<PrsDim_Relation>& r) {
  ga_real(r->Value()); const gp_Pnt& p = r->Position(); ga_real(p.X()); ga_real(p.Y()); ga_real(p.Z());
  ga_real(r->ArrowSize()); ga_flag(r->AutomaticPosition()); ga_int(r->SymbolPrs()); ga_int(r->ExtShape());
  ga_int(r->KindOfDimension()); ga_flag(r->IsMovable()); ga_flag(r->HasColor());
  ga_int(r->Text().Length()); ga_flag(r->Plane().IsNull());
}
static void v3pr_selection(const occ::handle<PrsDim_Relation>& r) {
  occ::handle<SelectMgr_Selection> s = new SelectMgr_Selection(0);
  GA_TRY({ static_cast<SelectMgr_SelectableObject*>(r.get())->ComputeSelection(s, 0);
    ga_int(s->Entities().Size());
    for (const auto& e : s->Entities()) {
      const occ::handle<Select3D_SensitiveEntity>& sens = e->BaseSensitive();
      ga_int(sens->NbSubElements()); ga_int(sens->OwnerId()->Priority());
      Select3D_BndBox3d b = sens->BoundingBox(); ga_flag(b.IsValid());
      if (b.IsValid()) for (double v : {b.CornerMin().x(), b.CornerMin().y(), b.CornerMin().z(), b.CornerMax().x(), b.CornerMax().y(), b.CornerMax().z()}) ga_real(v);
    } })
}
void v3_prsrel(std::istream& input) {
  GaReader r{input};
  const int kind = r.integer(), variant = r.integer();
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<V3d_View> view = viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow(); window->SetSize(400, 300); view->SetWindow(window);
  occ::handle<AIS_InteractiveContext> ctx = new AIS_InteractiveContext(viewer);
  occ::handle<PrsDim_Relation> rel;
  GA_TRY(rel = v3pr_make(kind, variant))
  if (rel.IsNull()) { ga_end(); return; }
  v3pr_state(rel);
  for (int count = r.integer(); count > 0; --count) {
    const int op = r.integer(), a = r.integer(), b = r.integer(), c = r.integer();
    switch (op) {
      case 0: rel->SetPosition(gp_Pnt(a, b, c)); break;
      case 1: rel->SetArrowSize(a / 10.0); break;
      case 2: rel->SetSymbolPrs(DsgPrs_ArrowSide(a)); break;
      case 3: rel->SetColor(Quantity_Color(a == 0 ? Quantity_NOC_RED : Quantity_NOC_GREEN)); break;
      case 4: rel->UnsetColor(); break;
      case 5: rel->SetBndBox(a, b, c, a + 20, b + 20, c + 20); break;
      case 6: rel->UnsetBndBox(); break;
      case 7: rel->SetValue(a / 10.0); break;
      case 8: rel->SetAutomaticPosition(a != 0); break;
      case 9: rel->SetExtShape(a); break;
      case 10: rel->SetText(v3pr_text(a)); break;
      case 16: { trace.events.clear();
        GA_TRY({ if (ctx->IsDisplayed(rel)) ctx->Redisplay(rel, false, true); else ctx->Display(rel, false);
          ga_int(rel->Presentations().Size()); ga_int((int)trace.events.size()); for (double e : trace.events) ga_real(e); })
        trace.events.clear(); GA_TRY(v3pr_selection(rel)) break; }
    }
    v3pr_state(rel);
  }
  ga_end();
}
