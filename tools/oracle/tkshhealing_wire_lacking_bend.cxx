#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge makeEdge(BRep_Builder& builder,
                            const TopoDS_Face& face,
                            const TopoDS_Vertex& first,
                            const TopoDS_Vertex& last,
                            const gp_Pnt& curveOrigin,
                            const gp_Pnt2d& pcurveOrigin)
{
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  TopoDS_Vertex vFirst = first;
  TopoDS_Vertex vLast = last;
  vFirst.Orientation(TopAbs_FORWARD);
  vLast.Orientation(TopAbs_REVERSED);
  builder.Add(edge, vFirst);
  builder.Add(edge, vLast);
  builder.UpdateEdge(edge, new Geom_Line(gp_Ax1(curveOrigin, gp_Dir(1., 0., 0.))), 1.e-7);
  builder.Range(edge, 0., 1., true);
  builder.UpdateEdge(edge,
                     new Geom2d_Line(gp_Ax2d(pcurveOrigin, gp_Dir2d(1., 0.))),
                     face,
                     1.e-7);
  builder.Range(edge, face, 0., 1.);
  builder.UpdateEdge(edge, 0.01);
  return edge;
}

int main()
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1.e-7);
  TopoDS_Vertex left, middle, right;
  builder.MakeVertex(left, gp_Pnt(-1., 0., 0.), 1.e-7);
  builder.MakeVertex(middle, gp_Pnt(0., 0., 0.), 1.e-7);
  builder.MakeVertex(right, gp_Pnt(1., 0., 0.), 1.e-7);
  TopoDS_Edge first = makeEdge(builder, face, left, middle, gp_Pnt(-1., 0., 0.), gp_Pnt2d(-1., 0.));
  TopoDS_Edge second = makeEdge(builder, face, middle, right, gp_Pnt(0., 0., 0.), gp_Pnt2d(0.001, 0.));
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(first);
  data->Add(second);
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire(data, face, 1.e-7);
  gp_Pnt2d gapStartBefore, gapEndBefore;
  analyzer->CheckLacking(2, 0., gapStartBefore, gapEndBefore);
  const double distanceBefore = analyzer->MaxDistance3d();
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  fixer.SetMaxTolerance(0.01);
  const bool changed = fixer.FixLacking(2, false);
  gp_Pnt2d gapFirst, gapLast;
  analyzer->CheckLacking(2, 0., gapFirst, gapLast);
  double firstP = 0., firstQ = 0., secondP = 0., secondQ = 0.;
  Handle(Geom2d_Curve) firstCurve, secondCurve;
  ShapeAnalysis_Edge analysis;
  analysis.PCurve(first, face, firstCurve, firstP, firstQ, false);
  analysis.PCurve(second, face, secondCurve, secondP, secondQ, false);
  std::cout << std::setprecision(17) << "bending=" << changed
            << " d3=" << distanceBefore
            << " gap=" << gapFirst.Distance(gapLast)
            << " done1=" << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " done5=" << fixer.LastFixStatus(ShapeExtend_DONE5)
            << " fail2=" << fixer.LastFixStatus(ShapeExtend_FAIL2)
            << " edge1tol=" << BRep_Tool::Tolerance(first)
            << " edge2tol=" << BRep_Tool::Tolerance(second)
            << " middletol=" << BRep_Tool::Tolerance(middle)
            << " first=" << firstCurve->Value(firstQ).X()
            << " second=" << secondCurve->Value(secondP).X()
            << " q1=" << firstCurve->Value(firstP + 0.25 * (firstQ - firstP)).X()
            << " q2=" << firstCurve->Value(firstP + 0.5 * (firstQ - firstP)).X()
            << " q3=" << firstCurve->Value(firstP + 0.75 * (firstQ - firstP)).X()
            << " r1=" << secondCurve->Value(secondP + 0.25 * (secondQ - secondP)).X()
            << " r2=" << secondCurve->Value(secondP + 0.5 * (secondQ - secondP)).X()
            << " r3=" << secondCurve->Value(secondP + 0.75 * (secondQ - secondP)).X()
            << " middletol=" << BRep_Tool::Tolerance(middle) << '\n';
}
