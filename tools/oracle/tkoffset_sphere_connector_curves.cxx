// Records the analytic connector curves reused by BRepOffset_Offset::Init(Vertex).
#include <BRepGProp.hxx>
#include <BRepOffset_MakeOffset.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Curve.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GProp_GProps.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Vertex.hxx>
#include <iomanip>
#include <iostream>

int main() {
  const TopoDS_Shape input = BRepPrimAPI_MakeBox(1.0, 2.0, 3.0).Shape();
  BRepOffset_MakeOffset maker;
  maker.Initialize(input, 0.5, 1.0e-7, BRepOffset_Skin, false, false, GeomAbs_Arc, false, false);
  maker.MakeOffsetShape();
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> vertices;
  TopExp::MapShapes(input, TopAbs_VERTEX, vertices);
  const TopoDS_Vertex vertex = TopoDS::Vertex(vertices(1));
  const TopoDS_Face face = TopoDS::Face(maker.Generated(vertex).First());
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(face, properties);
  std::cout << std::setprecision(17) << "sphere_area " << properties.Mass() << " edges";
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) {
    const TopoDS_Edge edge = TopoDS::Edge(it.Current());
    TopLoc_Location location;
    double first, last, first2d, last2d;
    const Handle(Geom_Curve) curve = BRep_Tool::Curve(edge, location, first, last);
    const GeomAdaptor_Curve adaptor(curve, first, last);
    const Handle(Geom2d_Curve) pcurve = BRep_Tool::CurveOnSurface(edge, face, first2d, last2d);
    Handle(Geom2d_Curve) pcurve_basis = pcurve;
    const Handle(Geom2d_TrimmedCurve) trimmed = Handle(Geom2d_TrimmedCurve)::DownCast(pcurve);
    if (!trimmed.IsNull()) pcurve_basis = trimmed->BasisCurve();
    std::cout << " [" << adaptor.GetType() << "," << pcurve_basis->DynamicType()->Name();
    const Handle(Geom2d_BSplineCurve) spline = Handle(Geom2d_BSplineCurve)::DownCast(pcurve_basis);
    if (!spline.IsNull()) std::cout << ",degree=" << spline->Degree() << ",knots=" << spline->NbKnots();
    std::cout << "]";
  }
  std::cout << "\n";
}
