// Native oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Line.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
#include <TopoDS_Edge.hxx>
#include <TopExp.hxx>
#include <iomanip>
#include <iostream>
static TopoDS_Edge curveEdge(const occ::handle<Geom_Curve>& curve,double start,double end,bool geometry=true) {
  BRep_Builder builder; TopoDS_Edge result; builder.MakeEdge(result);
  TopoDS_Vertex first,last;
  builder.MakeVertex(first,curve->Value(start),1.e-7); builder.MakeVertex(last,curve->Value(end),1.e-7);
  builder.Add(result,first.Oriented(TopAbs_FORWARD)); builder.Add(result,last.Oriented(TopAbs_REVERSED));
  if(geometry) { builder.UpdateEdge(result,curve,1.e-7); builder.Range(result,start,end,true); }
  return result;
}
static TopoDS_Edge edge(double origin,double dx,double dy,double length,bool geometry=true) {
  return curveEdge(new Geom_Line(gp_Pnt(origin,0.,0.),gp_Dir(dx,dy,0.)),0.,length,geometry);
}
static void curvedPair(TopoDS_Edge& first,TopoDS_Edge& second,bool shorter) {
  NCollection_Array1<gp_Pnt> poles(1,3);
  poles.SetValue(1,gp_Pnt(0.,0.,0.)); poles.SetValue(2,gp_Pnt(1.,2.,0.)); poles.SetValue(3,gp_Pnt(2.,0.,0.));
  occ::handle<Geom_Curve> curve=new Geom_BezierCurve(poles);
  first=curveEdge(curve,0.,1.); second=curveEdge(curve,shorter ? 0.5 : 0.,1.); second.Reverse();
}
static void inspect(const char* name,int mode) {
  TopoDS_Face face; BRep_Builder().MakeFace(face,new Geom_Plane(gp_Ax3()),1.e-7);
  TopoDS_Edge first=edge(0.,1.,0.,mode==9 ? 0. : 2.,mode!=7);
  TopoDS_Edge second=edge(mode==4 ? 3. : mode==9 ? 0. : 2.,-1.,mode==5 || mode==6 ? 0.2 : 0.,mode==1 || mode==3 ? 1. : mode==2 ? 3. : 2.);
  if(mode==3) { first=edge(2.,-1.,0.,2.); first.Reverse(); second=edge(1.,1.,0.,1.); second.Reverse(); }
  if(mode==8) BRep_Builder().Degenerated(first,true);
  if(mode>=11) curvedPair(first,second,mode==12);
  TopoDS_Edge sentinel=edge(30.,1.,0.,5.);
  TopoDS_Edge parts[]={sentinel,sentinel,sentinel,sentinel};
  occ::handle<ShapeExtend_WireData> wire=new ShapeExtend_WireData;
  if(mode!=10) { wire->Add(first); wire->Add(second); }
  ShapeAnalysis_Wire analyzer(wire,face,1.e-7);
  analyzer.CheckSmall(1,100.);
  bool result=analyzer.CheckTail(first,second,mode==6 ? -1. : 0.1,0.01,1.e-3,parts[0],parts[1],parts[2],parts[3]);
  std::cout << name << ' ' << result;
  for(auto& part : parts) {
    double start,last; BRep_Tool::Range(part,start,last);
    std::cout << ' ' << part.IsSame(first) << ' ' << part.IsSame(second) << ' ' << part.IsSame(sentinel)
              << ' ' << int(part.Orientation()) << ' ' << start << ' ' << last
              << ' ' << BRep_Tool::SameParameter(part) << ' ' << BRep_Tool::SameRange(part);
    for(auto vertex : {TopExp::FirstVertex(part),TopExp::LastVertex(part)}) {
      gp_Pnt point=BRep_Tool::Pnt(vertex);
      std::cout << ' ' << point.X() << ' ' << point.Y() << ' ' << point.Z() << ' ' << BRep_Tool::Tolerance(vertex);
    }
  }
  for(auto flag : {ShapeExtend_OK,ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_FAIL1,ShapeExtend_FAIL2})
    std::cout << ' ' << analyzer.LastCheckStatus(flag);
  std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  const char* names[]={"equal","second_short","first_short","reversed","gap","angle","angle_disabled","missing_curve","degenerated","zero_length","unloaded","bezier_equal","bezier_short"};
  for(int mode=0;mode<13;++mode) inspect(names[mode],mode);
}
