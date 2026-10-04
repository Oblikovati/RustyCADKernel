#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Curve.hxx>
#include <gp_Pnt2d.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>

#include <iomanip>
#include <iostream>

static void emitEndpoints(const occ::handle<Geom2d_Curve>& curve,
                          double first,
                          double last,
                          const char* label);

static void emitEdge(const TopoDS_Edge& edge, const TopoDS_Face& face, int edgeIndex)
{
  double first = 0.0, last = 0.0, reverseFirst = 0.0, reverseLast = 0.0;
  const auto forward = BRep_Tool::CurveOnSurface(edge, face, first, last);
  TopoDS_Edge reversed = edge;
  reversed.Reverse();
  const auto reverse = BRep_Tool::CurveOnSurface(reversed, face, reverseFirst, reverseLast);
  std::cout << std::setprecision(17) << "edge=" << edgeIndex << " closed="
            << BRep_Tool::IsClosed(edge, face) << " pair=" << (forward != reverse)
            << " range=" << first << ',' << last << " reverse_range=" << reverseFirst << ','
            << reverseLast;
  if (!forward.IsNull() && !reverse.IsNull())
  {
    emitEndpoints(forward, first, last, "forward");
    emitEndpoints(reverse, reverseFirst, reverseLast, "reverse");
  }
  std::cout << '\n';
}

static void emitEndpoints(const occ::handle<Geom2d_Curve>& curve,
                          double first,
                          double last,
                          const char* label)
{
  const gp_Pnt2d start = curve->Value(first), end = curve->Value(last);
  std::cout << ' ' << label << "=" << start.X() << ',' << start.Y() << ';' << end.X() << ','
            << end.Y();
}

int main()
{
  const TopoDS_Shape shape = BRepPrimAPI_MakeCylinder(2.0, 5.0).Shape();
  int faceIndex = 0;
  for (TopExp_Explorer faces(shape, TopAbs_FACE); faces.More(); faces.Next())
  {
    const TopoDS_Face face = TopoDS::Face(faces.Current());
    std::cout << "face=" << ++faceIndex << '\n';
    int edgeIndex = 0;
    for (TopExp_Explorer edges(face, TopAbs_EDGE); edges.More(); edges.Next())
      emitEdge(TopoDS::Edge(edges.Current()), face, ++edgeIndex);
  }
}
