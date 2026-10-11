// Martensite real directional dragging oracle, OCCT 3d097a0.
#include <AIS_LightSource.hxx>
#include <AIS_InteractiveContext.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_ViewerSelector.hxx>
#include <V3d_Viewer.hxx>
#include <V3d_View.hxx>
#include <Aspect_NeutralWindow.hxx>
#include <iostream>
#include <iomanip>
#include "tkservice_probe_null_driver.cxx"
static void dragging_case(int kind,int draggable,int target){
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver);
  const occ::handle<V3d_View> view=viewer->CreateView();
  const occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow;window->SetSize(256,256);view->SetWindow(window);
  const auto camera=view->Camera();camera->SetProjectionType(Graphic3d_Camera::Projection_Orthographic);camera->SetEye(gp_Pnt(0,0,100));camera->SetCenter(gp_Pnt(0,0,0));camera->SetUp(gp_Dir(0,1,0));camera->SetScale(100);camera->SetZRange(1,200);
  const occ::handle<AIS_InteractiveContext> context=new AIS_InteractiveContext(viewer);
  const occ::handle<Graphic3d_CLight> light=new Graphic3d_CLight(Graphic3d_TypeOfLightSource(kind));if(kind==1||kind==3)light->SetDirection(gp_Dir(0,0,-1));
  const occ::handle<AIS_LightSource> object=new AIS_LightSource(light);object->SetZoomable(true);object->SetDraggable(draggable!=0);context->Display(object,false);
  const occ::handle<AIS_LightSourceOwner> owner=new AIS_LightSourceOwner(object,15);
  const NCollection_Vec2<int> from(128,128),to(target==0?128:target==1?140:400,128);
  std::cout<<kind<<' '<<draggable<<' '<<target<<' ';
  for(AIS_DragAction action:{AIS_DragAction_Start,AIS_DragAction_Confirmed,AIS_DragAction_Update,AIS_DragAction_Abort,AIS_DragAction_Stop}){
    const bool result=static_cast<AIS_InteractiveObject*>(object.get())->ProcessDragging(context,view,owner,from,to,action);std::cout<<result<<' ';
    const auto& transform=object->LocalTransformation();for(int row=1;row<=3;++row)for(int col=1;col<=4;++col)std::cout<<transform.Value(row,col)<<' ';
    if(kind==1||kind==3){const auto direction=light->Direction();std::cout<<direction.X()<<' '<<direction.Y()<<' '<<direction.Z()<<' ';}
    else std::cout<<"0 0 0 ";
  }
  std::cout<<'\n';
}
int main(){std::cout<<std::setprecision(17);for(int kind=0;kind<4;++kind)for(int draggable:{0,1})for(int target:{0,1,2})dragging_case(kind,draggable,target);}
