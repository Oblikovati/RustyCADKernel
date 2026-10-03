// Oracle at the pinned OCCT revision for ShapeFix_Wire::FixLacking.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Plane.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <iomanip>
#include <iostream>

int main()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  TopoDS_Vertex vertex;
  builder.MakeVertex(vertex, gp_Pnt(), 1.e-7);
  TopoDS_Vertex first = vertex, last = vertex;
  first.Orientation(TopAbs_FORWARD);
  last.Orientation(TopAbs_REVERSED);
  builder.Add(edge, first);
  builder.Add(edge, last);
  builder.UpdateEdge(edge, new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1., 0.))), face, 1.e-7);
  builder.Range(edge, face, 0., 0.001);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(edge);
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire(data, face, 1.e-7);
  gp_Pnt2d p1, p2;
  const bool gap = analyzer->CheckLacking(1, 0., p1, p2);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  fixer.SetMaxTolerance(0.01);
  const bool changed = fixer.FixLacking(1, false);
  std::cout << std::setprecision(17) << "gap=" << gap << " d3=" << analyzer->MaxDistance3d()
            << " max=" << fixer.MaxTolerance() << " single=" << changed
            << " done1=" << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " done2=" << fixer.LastFixStatus(ShapeExtend_DONE2)
            << " fail1=" << fixer.LastFixStatus(ShapeExtend_FAIL1)
            << " fail2=" << fixer.LastFixStatus(ShapeExtend_FAIL2)
            << " tolerance=" << BRep_Tool::Tolerance(vertex) << '\n';
  fixer.FixLacking(false);
  std::cout << "aggregate=" << fixer.StatusLacking(ShapeExtend_DONE)
            << " count=" << data->NbEdges() << '\n';

  TopoDS_Edge edge2;
  builder.MakeEdge(edge2);
  TopoDS_Vertex vertex2;
  builder.MakeVertex(vertex2, gp_Pnt(), 1.e-7);
  TopoDS_Vertex first2 = vertex2, last2 = vertex2;
  first2.Orientation(TopAbs_FORWARD);
  last2.Orientation(TopAbs_REVERSED);
  builder.Add(edge2, first2);
  builder.Add(edge2, last2);
  builder.UpdateEdge(edge2, new Geom_Line(gp_Ax1()), 1.e-7);
  builder.Range(edge2, 0., 1.e-8, true);
  builder.UpdateEdge(edge2, new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1., 0.))), face, 1.e-7);
  builder.Range(edge2, face, 0., 0.001);
  Handle(ShapeExtend_WireData) data2 = new ShapeExtend_WireData;
  data2->Add(edge2);
  Handle(ShapeAnalysis_Wire) analyzer2 = new ShapeAnalysis_Wire(data2, face, 1.e-7);
  ShapeFix_Wire fixer2;
  fixer2.Init(analyzer2);
  fixer2.ModifyTopologyMode() = true;
  fixer2.SetMaxTolerance(1.e-4);
  const bool changed2 = fixer2.FixLacking(1, false);
  std::cout << "topology=" << changed2 << " count=" << data2->NbEdges()
            << " done2=" << fixer2.LastFixStatus(ShapeExtend_DONE2)
            << " done4=" << fixer2.LastFixStatus(ShapeExtend_DONE4)
            << " fail3=" << fixer2.LastFixStatus(ShapeExtend_FAIL3) << '\n';

  TopoDS_Edge edge3;
  builder.MakeEdge(edge3);
  TopoDS_Vertex vertex3;
  builder.MakeVertex(vertex3, gp_Pnt(), 1.e-7);
  TopoDS_Vertex first3 = vertex3, last3 = vertex3;
  first3.Orientation(TopAbs_FORWARD);
  last3.Orientation(TopAbs_REVERSED);
  builder.Add(edge3, first3);
  builder.Add(edge3, last3);
  builder.UpdateEdge(edge3, new Geom_Line(gp_Ax1()), 1.e-7);
  builder.Range(edge3, 0., 0.001, true);
  builder.UpdateEdge(edge3, new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1., 0.))), face, 1.e-7);
  builder.Range(edge3, face, 0., 0.001);
  Handle(ShapeExtend_WireData) data3 = new ShapeExtend_WireData;
  data3->Add(edge3);
  Handle(ShapeAnalysis_Wire) analyzer3 = new ShapeAnalysis_Wire(data3, face, 1.e-7);
  ShapeFix_Wire fixer3;
  fixer3.Init(analyzer3);
  fixer3.ModifyTopologyMode() = true;
  fixer3.SetMaxTolerance(1.e-4);
  const bool changed3 = fixer3.FixLacking(1, false);
  std::cout << "closed=" << changed3 << " count=" << data3->NbEdges()
            << " done1=" << fixer3.LastFixStatus(ShapeExtend_DONE1)
            << " done2=" << fixer3.LastFixStatus(ShapeExtend_DONE2)
            << " done4=" << fixer3.LastFixStatus(ShapeExtend_DONE4)
            << " fail1=" << fixer3.LastFixStatus(ShapeExtend_FAIL1) << '\n';

  TopoDS_Edge edge4;
  builder.MakeEdge(edge4);
  TopoDS_Vertex vertex4;
  builder.MakeVertex(vertex4, gp_Pnt(0.0005, 0., 0.), 1.e-7);
  TopoDS_Vertex first4 = vertex4, last4 = vertex4;
  first4.Orientation(TopAbs_FORWARD);
  last4.Orientation(TopAbs_REVERSED);
  builder.Add(edge4, first4);
  builder.Add(edge4, last4);
  builder.UpdateEdge(edge4, new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1., 0.))), face, 1.e-7);
  builder.Range(edge4, face, 0., 0.001);
  Handle(ShapeExtend_WireData) data4 = new ShapeExtend_WireData;
  data4->Add(edge4);
  Handle(ShapeAnalysis_Wire) analyzer4 = new ShapeAnalysis_Wire(data4, face, 1.e-7);
  ShapeFix_Wire fixer4;
  fixer4.Init(analyzer4);
  fixer4.SetMaxTolerance(1.e-4);
  const bool changed4 = fixer4.FixLacking(1, false);
  std::cout << "degenerated=" << changed4 << " count=" << data4->NbEdges()
            << " done1=" << fixer4.LastFixStatus(ShapeExtend_DONE1)
            << " done2=" << fixer4.LastFixStatus(ShapeExtend_DONE2)
            << " done3=" << fixer4.LastFixStatus(ShapeExtend_DONE3)
            << " fail2=" << fixer4.LastFixStatus(ShapeExtend_FAIL2) << '\n';

  TopoDS_Edge edge5;
  builder.MakeEdge(edge5);
  TopoDS_Vertex vertex5;
  builder.MakeVertex(vertex5, gp_Pnt(), 1.e-7);
  TopoDS_Vertex first5 = vertex5, last5 = vertex5;
  first5.Orientation(TopAbs_FORWARD);
  last5.Orientation(TopAbs_REVERSED);
  builder.Add(edge5, first5);
  builder.Add(edge5, last5);
  builder.UpdateEdge(edge5, new Geom_Line(gp_Ax1()), 1.e-7);
  builder.Range(edge5, 0., 0.001, true);
  builder.UpdateEdge(edge5, new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1., 0.))), face, 1.e-7);
  builder.Range(edge5, face, 0., 0.001);
  Handle(ShapeExtend_WireData) data5 = new ShapeExtend_WireData;
  data5->Add(edge5);
  Handle(ShapeAnalysis_Wire) analyzer5 = new ShapeAnalysis_Wire(data5, face, 1.e-7);
  ShapeFix_Wire fixer5;
  fixer5.Init(analyzer5);
  fixer5.SetMaxTolerance(0.01);
  const bool changed5 = fixer5.FixLacking(1, false);
  std::cout << "bent=" << changed5 << " count=" << data5->NbEdges()
            << " done1=" << fixer5.LastFixStatus(ShapeExtend_DONE1)
            << " done2=" << fixer5.LastFixStatus(ShapeExtend_DONE2)
            << " done5=" << fixer5.LastFixStatus(ShapeExtend_DONE5)
            << " fail2=" << fixer5.LastFixStatus(ShapeExtend_FAIL2)
            << " tolerance=" << BRep_Tool::Tolerance(vertex5) << '\n';

  TopoDS_Edge edge6;
  builder.MakeEdge(edge6);
  builder.UpdateEdge(edge6, new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1., 0.))), face, 1.e-7);
  builder.Range(edge6, face, 0., 0.001);
  Handle(ShapeExtend_WireData) data6 = new ShapeExtend_WireData;
  data6->Add(edge6);
  Handle(ShapeAnalysis_Wire) analyzer6 = new ShapeAnalysis_Wire(data6, face, 1.e-7);
  ShapeFix_Wire fixer6;
  fixer6.Init(analyzer6);
  const bool changed6 = fixer6.FixLacking(1, false);
  std::cout << "failure=" << changed6
            << " fail1=" << fixer6.LastFixStatus(ShapeExtend_FAIL1)
            << " fail2=" << fixer6.LastFixStatus(ShapeExtend_FAIL2) << '\n';

  TopoDS_Edge edge7;
  builder.MakeEdge(edge7);
  TopoDS_Vertex vertex7;
  builder.MakeVertex(vertex7, gp_Pnt(), 1.e-7);
  TopoDS_Vertex first7 = vertex7, last7 = vertex7;
  first7.Orientation(TopAbs_FORWARD);
  last7.Orientation(TopAbs_REVERSED);
  builder.Add(edge7, first7);
  builder.Add(edge7, last7);
  builder.UpdateEdge(edge7, new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1., 0.))), face, 1.e-7);
  builder.Range(edge7, face, 0., 1.);
  Handle(ShapeExtend_WireData) data7 = new ShapeExtend_WireData;
  data7->Add(edge7);
  Handle(ShapeAnalysis_Wire) analyzer7 = new ShapeAnalysis_Wire(data7, face, 1.e-7);
  ShapeFix_Wire fixer7;
  fixer7.Init(analyzer7);
  const bool changed7 = fixer7.FixLacking(1, false);
  std::cout << "maximum_tolerance=" << changed7
            << " fail1=" << fixer7.LastFixStatus(ShapeExtend_FAIL1)
            << " fail2=" << fixer7.LastFixStatus(ShapeExtend_FAIL2) << '\n';
}
