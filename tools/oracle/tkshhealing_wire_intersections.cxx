// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Ax3.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax1.hxx>
#include <Standard_Failure.hxx>
#include <Standard_NullObject.hxx>
#include <iomanip>
#include <iostream>
static TopoDS_Edge edgeOn(const TopoDS_Face& face, gp_Pnt2d first, gp_Pnt2d last, int mode, bool second) {
  BRep_Builder builder;
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  double z=(mode==5 && second) ? 2. : 0.;
  if (mode!=9) {
    TopoDS_Vertex a,b;
    builder.MakeVertex(a,gp_Pnt(first.X(),first.Y(),z),mode==6 ? 100. : 1.e-7);
    builder.MakeVertex(b,gp_Pnt(last.X(),last.Y(),z),mode==6 ? 100. : 1.e-7);
    builder.Add(edge,a.Oriented(TopAbs_FORWARD)); builder.Add(edge,b.Oriented(TopAbs_REVERSED));
  }
  gp_Dir2d direction(gp_Vec2d(first,last));
  if (mode!=7 || !second) {
    builder.UpdateEdge(edge,new Geom2d_Line(gp_Ax2d(first,direction)),face,1.e-7);
    builder.Range(edge,face,0.,mode==8 && second ? 0. : first.Distance(last));
  }
  if (mode==5 && second) {
    builder.UpdateEdge(edge,new Geom_Line(gp_Ax1(gp_Pnt(first.X(),first.Y(),z),gp_Dir(direction.X(),direction.Y(),0.))),1.e-7);
    builder.Range(edge,0.,first.Distance(last),true);
  }
  if (mode==4 && second) edge.Reverse();
  return edge;
}
static void inspect(const char* name, int mode) {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face,new Geom_Plane(gp_Ax3()),1.e-7);
  Handle(ShapeExtend_WireData) wire=new ShapeExtend_WireData;
  gp_Pnt2d a(0.,-1.),b(0.,1.);
  if(mode==1) { a=gp_Pnt2d(1.,0.); b=gp_Pnt2d(1.,2.); }
  if(mode==2) { a=gp_Pnt2d(0.,0.); b=gp_Pnt2d(0.,2.); }
  if(mode==3) { a=gp_Pnt2d(-0.5,0.); b=gp_Pnt2d(0.5,0.); }
  if(mode!=10) { wire->Add(edgeOn(face,gp_Pnt2d(-1.,0.),gp_Pnt2d(1.,0.),mode,false)); wire->Add(edgeOn(face,a,b,mode,true)); }
  ShapeAnalysis_Wire analysis(wire,face,1.e-7);
  NCollection_Sequence<IntRes2d_IntersectionPoint> uv; uv.Append(IntRes2d_IntersectionPoint());
  NCollection_Sequence<gp_Pnt> xyz; xyz.Append(gp_Pnt(77.,88.,99.));
  NCollection_Sequence<double> errors; errors.Append(66.);
  bool found=false; int exception=0;
  try { found=analysis.CheckIntersectingEdges(1,0,uv,xyz,errors); }
  catch(const Standard_NullObject&) { exception=1; }
  catch(const Standard_Failure&) { exception=2; }
  std::cout << name << " " << exception << " " << found << " " << analysis.LastCheckStatus(ShapeExtend_DONE1)
            << " " << analysis.LastCheckStatus(ShapeExtend_FAIL3) << " " << uv.Length() << " " << xyz.Length() << " " << errors.Length();
  for(int i=1;i<=uv.Length();++i) std::cout << " " << uv(i).ParamOnFirst() << " " << uv(i).ParamOnSecond()
            << " " << xyz(i).X() << " " << xyz(i).Y() << " " << xyz(i).Z() << " " << errors(i);
  std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  const char* names[]={"crossing","endpoint","one_endpoint","overlap","reversed","spatial_error","tolerated","missing_pcurve","zero_range","missing_vertices","unloaded"};
  for(int mode=0;mode<11;++mode) inspect(names[mode],mode);
}
