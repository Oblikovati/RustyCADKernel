// Observe FastComputeAnalytic by member-pointer access without changing OCCT sources.
#include <algorithm>
#include <cmath>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Circ.hxx>
#include <gp_Elips.hxx>
#include <gp_Hypr.hxx>
#include <gp_Lin.hxx>
#include <gp_Parab.hxx>
#include <gp_Pln.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Sphere.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_Line.hxx>
#include <Geom_OffsetCurve.hxx>
#include <NCollection_Array1.hxx>
template<class Tag, typename Tag::type Member> struct BeanAnalyticAccess {
  friend typename Tag::type bean_analytic_member(Tag) { return Member; }
};
struct BeanFastAnalyticTag {
  using type = bool (IntTools_BeanFaceIntersector::*)();
  friend type bean_analytic_member(BeanFastAnalyticTag);
};
template struct BeanAnalyticAccess<BeanFastAnalyticTag, &IntTools_BeanFaceIntersector::FastComputeAnalytic>;

static gp_Ax3 bean_fast_axis(const std::vector<double>& v, int offset) {
  return gp_Ax3(gp_Pnt(v[offset], v[offset + 1], v[offset + 2]),
                gp_Dir(v[offset + 3], v[offset + 4], v[offset + 5]));
}

static TopoDS_Edge bean_fast_edge(const std::vector<double>& v) {
  const gp_Ax2 anAxis(gp_Pnt(v[2], v[3], v[4]), gp_Dir(v[5], v[6], v[7]));
  const int aKind = int(v[0]);
  if (aKind == 5 || aKind == 6) {
    NCollection_Array1<gp_Pnt> poles(1, 2);
    poles(1) = anAxis.Location();
    poles(2) = anAxis.Location().Translated(gp_Vec(anAxis.Direction()));
    if (aKind == 5) return BRepBuilderAPI_MakeEdge(new Geom_BezierCurve(poles), v[16], v[17]);
    NCollection_Array1<double> knots(1, 2);
    NCollection_Array1<int> mults(1, 2);
    knots(1) = 0.; knots(2) = 1.; mults(1) = 2; mults(2) = 2;
    return BRepBuilderAPI_MakeEdge(new Geom_BSplineCurve(poles, knots, mults, 1), v[16], v[17]);
  }
  if (aKind == 7) {
    Handle(Geom_Curve) basis = new Geom_Line(anAxis.Location(), anAxis.Direction());
    return BRepBuilderAPI_MakeEdge(new Geom_OffsetCurve(basis, 0.25, gp_Dir(0., 1., 0.)), v[16], v[17]);
  }
  if (aKind == 0) return BRepBuilderAPI_MakeEdge(gp_Lin(anAxis.Location(), anAxis.Direction()), v[16], v[17]);
  if (aKind == 1) return BRepBuilderAPI_MakeEdge(gp_Circ(anAxis, v[8]), v[16], v[17]);
  if (aKind == 2) return BRepBuilderAPI_MakeEdge(gp_Elips(anAxis, std::max(v[8], 2.), std::max(v[8], 2.) * 0.5), v[16], v[17]);
  if (aKind == 3) return BRepBuilderAPI_MakeEdge(gp_Hypr(anAxis, std::max(v[8], 2.), std::max(v[8], 2.) * 0.5), v[16], v[17]);
  return BRepBuilderAPI_MakeEdge(gp_Parab(anAxis, std::max(v[8], 1.)), v[16], v[17]);
}

static TopoDS_Face bean_fast_face(const std::vector<double>& v) {
  const gp_Ax3 anAxis = bean_fast_axis(v, 9);
  const int aKind = int(v[1]);
  if (aKind == 0) return BRepBuilderAPI_MakeFace(gp_Pln(anAxis), -10., 10., -10., 10.);
  if (aKind == 1) return BRepBuilderAPI_MakeFace(gp_Cylinder(anAxis, v[15]), 0., 2. * M_PI, -10., 10.);
  return BRepBuilderAPI_MakeFace(gp_Sphere(anAxis, v[15]), -M_PI / 2., M_PI / 2., 0., 2. * M_PI);
}

void bean_fast_analytic(const std::vector<double>& v) {
  const TopoDS_Edge anEdge = bean_fast_edge(v);
  const TopoDS_Face aFace = bean_fast_face(v);
  const BRepAdaptor_Curve aCurve(anEdge);
  const BRepAdaptor_Surface aSurface(aFace, false);
  IntTools_BeanFaceIntersector anIntersector(aCurve, aSurface, v[16], v[17], -10., 10., -10., 10., v[18], 0.);
  const bool isDone = (anIntersector.*bean_analytic_member(BeanFastAnalyticTag{}))();
  bo_int(isDone);
  bo_int(anIntersector.Result().Length());
  for (const auto& aRange : anIntersector.Result()) bo_range(aRange);
}
