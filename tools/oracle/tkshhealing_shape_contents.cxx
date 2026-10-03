// Repository-owned oracle against OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <ShapeAnalysis_ShapeContents.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_Line.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Cylinder.hxx>
#include <gp.hxx>
#include <iostream>
static void row(const char* name,const ShapeAnalysis_ShapeContents& c) {
  std::cout << name;
  std::cout << ' ' << c.NbSolids();
  std::cout << ' ' << c.NbShells();
  std::cout << ' ' << c.NbFaces();
  std::cout << ' ' << c.NbWires();
  std::cout << ' ' << c.NbEdges();
  std::cout << ' ' << c.NbVertices();
  std::cout << ' ' << c.NbSolidsWithVoids();
  std::cout << ' ' << c.NbBigSplines();
  std::cout << ' ' << c.NbC0Surfaces();
  std::cout << ' ' << c.NbC0Curves();
  std::cout << ' ' << c.NbOffsetSurf();
  std::cout << ' ' << c.NbIndirectSurf();
  std::cout << ' ' << c.NbOffsetCurves();
  std::cout << ' ' << c.NbTrimmedCurve2d();
  std::cout << ' ' << c.NbTrimmedCurve3d();
  std::cout << ' ' << c.NbBSplibeSurf();
  std::cout << ' ' << c.NbBezierSurf();
  std::cout << ' ' << c.NbTrimSurf();
  std::cout << ' ' << c.NbWireWitnSeam();
  std::cout << ' ' << c.NbWireWithSevSeams();
  std::cout << ' ' << c.NbFaceWithSevWires();
  std::cout << ' ' << c.NbNoPCurve();
  std::cout << ' ' << c.NbFreeFaces();
  std::cout << ' ' << c.NbFreeWires();
  std::cout << ' ' << c.NbFreeEdges();
  std::cout << ' ' << c.NbSharedSolids();
  std::cout << ' ' << c.NbSharedShells();
  std::cout << ' ' << c.NbSharedFaces();
  std::cout << ' ' << c.NbSharedWires();
  std::cout << ' ' << c.NbSharedFreeWires();
  std::cout << ' ' << c.NbSharedFreeEdges();
  std::cout << ' ' << c.NbSharedEdges();
  std::cout << ' ' << c.NbSharedVertices();
  std::cout << ' ' << c.BigSplineSec()->Length();
  std::cout << ' ' << c.IndirectSec()->Length();
  std::cout << ' ' << c.OffsetSurfaceSec()->Length();
  std::cout << ' ' << c.Trimmed3dSec()->Length();
  std::cout << ' ' << c.OffsetCurveSec()->Length();
  std::cout << ' ' << c.Trimmed2dSec()->Length();
  std::cout << '\n';
}
static void modes(ShapeAnalysis_ShapeContents& c) {
  c.ModifyBigSplineMode()=true; c.ModifyIndirectMode()=true;
  c.ModifyOffsetSurfaceMode()=true; c.ModifyTrimmed3dMode()=true;
  c.ModifyOffsetCurveMode()=true; c.ModifyTrimmed2dMode()=true;
}
static TopoDS_Face face(const occ::handle<Geom_Surface>& surface) {
  BRep_Builder builder; TopoDS_Face face;
  builder.MakeFace(face,surface,1e-7); return face;
}
static occ::handle<Geom_BSplineSurface> spline(int count) {
  NCollection_Array2<gp_Pnt> poles(1,2,1,count);
  NCollection_Array1<double> u(1,2),v(1,count);
  NCollection_Array1<int> um(1,2),vm(1,count);
  u(1)=0;u(2)=1;um(1)=um(2)=2;
  for(int j=1;j<=count;++j) {
    v(j)=j-1;vm(j)=(j==1||j==count)?2:1;
    poles(1,j)=gp_Pnt(0,j-1,0);poles(2,j)=gp_Pnt(1,j-1,0);
  }
  return new Geom_BSplineSurface(poles,u,v,um,vm,1,1);
}
static TopoDS_Face curve_face() {
  TopoDS_Face f=face(new Geom_Plane(gp::XOY()));
  TopoDS_Wire w;BRep_Builder b;b.MakeWire(w);
  occ::handle<Geom_Curve> line=new Geom_Line(gp::OX());
  occ::handle<Geom2d_Curve> line2=new Geom2d_Line(gp::OX2d());
  TopoDS_Edge e;b.MakeEdge(e,new Geom_TrimmedCurve(line,0,1),1e-7);
  b.UpdateEdge(e,new Geom2d_TrimmedCurve(line2,0,1),f,1e-7);b.Range(e,0,1);b.Add(w,e);
  b.MakeEdge(e,new Geom_OffsetCurve(line,1,gp::DZ()),1e-7);
  b.UpdateEdge(e,new Geom2d_OffsetCurve(line2,1),f,1e-7);b.Range(e,0,1);b.Add(w,e);
  b.Add(f,w);return f;
}
int main() {
  ShapeAnalysis_ShapeContents c;c.Clear();row("clear",c);
  const TopoDS_Shape box=BRepPrimAPI_MakeBox(1,2,3).Shape();
  c.Perform(box);row("box",c);
  BRep_Builder b;TopoDS_Compound compound;b.MakeCompound(compound);b.Add(compound,box);
  gp_Trsf shift;shift.SetTranslation(gp_Vec(10,0,0));b.Add(compound,box.Moved(TopLoc_Location(shift)));
  c.Perform(compound);row("repeated-located-box",c);
  modes(c);
  c.Perform(BRepBuilderAPI_MakeFace(gp_Cylinder(gp::XOY(),1),0,2*M_PI,0,2).Face());row("seam-cylinder",c);
  gp_Ax3 indirect(gp::XOY());indirect.YReverse();
  c.Perform(face(new Geom_Plane(indirect)));row("indirect",c);
  c.Perform(face(new Geom_OffsetSurface(new Geom_Plane(gp::XOY()),1)));row("offset-surface",c);
  c.Perform(face(new Geom_RectangularTrimmedSurface(spline(3),0.,1.,0.,2.)));row("trimmed-spline",c);
  c.Perform(face(spline(4096)));row("spline-threshold",c);
  c.Perform(face(spline(4097)));row("big-spline",c);
  NCollection_Array2<gp_Pnt> poles(1,2,1,2);
  poles(1,1)=gp_Pnt(0,0,0);poles(2,1)=gp_Pnt(1,0,0);
  poles(1,2)=gp_Pnt(0,1,0);poles(2,2)=gp_Pnt(1,1,0);
  c.Perform(face(new Geom_BezierSurface(poles)));row("bezier-surface",c);
  const TopoDS_Face curves=curve_face();c.Perform(curves);row("curve-kinds",c);
  auto saved=c.Trimmed3dSec();c.ClearFlags();row("clear-flags",c);
  ShapeAnalysis_ShapeContents copy=c;copy.Clear();row("shared-clear",c);
  c.Perform(curves);row("recording-disabled",c);
  TopoDS_Wire wire;b.MakeWire(wire);TopoDS_Edge edge;b.MakeEdge(edge);b.Add(wire,edge);
  c.Perform(wire);row("free-wire",c);
  TopoDS_Face missing=face(new Geom_Plane(gp::XOY()));b.Add(missing,wire);
  c.Perform(missing);row("missing-pcurve",c);
}
