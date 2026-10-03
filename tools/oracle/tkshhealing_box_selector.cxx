#include <ShapeAnalysis_BoxBndTree.hxx>
#include <ShapeAnalysis.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <iostream>
using Selector=ShapeAnalysis_BoxBndTreeSelector;
static TopoDS_Wire wire(double offset) {
  return BRepBuilderAPI_MakeWire(BRepBuilderAPI_MakeEdge(gp_Pnt(offset,0,0),gp_Pnt(offset+1,0,0)));
}
static void observe(const char* name,Selector& selector,bool accepted) {
  std::cout << name << ' ' << accepted << ' ' << selector.GetNb() << ' ' << selector.Stop();
  for(auto flag:{ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_DONE4,ShapeExtend_FAIL2,ShapeExtend_OK})
    std::cout << ' ' << selector.LastCheckStatus(flag);
  std::cout << '\n';
}
static void orientations(bool shared) {
  occ::handle<NCollection_HArray1<TopoDS_Shape>> array=new NCollection_HArray1<TopoDS_Shape>(1,1);array->SetValue(1,wire(0));
  TopoDS_Vertex first,last;ShapeAnalysis::FindBounds(array->Value(1),first,last);
  TopoDS_Vertex far,unused;ShapeAnalysis::FindBounds(wire(10),far,unused);
  for(int mode=0;mode<4;++mode) {
    Selector selector(array,shared);
    TopoDS_Vertex a=mode<2?far:(mode==2?last:first);
    TopoDS_Vertex b=mode<2?(mode==0?first:last):far;
    selector.DefineVertexes(a,b);selector.DefinePnt(BRep_Tool::Pnt(a),BRep_Tool::Pnt(b));
    std::string name=(shared?"shared":"distance")+std::to_string(mode+1);
    observe(name.c_str(),selector,selector.Accept(1));
  }
}
static void retained_candidates() {
  occ::handle<NCollection_HArray1<TopoDS_Shape>> array=new NCollection_HArray1<TopoDS_Shape>(1,2);
  array->SetValue(1,wire(0));array->SetValue(2,wire(2));
  Selector selector(array,false);selector.SetTolerance(10);
  selector.DefinePnt(gp_Pnt(2,0,0),gp_Pnt(-0.5,0,0));
  observe("first",selector,selector.Accept(1));
  selector.SetStop();observe("head_closer",selector,selector.Accept(2));
  selector.LoadList(2);observe("visited",selector,selector.Accept(2));
  Bnd_Box box;box.SetWhole();selector.DefineBoxes(box,box);
  observe("boxes_reset",selector,selector.Accept(1));
  selector.SetTolerance(10);observe("tolerance_reset",selector,selector.Accept(1));
}
static void shared_failure() {
  occ::handle<NCollection_HArray1<TopoDS_Shape>> array=new NCollection_HArray1<TopoDS_Shape>(1,1);
  array->SetValue(1,wire(0));Selector selector(array,true);
  TopoDS_Vertex first,last;ShapeAnalysis::FindBounds(wire(10),first,last);
  selector.DefineVertexes(first,last);selector.SetNb(9);
  observe("shared_failure",selector,selector.Accept(1));
}
int main() {orientations(true);orientations(false);retained_candidates();shared_failure();}
