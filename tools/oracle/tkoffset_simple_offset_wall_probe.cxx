// Native probe for BRepOffset_MakeSimpleOffset::BuildWallFace without changing OCCT sources.
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepOffset_MakeSimpleOffset.hxx>
#include <BRepTools_Modification.hxx>
#include <BRepTools_Modifier.hxx>
#include <BRepTools_TrsfModification.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_Curve.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Surface.hxx>
#include <NCollection_DataMap.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopLoc_Location.hxx>
#include <gp.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <iomanip>
#include <iostream>

template <class Tag, typename Tag::type Member> struct PrivateAccess
{
  friend typename Tag::type private_access(Tag) { return Member; }
};
struct BuildWallTag
{
  using type = TopoDS_Face (BRepOffset_MakeSimpleOffset::*)(const TopoDS_Edge&);
  friend type private_access(BuildWallTag);
};
template struct PrivateAccess<BuildWallTag, &BRepOffset_MakeSimpleOffset::BuildWallFace>;
struct BuilderTag
{
  using type = BRepTools_Modifier BRepOffset_MakeSimpleOffset::*;
  friend type private_access(BuilderTag);
};
template struct PrivateAccess<BuilderTag, &BRepOffset_MakeSimpleOffset::myBuilder>;
struct VertexEdgesTag
{
  using type = NCollection_DataMap<TopoDS_Vertex, TopoDS_Edge> BRepOffset_MakeSimpleOffset::*;
  friend type private_access(VertexEdgesTag);
};
template struct PrivateAccess<VertexEdgesTag, &BRepOffset_MakeSimpleOffset::myMapVE>;

void configure(BRepOffset_MakeSimpleOffset& maker, const TopoDS_Shape& shape, gp_Trsf transform)
{
  BRepTools_Modifier& builder = maker.*private_access(BuilderTag{});
  builder.Init(shape);
  occ::handle<BRepTools_Modification> modification = new BRepTools_TrsfModification(transform);
  builder.Perform(modification);
}

void print_seam_curves(TopoDS_Edge edge, const TopoDS_Face& face, int index);

void print_curve(TopoDS_Edge edge, const TopoDS_Face& face, int index)
{
  double curveFirst = 0.0, curveLast = 0.0, pcurveFirst = 0.0, pcurveLast = 0.0;
  TopLoc_Location location;
  const TopAbs_Orientation orientation = edge.Orientation();
  const occ::handle<Geom_Curve> curve = BRep_Tool::Curve(edge, location, curveFirst, curveLast);
  TopoDS_Edge forward = edge;
  forward.Orientation(TopAbs_FORWARD);
  const occ::handle<Geom2d_Curve> pcurveForward =
    BRep_Tool::CurveOnSurface(forward, face, pcurveFirst, pcurveLast);
  const gp_Pnt2d uvFirst = pcurveForward->Value(pcurveFirst), uvLast = pcurveForward->Value(pcurveLast);
  const gp_Pnt curveStart = curve->Value(curveFirst).Transformed(location.Transformation());
  const gp_Pnt curveEnd = curve->Value(curveLast).Transformed(location.Transformation());
  std::cout << "edge " << index << " ori " << orientation << " range3d " << curveFirst << ' '
            << curveLast << " range2d " << pcurveFirst << ' ' << pcurveLast << " curve "
            << (curve.IsNull() ? "null" : curve->DynamicType()->Name())
            << " pcurve " << (pcurveForward.IsNull() ? "null" : pcurveForward->DynamicType()->Name())
            << " xyz " << curveStart.X() << ' ' << curveStart.Y() << ' ' << curveStart.Z() << ' '
            << curveEnd.X() << ' ' << curveEnd.Y() << ' ' << curveEnd.Z() << " uv " << uvFirst.X()
            << ' ' << uvFirst.Y() << ' ' << uvLast.X() << ' ' << uvLast.Y() << " tol "
            << BRep_Tool::Tolerance(edge) << " flags " << BRep_Tool::SameRange(edge) << ' '
            << BRep_Tool::SameParameter(edge) << ' ' << BRep_Tool::IsClosed(edge, face) << '\n';
  if (BRep_Tool::IsClosed(edge, face)) print_seam_curves(edge, face, index);
}

