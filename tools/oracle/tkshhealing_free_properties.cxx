#include <ShapeAnalysis_FreeBoundsProperties.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Circ.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <iomanip>
using Properties=ShapeAnalysis_FreeBoundsProperties;
static TopoDS_Wire polygon(bool notch) {
  BRepBuilderAPI_MakePolygon maker;
  maker.Add(gp_Pnt(0,0,0));maker.Add(gp_Pnt(4,0,0));
  maker.Add(notch?gp_Pnt(0,0.1,0):gp_Pnt(4,2,0));
  if(!notch) {maker.Add(gp_Pnt(0,2,0));maker.Close();}
  return maker.Wire();
}
static void measure(const char* name,const TopoDS_Wire& wire) {
  Properties properties;
  occ::handle<ShapeAnalysis_FreeBoundData> record=new ShapeAnalysis_FreeBoundData(wire);
  const bool filled=properties.FillProperties(record,99.);
  const bool notched=properties.CheckNotches(record,99.);
  std::cout << name << ' ' << filled << ' ' << notched << ' ' << record->Area() << ' '
    << record->Perimeter() << ' ' << record->Ratio() << ' ' << record->Width() << ' ' << record->NbNotches() << '\n';
}
static void lifecycle() {
  Properties properties;
  std::cout << "default " << properties.IsLoaded() << ' ' << properties.Perform() << ' '
    << properties.NbFreeBounds() << ' ' << properties.CheckContours() << '\n';
  TopoDS_Face face=BRepBuilderAPI_MakeFace(gp_Pln(),0,4,0,2);
  properties.Init(face,false,false);properties.Perform();properties.Perform();
  std::cout << "repeated " << properties.IsLoaded() << ' ' << properties.NbClosedFreeBounds()
    << ' ' << properties.NbOpenFreeBounds() << ' ' << properties.ClosedFreeBound(1)->Area() << '\n';
  properties.Init(face,0.125,false,false);properties.Init(face,true,true);
  std::cout << "retained " << properties.Tolerance() << ' ' << properties.NbFreeBounds() << '\n';
}
static void notch_outputs() {
  Properties properties;TopoDS_Wire notch;double width=17.;
  bool accepted=properties.CheckNotches(polygon(true),1,notch,width,99.);
  int count=0;for(TopExp_Explorer it(notch,TopAbs_EDGE);it.More();it.Next()) ++count;
  std::cout << "notch " << accepted << ' ' << width << ' ' << count << '\n';
  accepted=properties.CheckNotches(polygon(true),0,notch,width,99.);
  count=0;for(TopExp_Explorer it(notch,TopAbs_EDGE);it.More();it.Next()) ++count;
  std::cout << "invalid " << accepted << ' ' << width << ' ' << count << '\n';
}
static void forecast() {
  BRep_Builder builder;TopoDS_Compound compound;builder.MakeCompound(compound);
  builder.Add(compound,BRepBuilderAPI_MakeFace(gp_Pln(),0,4,0,2));
  Properties properties(compound,0.001,false,false);
  const bool result=properties.Perform();
  std::cout << "forecast " << result << ' ' << properties.NbClosedFreeBounds() << ' '
    << properties.NbOpenFreeBounds() << ' ' << properties.ClosedFreeBound(1)->Area() << '\n';
}
static void tiny_notch() {
  BRepBuilderAPI_MakePolygon maker;
  maker.Add(gp_Pnt(0,0,0));maker.Add(gp_Pnt(4,0,0));
  maker.Add(gp_Pnt(4,0.001,0));maker.Add(gp_Pnt(0,0.1,0));
  Properties properties;TopoDS_Wire notch;double width=17.;
  properties.Init(TopoDS_Shape(),0.01,false,false);
  const bool result=properties.CheckNotches(maker.Wire(),1,notch,width);
  int count=0;for(TopExp_Explorer it(notch,TopAbs_EDGE);it.More();it.Next()) ++count;
  std::cout << "tiny_notch " << result << ' ' << width << ' ' << count << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  measure("rectangle",polygon(false));measure("open_notch",polygon(true));
  measure("circle",BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(),2.))));
  lifecycle();notch_outputs();forecast();tiny_notch();
}
