// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; DS observations are not certificates.
#include <BOPAlgo_Builder.hxx>
#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <TopLoc_Location.hxx>
#include <gp_Trsf.hxx>
#include <iomanip>
#include <iostream>
class PartitionBuilder final : public BOPAlgo_Builder {
  void CheckData() override { myReport->Merge(myPaveFiller->GetReport()); }
};
static NCollection_List<TopoDS_Shape> Sources(int mode)
{
  const TopoDS_Shape first=BRepBuilderAPI_MakeEdge(gp_Pnt(-1,0,0),gp_Pnt(1,0,0)).Edge();
  NCollection_List<TopoDS_Shape> result; result.Append(first);
  if (mode==1) return result;
  if (mode==4) {
    gp_Trsf shift;shift.SetTranslation(gp_Vec(0,3,0));
    result.Append(first.Located(TopLoc_Location(shift)));return result;
  }
  if (mode==2) result.Append(BRepBuilderAPI_MakeEdge(gp_Pnt(-1,0,0),gp_Pnt(1,0,0)).Edge());
  else result.Append(BRepBuilderAPI_MakeEdge(gp_Pnt(0,-1,0),gp_Pnt(0,1,0)).Edge());
  if (mode==3) result.Append(first.Reversed());
  return result;
}
static void EmitBlock(BOPDS_DS& ds,const occ::handle<BOPDS_PaveBlock>& own)
{
  const auto real=ds.RealPaveBlock(own);
  double a,b,c,d; int v1,v2,w1,w2;
  own->Range(a,b);real->Range(c,d);own->Indices(v1,v2);real->Indices(w1,w2);
  std::cout<<"block "<<own->OriginalEdge()<<' '<<own->Edge()<<' '<<real->OriginalEdge()<<' '<<real->Edge()
           <<' '<<a<<' '<<b<<' '<<c<<' '<<d<<' '<<v1<<' '<<v2<<' '<<w1<<' '<<w2<<'\n';
}
static void EmitSource(int index,BOPDS_DS& ds,const PartitionBuilder& builder)
{
  const auto& shape=ds.Shape(index);double first,last;
  BRep_Tool::Range(TopoDS::Edge(shape),first,last);
  const auto* images=builder.Images().Seek(shape);
  std::cout<<"source "<<index<<' '<<first<<' '<<last<<' '<<ds.ShapeInfo(index).HasReference()
           <<' '<<(images?static_cast<long long>(images->Size()):-1LL)<<'\n';
  for (const auto& block:ds.PaveBlocks(index)) EmitBlock(ds,block);
  if (images) for (const auto& image:*images) std::cout<<"image "<<ds.Index(image)<<'\n';
}
static void Capture(int mode)
{
  const auto sources=Sources(mode);BOPAlgo_PaveFiller filler;
  filler.SetArguments(sources);filler.SetNonDestructive(true);filler.Perform();
  PartitionBuilder builder;builder.SetArguments(sources);builder.SetNonDestructive(true);
  builder.PerformWithFiller(filler);
  if(filler.HasErrors()||builder.HasErrors()) throw Standard_Failure("partition HasErrors=true; expected completed observation");
  auto& ds=*filler.PDS();std::cout<<"mode "<<mode<<' '<<ds.NbSourceShapes()<<'\n';
  for(int index=0;index<ds.NbSourceShapes();++index)
    if(ds.ShapeInfo(index).ShapeType()==TopAbs_EDGE) EmitSource(index,ds,builder);
}
int main()
{
  std::cout<<std::setprecision(17);
  for(int mode=0;mode<5;++mode) Capture(mode);
  return 0;
}
