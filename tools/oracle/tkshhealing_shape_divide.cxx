// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871: whole-shape division flow.
#include <ShapeUpgrade_ShapeDivide.hxx>
#include <ShapeUpgrade_FaceDivideArea.hxx>
#include <ShapeUpgrade_WireDivide.hxx>
#include <ShapeExtend_BasicMsgRegistrator.hxx>
#include <ShapeExtend.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS_Wire.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Pln.hxx>
#include <gp_Trsf.hxx>
#include <iostream>
class Probe : public ShapeUpgrade_ShapeDivide {
public:
  using ShapeUpgrade_ShapeDivide::ShapeUpgrade_ShapeDivide;
  bool retained() const {return myShape.IsSame(original);}
  TopoDS_Shape original;
};
class FailingFace : public ShapeUpgrade_FaceDivide {
public: bool Perform(double) override { throw Standard_Failure("fixture face failure"); }
};
class CopyWire : public ShapeUpgrade_WireDivide {
public:
  void Perform() override {
    TopoDS_Wire copy=TopoDS::Wire(myWire.EmptyCopied()); BRep_Builder builder;
    for(TopoDS_Iterator it(myWire);it.More();it.Next()) builder.Add(copy,it.Value());
    myWire=copy; myStatus=ShapeExtend::EncodeStatus(ShapeExtend_DONE1)|ShapeExtend::EncodeStatus(ShapeExtend_FAIL1);
  }
};
class CountMessages : public ShapeExtend_BasicMsgRegistrator {
public: int count=0;
  void Send(const TopoDS_Shape&,const Message_Msg&,const Message_Gravity) override { ++count; }
};
static TopoDS_Shape fixture(int mode) {
  if(mode==12) { TopoDS_Edge edge; BRep_Builder().MakeEdge(edge); return edge; }
  if(mode==0) return TopoDS_Shape();
  if(mode==9) return BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(2,0,0)).Edge();
  if(mode==8) { TopoDS_Wire wire; BRep_Builder builder; builder.MakeWire(wire); builder.Add(wire,BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(2,0,0)).Edge()); return wire; }
  if(mode==4) return BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(2,0,0)).Edge();
  TopoDS_Shape face=BRepBuilderAPI_MakeFace(gp_Pln(),0.,2.,0.,2.).Face();
  if(mode!=3 && mode!=7 && mode!=10 && mode!=11) return face;
  TopoDS_Compound compound; BRep_Builder builder; builder.MakeCompound(compound);
  builder.Add(compound,face);
  gp_Trsf shift; shift.SetTranslation(gp_Vec(0.,0.,5.));
  face.Location(TopLoc_Location(shift)); builder.Add(compound,face);
  compound.Reverse(); return compound;
}
static int count(const TopoDS_Shape& shape, TopAbs_ShapeEnum kind) {
  int n=0; for(TopExp_Explorer it(shape,kind);it.More();it.Next()) ++n; return n;
}
static void inspect(int mode) {
  TopoDS_Shape source=fixture(mode); Probe divider(source); divider.original=source;
  auto originalContext=divider.GetContext();
  occ::handle<CountMessages> messages=new CountMessages; divider.SetMsgRegistrator(messages);
  if(mode==8 || mode==9 || mode==14) {
    occ::handle<ShapeUpgrade_FaceDivide> face=new ShapeUpgrade_FaceDivide;
    face->SetWireDivideTool(new CopyWire); divider.SetSplitFaceTool(face);
  }
  if(mode==2 || mode==3 || mode==7) {
    occ::handle<ShapeUpgrade_FaceDivideArea> area=new ShapeUpgrade_FaceDivideArea;
    area->MaxArea()=2.; divider.SetSplitFaceTool(area);
  }
  if(mode==5 || mode==11) divider.SetSplitFaceTool(new FailingFace);
  if(mode==6) divider.GetContext()->Remove(source);
  if(mode==7) divider.GetContext()->ModeConsiderLocation()=true;
  if(mode==13) { occ::handle<ShapeUpgrade_FaceDivide> face=new ShapeUpgrade_FaceDivide; face->SetWireDivideTool(nullptr); divider.SetSplitFaceTool(face); }
  bool changed=divider.Perform(mode!=6 && mode!=7);
  auto result=divider.Result();
  std::cout<<mode<<' '<<changed<<' '<<result.IsNull()<<' '<<result.IsSame(source)
    <<' '<<count(result,TopAbs_FACE)<<' '<<count(result,TopAbs_EDGE)
    <<' '<<(divider.GetContext()==originalContext)<<' '<<divider.retained();
  for(auto status:{ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3})
    std::cout<<' '<<divider.Status(status);
  int reversed=0; double z=0;
  for(TopExp_Explorer it(result,TopAbs_FACE);it.More();it.Next()) {
    reversed += it.Current().Orientation()==TopAbs_REVERSED;
    z += it.Current().Location().Transformation().TranslationPart().Z();
  }
  std::cout<<' '<<messages->count<<' '<<reversed<<' '<<z<<'\n';
}
int main() { for(int mode=0;mode<15;++mode) inspect(mode); }
