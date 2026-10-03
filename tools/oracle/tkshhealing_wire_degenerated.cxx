#include <array>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Plane.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_Wire.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Edge.hxx>
#include <ShapeExtend_Status.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pnt.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Circ.hxx>
#include <gp.hxx>
#include <iostream>
#include <cmath>

static void check_invalid_flag_removal();
static void check_existing_singular_edge_replacement();

static void check_bulk_degenerated();



static void check_duplicate_degenerated_wrap();
static void check_fix_unready_preserves_analyzer_status();
static TopoDS_Edge make_apex_degenerated_edge(BRep_Builder& builder,
                                              const TopoDS_Face& face,
                                              const gp_Pnt& apex);

int main()
{
  check_invalid_flag_removal();
  check_existing_singular_edge_replacement();
  check_bulk_degenerated();
  check_duplicate_degenerated_wrap();
  check_fix_unready_preserves_analyzer_status();
}

static void check_fix_unready_preserves_analyzer_status()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.0e-7);
  TopoDS_Vertex vertex;
  builder.MakeVertex(vertex, gp_Pnt(1.0, 0.0, 0.0), 1.0e-7);
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  builder.Degenerated(edge, true);
  TopoDS_Vertex firstVertex = vertex;
  firstVertex.Orientation(TopAbs_FORWARD);
  TopoDS_Vertex lastVertex = vertex;
  lastVertex.Orientation(TopAbs_REVERSED);
  builder.Add(edge, firstVertex);
  builder.Add(edge, lastVertex);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(edge);
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  gp_Pnt2d first;
  gp_Pnt2d last;
  analyzer->CheckDegenerated(1, first, last);
  const bool priorFail = analyzer->LastCheckStatus(ShapeExtend_FAIL2);
  analyzer->SetFace(TopoDS_Face());
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  const bool changed = fixer.FixDegenerated(1);
  std::cout << "unready=" << changed << " prior_fail2=" << priorFail
            << " after_fail2=" << analyzer->LastCheckStatus(ShapeExtend_FAIL2) << '\n';
}

static TopoDS_Edge make_apex_degenerated_edge(BRep_Builder& builder,
                                              const TopoDS_Face& face,
                                              const gp_Pnt& apex)
{
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  TopoDS_Vertex vertex;
  builder.MakeVertex(vertex, apex, 1.0e-7);
  TopoDS_Vertex first = vertex;
  first.Orientation(TopAbs_FORWARD);
  TopoDS_Vertex last = vertex;
  last.Orientation(TopAbs_REVERSED);
  builder.Add(edge, first);
  builder.Add(edge, last);
  Handle(Geom2d_Line) pcurve = new Geom2d_Line(
    gp_Ax2d(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0)));
  builder.UpdateEdge(edge, pcurve, face, 1.0e-7);
  builder.Range(edge, face, 0.0, 2.0 * M_PI);
  return edge;
}

static void check_duplicate_degenerated_wrap()
{
  BRep_Builder builder;
  Handle(Geom_ConicalSurface) cone = new Geom_ConicalSurface(gp_Ax3(), 0.5, 0.0);
  TopoDS_Face face;
  builder.MakeFace(face, cone, 1.0e-7);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  TopoDS_Edge edge = make_apex_degenerated_edge(builder, face, cone->Apex());
  data->Add(edge);
  data->Add(edge);
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  gp_Pnt2d first;
  gp_Pnt2d last;
  const bool found = analyzer->CheckDegenerated(2, first, last);
  std::cout << "duplicate_check=" << found << " done2="
            << analyzer->LastCheckStatus(ShapeExtend_DONE2) << " done1="
            << analyzer->LastCheckStatus(ShapeExtend_DONE1) << '\n';
  const bool foundFirst = analyzer->CheckDegenerated(1, first, last);
  std::cout << "duplicate_first_check=" << foundFirst << " done2="
            << analyzer->LastCheckStatus(ShapeExtend_DONE2) << '\n';
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  const bool changed = fixer.FixDegenerated();
  std::cout << "duplicate=" << changed << " count=" << data->NbEdges()
            << " degenerated=" << BRep_Tool::Degenerated(data->Edge(1)) << " status2="
            << fixer.StatusDegenerated(ShapeExtend_DONE2) << "\n";
}













static void check_invalid_flag_removal()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.0e-7);
  TopoDS_Vertex vertex;
  builder.MakeVertex(vertex, gp_Pnt(1.0, 0.0, 0.0), 1.0e-7);
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  builder.Degenerated(edge, true);
  TopoDS_Vertex forward = vertex;
  forward.Orientation(TopAbs_FORWARD);
  TopoDS_Vertex reverse = vertex;
  reverse.Orientation(TopAbs_REVERSED);
  builder.Add(edge, forward);
  builder.Add(edge, reverse);

  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(edge);
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  bool changed = fixer.FixDegenerated(1);
  std::cout << "changed=" << changed << " count=" << data->NbEdges() << " done3="
            << fixer.LastFixStatus(ShapeExtend_DONE3) << " fail2="
            << analyzer->LastCheckStatus(ShapeExtend_FAIL2) << '\n';
}

static void check_existing_singular_edge_replacement()
{
  BRep_Builder builder;
  gp_Ax3 axis;
  Handle(Geom_ConicalSurface) cone = new Geom_ConicalSurface(axis, 0.5, 0.0);
  TopoDS_Face face;
  builder.MakeFace(face, cone, 1.0e-7);
  TopoDS_Vertex vertex;
  builder.MakeVertex(vertex, cone->Apex(), 1.0e-7);
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  TopoDS_Vertex forward = vertex;
  forward.Orientation(TopAbs_FORWARD);
  TopoDS_Vertex reverse = vertex;
  reverse.Orientation(TopAbs_REVERSED);
  builder.Add(edge, forward);
  builder.Add(edge, reverse);
  Handle(Geom2d_Line) pcurve = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.)));
  builder.UpdateEdge(edge, pcurve, face, 1.0e-7);
  builder.Range(edge, face, 0., 2. * M_PI);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(edge);
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  bool changed = fixer.FixDegenerated(1);
  std::cout << "replace=" << changed << " count=" << data->NbEdges() << " degenerated="
            << BRep_Tool::Degenerated(data->Edge(1)) << " done2="
            << fixer.LastFixStatus(ShapeExtend_DONE2) << "\n";
}

static void check_bulk_degenerated()
{
  BRep_Builder builder;
  Handle(Geom_ConicalSurface) cone = new Geom_ConicalSurface(gp_Ax3(), 0.5, 0.0);
  TopoDS_Face face;
  builder.MakeFace(face, cone, 1.0e-7);
  TopoDS_Vertex vertex;
  builder.MakeVertex(vertex, cone->Apex(), 1.0e-7);
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  TopoDS_Vertex forward = vertex;
  forward.Orientation(TopAbs_FORWARD);
  TopoDS_Vertex reverse = vertex;
  reverse.Orientation(TopAbs_REVERSED);
  builder.Add(edge, forward);
  builder.Add(edge, reverse);
  Handle(Geom2d_Line) pcurve = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0., 0.), gp_Dir2d(1., 0.)));
  builder.UpdateEdge(edge, pcurve, face, 1.0e-7);
  builder.Range(edge, face, 0., 2. * M_PI);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(edge);
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  bool changed = fixer.FixDegenerated();
  std::cout << "bulk=" << changed << " count=" << data->NbEdges() << " done2="
            << fixer.StatusDegenerated(ShapeExtend_DONE2) << '\n';
}