void print_seam_curves(TopoDS_Edge edge, const TopoDS_Face& face, int index)
{
  double first = 0.0, last = 0.0;
  edge.Orientation(TopAbs_REVERSED);
  const occ::handle<Geom2d_Curve> curve = BRep_Tool::CurveOnSurface(edge, face, first, last);
  const gp_Pnt2d uvFirst = curve->Value(first), uvLast = curve->Value(last);
  std::cout << "seam " << index << " range " << first << ' ' << last << " uv " << uvFirst.X()
            << ' ' << uvFirst.Y() << ' ' << uvLast.X() << ' ' << uvLast.Y() << '\n';
}

void print_wall(BRepOffset_MakeSimpleOffset& maker, const TopoDS_Edge& edge, int index)
{
  const TopoDS_Face face = (maker.*private_access(BuildWallTag{}))(edge);
  NCollection_DataMap<TopoDS_Vertex, TopoDS_Edge>& map = maker.*private_access(VertexEdgesTag{});
  std::cout << "wall " << index << " null " << face.IsNull() << " map " << map.Extent();
  if (face.IsNull()) { std::cout << '\n'; return; }
  TopLoc_Location location;
  const occ::handle<Geom_Surface> surface = BRep_Tool::Surface(face, location);
  double uMin = 0.0, uMax = 0.0, vMin = 0.0, vMax = 0.0;
  surface->Bounds(uMin, uMax, vMin, vMax);
  const gp_Pnt point = surface->Value(0.25, 0.5).Transformed(location.Transformation());
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> edges;
  TopExp::MapShapes(face, TopAbs_EDGE, edges);
  std::cout << " valid " << BRepCheck_Analyzer(face).IsValid() << " orient " << face.Orientation()
            << " surface " << surface->DynamicType()->Name() << " loc " << location.IsIdentity()
            << " bounds " << uMin << ' ' << uMax << ' ' << vMin << ' ' << vMax << " tol "
            << BRep_Tool::Tolerance(face) << " point " << point.X() << ' '
            << point.Y() << ' ' << point.Z() << " edges " << edges.Extent() << '\n';
  for (int edgeIndex = 1; edgeIndex <= edges.Extent(); ++edgeIndex)
    print_curve(TopoDS::Edge(edges(edgeIndex)), face, edgeIndex);
}

TopoDS_Face plane_face()
{
  return BRepBuilderAPI_MakeFace(gp_Pln(gp::XOY()), 0.0, 2.0, 0.0, 3.0).Face();
}

TopoDS_Edge circle_edge(double z, double last)
{
  const gp_Ax2 axis(gp_Pnt(0.0, 0.0, z), gp::DZ());
  return BRepBuilderAPI_MakeEdge(gp_Circ(axis, 3.0), 0.0, last).Edge();
}

void print_plane_walls()
{
  const TopoDS_Face face = plane_face();
  BRepOffset_MakeSimpleOffset maker;
  gp_Trsf translation;
  translation.SetTranslation(gp_Vec(0.0, 0.0, 1.0));
  configure(maker, face, translation);
  TopExp_Explorer edges(face, TopAbs_EDGE);
  int index = 0;
  while (edges.More()) { print_wall(maker, TopoDS::Edge(edges.Current()), ++index); edges.Next(); }
}

void print_arc_wall(const char* name, double last, double offsetZ)
{
  const TopoDS_Edge edge = circle_edge(0.0, last);
  BRepOffset_MakeSimpleOffset maker;
  gp_Trsf translation;
  translation.SetTranslation(gp_Vec(0.0, 0.0, offsetZ));
  configure(maker, edge, translation);
  std::cout << "case " << name << '\n';
  print_wall(maker, edge, 1);
}

int main()
{
  std::cout << std::setprecision(17);
  std::cout << "case plane\n";
  print_plane_walls();
  print_arc_wall("coincident", 3.141592653589793, 0.0);
  print_arc_wall("half_arc", 3.141592653589793, 2.0);
  print_arc_wall("closed_circle", 6.283185307179586, 2.0);
}
