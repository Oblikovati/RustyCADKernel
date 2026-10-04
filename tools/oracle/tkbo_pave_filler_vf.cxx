#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_FaceInfo.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <iomanip>
class Probe : public BOPAlgo_PaveFiller {
public:
 void InitOnly(){Init(Message_ProgressRange());}
 void VF(){PerformVF(Message_ProgressRange());}
 void VV(){PerformVV(Message_ProgressRange());}
 void VE(){PerformVE(Message_ProgressRange());}
 void EE(){PerformEE(Message_ProgressRange());}
 BOPDS_DS& DS(){return *myDS;}
};
void output(Probe& filler,int mode,int pass)
{
 auto& ds=filler.DS();
 std::cout<<"case "<<mode<<' '<<pass<<' '<<ds.NbShapes()<<' '<<ds.InterfVF().Length()<<'\n';
 for(const auto& record:ds.InterfVF()) {
  double u,v;record.UV(u,v);
  std::cout<<"vf "<<record.Index1()<<' '<<record.Index2()<<' '<<record.IndexNew()<<' '<<u<<' '<<v<<'\n';
 }
 for(int i=0;i<ds.NbShapes();++i) {
  if(ds.ShapeInfo(i).ShapeType()==TopAbs_VERTEX)
   std::cout<<"v "<<i<<' '<<BRep_Tool::Tolerance(TopoDS::Vertex(ds.Shape(i)))<<' '<<ds.GetSameDomainIndex(i)<<'\n';
  if(ds.ShapeInfo(i).ShapeType()!=TopAbs_FACE)continue;
  const auto& info=ds.FaceInfo(i);
  std::cout<<"f "<<i<<' '<<ds.HasFaceInfo(i)<<' '<<info.VerticesIn().Extent();
  for(int index:info.VerticesIn())std::cout<<' '<<index;
  std::cout<<'\n';
 }
}
void run(int mode)
{
 Probe filler;
 if(mode==6) {
  filler.AddArgument(BRepBuilderAPI_MakeEdge(gp_Pnt(-.5,0,0),gp_Pnt(.5,0,0)).Edge());
  filler.AddArgument(BRepBuilderAPI_MakeEdge(gp_Pnt(0,-.5,0),gp_Pnt(0,.5,0)).Edge());
 } else {
  TopoDS_Vertex vertex; BRep_Builder builder;
  builder.MakeVertex(vertex,gp_Pnt(mode==3?2.:(mode==4||mode==8)?1.:.25,.5,mode==1||mode==2?5e-7:0.),1e-7);
  filler.AddArgument(vertex);
  if(mode==7) { TopoDS_Vertex other; builder.MakeVertex(other,gp_Pnt(.25,.5,1e-7),1e-7); filler.AddArgument(other); }
 }
 filler.AddArgument(BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0),gp_Dir(0,0,1)),-1.,1.,-1.,1.).Face());
 filler.SetFuzzyValue(1e-6);filler.SetNonDestructive(mode==2);
 if(mode==5)filler.SetGlue(BOPAlgo_GlueFull);
 filler.InitOnly();if(mode==6)filler.EE();if(mode==7)filler.VV();if(mode==8)filler.VE();
 filler.VF();output(filler,mode,0);filler.VF();output(filler,mode,1);
}
int main(){std::cout<<std::fixed<<std::setprecision(9);for(int mode=0;mode<9;++mode)run(mode);}
