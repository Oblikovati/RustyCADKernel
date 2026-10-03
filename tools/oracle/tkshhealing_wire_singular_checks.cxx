// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 native branch regressions.
#include <BRep_Builder.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_Circle.hxx>
#include <Geom2d_Line.hxx>
#include <gp_Ax2d.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax3.hxx>
#include <gp_Ax2.hxx>
#include <iostream>

static TopoDS_Edge edgeAt(const gp_Pnt& first, const gp_Pnt& last)
{
  BRep_Builder builder;
  TopoDS_Edge edge;
  TopoDS_Vertex a, b;
  builder.MakeEdge(edge);
  builder.MakeVertex(a, first, 1.e-7);
  builder.MakeVertex(b, last, 1.e-7);
  a.Orientation(TopAbs_FORWARD);
  b.Orientation(TopAbs_REVERSED);
  builder.Add(edge, a);
  builder.Add(edge, b);
  return edge;
}

static void inspect(const char* name, const Handle(ShapeExtend_WireData)& wire)
{
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_ConicalSurface(gp_Ax3(), .5, 0.), 1.e-7);
  if (wire->NbEdges() == 2) {
    for (int index = 1; index <= 2; ++index) {
      builder.UpdateEdge(wire->Edge(index), new Geom2d_Line(gp_Ax2d(gp_Pnt2d(index * 2., 0.), gp_Dir2d(1., 0.))), face, 1.e-7);
      builder.Range(wire->Edge(index), face, 0., 1.);
    }
  }
  ShapeAnalysis_Wire analyzer(wire, face, 1.e-7);
  gp_Pnt2d first, last;
  const bool found = analyzer.CheckDegenerated(wire->NbEdges(), first, last);
  std::cout << name << " " << found << " "
            << analyzer.LastCheckStatus(ShapeExtend_DONE1) << " "
            << analyzer.LastCheckStatus(ShapeExtend_DONE2) << " "
            << analyzer.LastCheckStatus(ShapeExtend_FAIL1) << " "
            << analyzer.LastCheckStatus(ShapeExtend_FAIL2) << '\n';
}

int main()
{
  Handle(ShapeExtend_WireData) single = new ShapeExtend_WireData;
  TopoDS_Edge apex = edgeAt(gp_Pnt(), gp_Pnt());
  single->Add(apex);
  inspect("apex_without_pcurve", single);
  BRep_Builder builder;
  builder.UpdateEdge(apex, new Geom_Circle(gp_Ax2(gp_Pnt(-1.,0.,0.), gp_Dir(0.,0.,1.)), 1.), 1.e-7);
  builder.Range(apex, 0., 6.283185307179586, true);
  inspect("closed_curve_midpoint_veto", single);
  Handle(ShapeExtend_WireData) disconnected = new ShapeExtend_WireData;
  disconnected->Add(edgeAt(gp_Pnt(2.,0.,0.), gp_Pnt(1.,0.,0.)));
  disconnected->Add(edgeAt(gp_Pnt(), gp_Pnt(1.,0.,0.)));
  inspect("disconnected_singular_vertex", disconnected);
}
