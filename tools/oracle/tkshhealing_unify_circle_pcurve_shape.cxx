#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepTools.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_Circle.hxx>
#include <NCollection_Array1.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <NCollection_Array2.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Circ.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>
#include <cmath>

static occ::handle<Geom_BSplineSurface> makeSurface()
{
  NCollection_Array2<gp_Pnt> poles(1, 3, 1, 3);
  const double xs[] = {-1.0, 0.0, 1.0};
  const double squares[] = {1.0, -1.0, 1.0};
  const double radius = 0.5;
  for (int u = 1; u <= 3; ++u)
  {
    for (int v = 1; v <= 3; ++v)
    {
      poles.SetValue(u, v, gp_Pnt(xs[u - 1], xs[v - 1], squares[u - 1] + squares[v - 1] - radius * radius));
    }
  }
  NCollection_Array1<double> knots(1, 2);
  NCollection_Array1<int> mults(1, 2);
  knots.SetValue(1, -1.0);
  knots.SetValue(2, 1.0);
  mults.SetValue(1, 3);
  mults.SetValue(2, 3);
  return new Geom_BSplineSurface(poles, knots, knots, mults, mults, 2, 2, false, false);
}

static TopoDS_Face makeFace(const occ::handle<Geom_BSplineSurface>& surface)
{
  const gp_Pnt start(0.5, 0.0, 0.0);
  const gp_Pnt end(-0.5, 0.0, 0.0);
  const TopoDS_Vertex startVertex = BRepBuilderAPI_MakeVertex(start);
  const TopoDS_Vertex endVertex = BRepBuilderAPI_MakeVertex(end);
  const gp_Circ circle(gp_Ax2(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), 0.5);
  occ::handle<Geom_Circle> curve3d = new Geom_Circle(circle);
  const double pi = std::acos(-1.0);
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(curve3d, startVertex, endVertex, 0.0, pi);
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(curve3d, endVertex, startVertex, pi, 2.0 * pi);
  const gp_Circ2d pcurve(gp_Ax22d(gp_Pnt2d(0.0, 0.0), gp_Dir2d(1.0, 0.0)), 0.5);
  BRep_Builder builder;
  builder.UpdateEdge(first, new Geom2d_Circle(pcurve), surface, TopLoc_Location(), 1.0e-7);
  builder.UpdateEdge(second, new Geom2d_Circle(pcurve), surface, TopLoc_Location(), 1.0e-7);
  builder.Range(first, surface, TopLoc_Location(), 0.0, pi);
  builder.Range(second, surface, TopLoc_Location(), pi, 2.0 * pi);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(first);
  wireBuilder.Add(second);
  return BRepBuilderAPI_MakeFace(surface, wireBuilder.Wire(), true).Face();
}

int main()
{
  occ::handle<Geom_BSplineSurface> surface = makeSurface();
  TopoDS_Face face = makeFace(surface);
  std::cout << std::setprecision(17) << "input faces=1 edges=";
  int inputEdges = 0;
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next())
  {
    ++inputEdges;
  }
  std::cout << inputEdges << " valid=" << BRepCheck_Analyzer(face, true).IsValid() << '\n';
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next())
  {
    const TopoDS_Edge edge = TopoDS::Edge(it.Current());
    double first = 0.0;
    double last = 0.0;
    const auto pcurve = BRep_Tool::CurveOnSurface(edge, face, first, last);
    TopoDS_Edge reversed = edge;
    reversed.Reverse();
    double reverseFirst = 0.0;
    double reverseLast = 0.0;
    const auto reversePCurve = BRep_Tool::CurveOnSurface(reversed, face, reverseFirst, reverseLast);
    std::cout << "input_edge " << Geom2dAdaptor_Curve(pcurve).GetType() << ' ' << first << ' ' << last
              << " reverse_same=" << (pcurve == reversePCurve) << " reverse_type="
              << Geom2dAdaptor_Curve(reversePCurve).GetType();
    const gp_Circ2d c = Geom2dAdaptor_Curve(pcurve).Circle();
    std::cout << " center=" << c.Location().X() << ',' << c.Location().Y() << " radius=" << c.Radius()
              << " xdir=" << c.Position().XDirection().X() << ',' << c.Position().XDirection().Y()
              << " edgeori=" << edge.Orientation() << '\n';
  }
  ShapeUpgrade_UnifySameDomain unify(face, true, false, false);
  unify.Build();
  TopoDS_Shape result = unify.Shape();
  int edges = 0;
  for (TopExp_Explorer it(result, TopAbs_EDGE); it.More(); it.Next())
  {
    const TopoDS_Edge edge = TopoDS::Edge(it.Current());
    ++edges;
    double first = 0.0;
    double last = 0.0;
    const auto pcurve = BRep_Tool::CurveOnSurface(edge, face, first, last);
    std::cout << "edge " << Geom2dAdaptor_Curve(pcurve).GetType() << ' ' << first << ' ' << last;
    if (!pcurve.IsNull())
    {
      const gp_Pnt2d a = pcurve->Value(first);
      const gp_Pnt2d b = pcurve->Value(last);
      std::cout << ' ' << a.X() << ' ' << a.Y() << ' ' << b.X() << ' ' << b.Y();
    }
    std::cout << '\n';
  }
  std::cout << "result faces=";
  int faces = 0;
  for (TopExp_Explorer it(result, TopAbs_FACE); it.More(); it.Next())
  {
    ++faces;
  }
  std::cout << faces << " edges=" << edges << " valid=" << BRepCheck_Analyzer(result, true).IsValid() << '\n';
  return 0;
}
