// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS.hxx>
#include <iomanip>
#include <iostream>
static TopoDS_Edge edge(const TopoDS_Face& face,double start,double direction,double length,int mode,bool first) {
  BRep_Builder builder; TopoDS_Edge result; builder.MakeEdge(result);
  double vertexX = mode==6 && !first ? 3. : 0.;
  if(mode!=5 || !first) {
    TopoDS_Vertex vertex; builder.MakeVertex(vertex,gp_Pnt(vertexX,0.,0.),1.e-7);
    builder.Add(result,vertex.Oriented(TopAbs_FORWARD)); builder.Add(result,vertex.Oriented(TopAbs_REVERSED));
  }
  if(mode!=7 || !first) {
    occ::handle<Geom2d_Curve> curve=new Geom2d_Line(gp_Pnt2d(start,0.),gp_Dir2d(direction,0.));
    if(mode==8 && !first) curve=new Geom2d_Line(gp_Pnt2d(start,0.),gp_Dir2d(0.,1.));
    if(mode==9 && !first) curve=new Geom2d_Line(gp_Pnt2d(start,1.),gp_Dir2d(direction,0.));
    if(mode>=10 && !first) {
      NCollection_Array1<gp_Pnt2d> poles(1,4);
      poles.SetValue(1,gp_Pnt2d(2.,0.)); poles.SetValue(2,gp_Pnt2d(mode==10 ? 2. : 1.5,0.));
      poles.SetValue(3,gp_Pnt2d(1.5,mode==10 ? 0. : 2.)); poles.SetValue(4,gp_Pnt2d(1.,0.));
      curve=new Geom2d_BezierCurve(poles);
    }
    builder.UpdateEdge(result,curve,face,1.e-7); builder.Range(result,face,0.,length);
  }
  if(mode==4 && first) builder.Degenerated(result,true);
  return result;
}
static void inspect(const char* name,int mode) {
  TopoDS_Face face; BRep_Builder().MakeFace(face,new Geom_CylindricalSurface(gp_Ax3(),1.),1.e-7);
  TopoDS_Edge first=edge(face,0.,1.,2.,mode,true);
  TopoDS_Edge last=edge(face,2.,-1.,mode==2 ? 3. : mode==1 ? 2. : 1.,mode,false);
  if(mode==3) { first=edge(face,2.,-1.,2.,mode,true); first.Reverse(); last=edge(face,1.,1.,1.,mode,false); last.Reverse(); }
  occ::handle<ShapeExtend_WireData> wire=new ShapeExtend_WireData;
  if(mode!=12) { wire->Add(first); wire->Add(last); }
  ShapeAnalysis_Wire analysis(wire,face,1.e-7);
  int shortIndex=71; double parameter=83.;
  bool changed=analysis.CheckNotchedEdges(0,shortIndex,parameter,1.e-5);
  std::cout << name << ' ' << changed << ' ' << shortIndex << ' ' << parameter;
  for(auto flag : {ShapeExtend_DONE,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3})
    std::cout << ' ' << analysis.LastCheckStatus(flag);
  std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  const char* names[]={"second_short","equal","first_short","reversed","degenerated","missing_vertex","disconnected","missing_pcurve","tangent","gap","zero_tangent","curved_mismatch","unloaded"};
  for(int mode=0;mode<13;++mode) inspect(names[mode],mode);
}
