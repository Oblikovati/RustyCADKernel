// Martensite direct native edge-selection oracle: actual geometry, polygons, retained output and appended entities.
#include <StdSelect_BRepSelectionTool.hxx>
#include <Select3D_SensitivePoly.hxx>
#include <Select3D_SensitiveSegment.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Edge.hxx>
#include <Poly_Polygon3D.hxx>
#include <gp_Elips.hxx>
#include <gp_Circ.hxx>
#include <iostream>
#include <iomanip>
#include <SelectMgr_SelectingVolumeManager.hxx>
#include <SelectBasics_PickResult.hxx>
static TopoDS_Edge fixture_edge(int index)
{
  if(index==0)return BRepBuilderAPI_MakeEdge(gp_Pnt(1,2,3),gp_Pnt(5,2,3));
  if(index==1)return BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(),2),0.2,3.4);
  if(index==2)return BRepBuilderAPI_MakeEdge(gp_Elips(gp_Ax2(),3,1),0.2,5.4);
  TopoDS_Edge edge;BRep_Builder builder;builder.MakeEdge(edge);
  if(index==3)return edge;
  if(index==4)return BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(),1.e-8),0.,3.);
  NCollection_Array1<gp_Pnt> points(-2,index==5?-1:0);
  for(int i=points.Lower();i<=points.Upper();++i)points(i)=gp_Pnt(i+2,(i+2)*(i+2),1);
  builder.UpdateEdge(edge,new Poly_Polygon3D(points));
  return edge;
}
static void report_point(const gp_Pnt& point)
{
  std::cout<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z();
}
static void edge_case(int index)
{
  const occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(7);
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  occ::handle<Select3D_SensitiveEntity> output=new Select3D_SensitivePoint(owner,gp_Pnt(9,8,7));
  auto sentinel=output;
  StdSelect_BRepSelectionTool::GetEdgeSensitive(fixture_edge(index),owner,selection,0.1,0.5,9,500.,output);
  std::cout<<"edge "<<index<<' '<<output->DynamicType()->Name()<<' '<<output->NbSubElements()
    <<' '<<selection->Entities().Size()<<' '<<(output==sentinel);
  if(auto segment=occ::down_cast<Select3D_SensitiveSegment>(output))
  {report_point(segment->StartPoint());report_point(segment->EndPoint());}
  else if(auto poly=occ::down_cast<Select3D_SensitivePoly>(output))
  {auto points=poly->Points3D();for(int i=points->Lower();i<=points->Upper();++i)report_point(points->Value(i));}
  else if(auto point=occ::down_cast<Select3D_SensitivePoint>(output))report_point(point->Point());
  std::cout<<'\n';
}
static void edge_pick_case(int index,int ray)
{
  const occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(7);
  const occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  occ::handle<Select3D_SensitiveEntity> output=new Select3D_SensitivePoint(owner,gp_Pnt(9,8,7));
  StdSelect_BRepSelectionTool::GetEdgeSensitive(fixture_edge(index),owner,selection,0.1,0.5,9,500.,output);
  const gp_Pnt origins[]={gp_Pnt(0,0,10),gp_Pnt(2,2,10),gp_Pnt(2,0,10),gp_Pnt(9,8,10)};
  SelectMgr_SelectingVolumeManager volume;
  gp_Pnt origin=origins[ray<4?ray:0];
  if(ray==4)
  {
    gp_Pnt start;
    if(auto segment=occ::down_cast<Select3D_SensitiveSegment>(output))start=segment->StartPoint();
    else if(auto poly=occ::down_cast<Select3D_SensitivePoly>(output))start=poly->Points3D()->First();
    else start=occ::down_cast<Select3D_SensitivePoint>(output)->Point();
    origin.SetCoord(start.X(),start.Y(),10.);
  }
  volume.InitAxisSelectingVolume(gp_Ax1(origin,gp_Dir(0,0,-1)));
  volume.BuildSelectingVolume();
  SelectBasics_PickResult pick;
  bool hit=output->Matches(volume,pick);
  std::cout<<"pick "<<index<<' '<<ray<<' '<<hit;
  if(hit)std::cout<<' '<<pick.Depth()<<' '<<pick.DistToGeomCenter();
  for(const auto& selected:selection->Entities())
  {
    SelectBasics_PickResult extra;
    bool appended=selected->BaseSensitive()->Matches(volume,extra);
    std::cout<<" appended "<<appended;
    if(appended)std::cout<<' '<<extra.Depth()<<' '<<extra.DistToGeomCenter();
  }
  std::cout<<'\n';
}
int main(int argc,char**)
{
  std::cout<<std::setprecision(17);
  if(argc==1)for(int index=0;index<7;++index)edge_case(index);
  else
  {
    for(int index=0;index<7;++index)for(int ray=0;ray<4;++ray)edge_pick_case(index,ray);
    for(int index=0;index<7;++index)edge_pick_case(index,4);
  }
}
