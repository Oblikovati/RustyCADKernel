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
static void check_periodic_shift();
static void check_bulk_degenerated();

int main()
{
  check_invalid_flag_removal();
  check_existing_singular_edge_replacement();
  check_periodic_shift();
  check_bulk_degenerated();
}

static TopoDS_Edge make_periodic_edge(BRep_Builder& builder,
                                      const TopoDS_Face& face,
                                      const Handle(Geom_CylindricalSurface)& cylinder,
                                      double firstU,
                                      double lastU)
{
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  Handle(Geom2d_Line) pcurve = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(firstU, 0.), gp_Dir2d(1., 0.)));
  builder.UpdateEdge(edge, pcurve, face, 1.0e-7);
  builder.Range(edge, face, 0., lastU - firstU);
  TopoDS_Vertex first;
  TopoDS_Vertex last;
  builder.MakeVertex(first, cylinder->Value(firstU, 0.), 1.0e-7);
  builder.MakeVertex(last, cylinder->Value(lastU, 0.), 1.0e-7);
  TopoDS_Vertex orientedFirst = first;
  orientedFirst.Orientation(TopAbs_FORWARD);
  TopoDS_Vertex orientedLast = last;
  orientedLast.Orientation(TopAbs_REVERSED);
  builder.Add(edge, orientedFirst);
  builder.Add(edge, orientedLast);
  return edge;
}

static void check_periodic_shift()
{
  BRep_Builder builder;
  Handle(Geom_CylindricalSurface) cylinder = new Geom_CylindricalSurface(gp_Ax3(), 1.0);
  TopoDS_Face face;
  builder.MakeFace(face, cylinder, 1.0e-7);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(make_periodic_edge(builder, face, cylinder, 5.8, 6.0));
  data->Add(make_periodic_edge(builder, face, cylinder, -0.2, 0.0));
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  bool changed = fixer.FixShifted();
  ShapeAnalysis_Edge edgeAnalysis;
  Handle(Geom2d_Curve) firstCurve;
  double first;
  double last;
  edgeAnalysis.PCurve(data->Edge(1), face, firstCurve, first, last, true);
  gp_Pnt2d firstStart = firstCurve->Value(first);
  gp_Pnt2d firstEnd = firstCurve->Value(last);
  edgeAnalysis.PCurve(data->Edge(2), face, firstCurve, first, last, true);
  gp_Pnt2d secondStart = firstCurve->Value(first);
  gp_Pnt2d secondEnd = firstCurve->Value(last);
  std::cout << "shift=" << changed << " done1=" << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " done2=" << fixer.LastFixStatus(ShapeExtend_DONE2) << " uv=" << firstStart.X()
            << ',' << firstEnd.X() << ',' << secondStart.X() << ',' << secondEnd.X() << '\n';
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
