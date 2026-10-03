#include <ShapeFix.hxx>
#include <Geom_Plane.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <ShapeExtend_BasicMsgRegistrator.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <TopExp_Explorer.hxx>
#include <Message_ProgressIndicator.hxx>
#include <iostream>
#include <iomanip>
class CancelledProgress : public Message_ProgressIndicator {
  bool UserBreak() override { return true; }
  void Show(const Message_ProgressScope&, const bool) override {}
};
class WarningCounter : public ShapeExtend_BasicMsgRegistrator {
public:
  int count = 0;
  void Send(const TopoDS_Shape&, const Message_Msg&, const Message_Gravity gravity) override {
    if(gravity == Message_Warning) ++count;
  }
};
static void observe(const char* name, const TopoDS_Shape& shape, bool enforce, bool cancel) {
  occ::handle<CancelledProgress> progress = new CancelledProgress;
  Message_ProgressRange range = cancel ? progress->Start() : Message_ProgressRange();
  occ::handle<WarningCounter> messages = new WarningCounter;
  bool status = ShapeFix::SameParameter(shape, enforce, 0.125, range, messages);
  int same = 0, ranges = 0; double tolerances = 0.;
  for (TopExp_Explorer it(shape, TopAbs_EDGE); it.More(); it.Next()) {
    const TopoDS_Edge edge = TopoDS::Edge(it.Current());
    same += BRep_Tool::SameParameter(edge); ranges += BRep_Tool::SameRange(edge);
    tolerances += BRep_Tool::Tolerance(edge);
  }
  std::cout << name << ' ' << status << ' ' << same << ' ' << ranges << ' '
            << std::setprecision(17) << tolerances << ' ' << messages->count << '\n';
}
static TopoDS_Shape displaced_edge_face() {
  BRep_Builder builder;
  TopoDS_Face face; builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1e-7);
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,1),gp_Pnt(1,0,1));
  TopoDS_Wire wire; builder.MakeWire(wire); builder.Add(wire,edge); builder.Add(face,wire);
  return face;
}
int main() {
  BRep_Builder builder; TopoDS_Compound empty; builder.MakeCompound(empty);
  observe("empty",empty,false,false); observe("empty_cancel",empty,false,true);
  TopoDS_Edge line = BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(1,0,0));
  observe("line",line,false,false); observe("line_enforce",line,true,false);
  observe("line_cancel",line,true,true);
  observe("displaced",displaced_edge_face(),false,false);
  observe("box",BRepPrimAPI_MakeBox(2,3,4).Shape(),true,false);
  observe("cylinder",BRepPrimAPI_MakeCylinder(2,5).Shape(),true,false);
  TopoDS_Edge noCurve; builder.MakeEdge(noCurve); observe("no_curve",noCurve,true,false);
}
