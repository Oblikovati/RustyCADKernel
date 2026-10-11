// Martensite CPU oracle: actual constructor, sensitive owners, highlights and timer-backed clicks.
#include <AIS_ViewCube.hxx>
#include <AIS_AnimationCamera.hxx>
#include <AIS_InteractiveContext.hxx>
#include <AIS_PointCloud.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_SensitiveEntity.hxx>
#include <SelectMgr_SelectingVolumeManager.hxx>
#include <Graphic3d_ArrayOfPoints.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <SelectMgr_AxisIntersector.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <Media_Timer.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <Prs3d_Drawer.hxx>
#include <iomanip>
#include <iostream>
#include "tkservice_probe_null_driver.cxx"
struct HighlightCounts {int arrays=0,vertices=0,indices=0;};
class WidgetHighlightGroup:public SvGroup {
  HighlightCounts& counts;
public:
  WidgetHighlightGroup(const occ::handle<Graphic3d_Structure>& structure,HighlightCounts& value):SvGroup(structure),counts(value){}
  void AddPrimitiveArray(Graphic3d_TypeOfPrimitiveArray,const occ::handle<Graphic3d_IndexBuffer>& index,const occ::handle<Graphic3d_Buffer>& vertices,const occ::handle<Graphic3d_BoundBuffer>&,bool) override {
    ++counts.arrays;counts.vertices+=vertices->NbElements;counts.indices+=index.IsNull()?0:index->NbElements;
  }
};
class WidgetHighlightStructure:public SvCStructure {
  HighlightCounts& counts;
public:
  WidgetHighlightStructure(const occ::handle<Graphic3d_StructureManager>& manager,HighlightCounts& value):SvCStructure(manager),counts(value){}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& structure) override {
    auto group=occ::handle<Graphic3d_Group>(new WidgetHighlightGroup(structure,counts));myGroups.Append(group);return group;
  }
};
class WidgetHighlightDriver:public SvDriver {
  HighlightCounts& counts;
public:
  WidgetHighlightDriver(HighlightCounts& value):counts(value){}
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& manager) override {return new WidgetHighlightStructure(manager,counts);}
};
class WidgetContext:public AIS_InteractiveContext {
public:
  WidgetContext(const occ::handle<V3d_Viewer>& viewer):AIS_InteractiveContext(viewer){}
  void activate(const occ::handle<V3d_View>& view) {myLastActiveView=view.get();}
};
class SeekAnimation:public AIS_AnimationCamera {
public:
  double next=0;
  SeekAnimation():AIS_AnimationCamera("AIS_ViewCube",{}) {SetOwnDuration(0.5);}
  void StartTimer(double start,double speed,bool update,bool) override {
    AIS_AnimationCamera::StartTimer(start,speed,update,true);
  }
  double UpdateTimer() override {Timer()->Stop();Timer()->Seek(next);return AIS_AnimationCamera::UpdateTimer();}
};
class WidgetCube:public AIS_ViewCube {
public:
  int frames=0,finished=0;bool finishAttached=false;
  using AIS_ViewCube::createBoxPartTriangles;
  occ::handle<Graphic3d_Camera> start() const {return myStartState;}
  occ::handle<Graphic3d_Camera> end() const {return myEndState;}
  void onAfterAnimation() override {
    ++frames;auto tick=occ::down_cast<SeekAnimation>(ViewAnimation());if(!tick.IsNull())tick->next+=0.1;
  }
  void onAnimationFinished() override {++finished;finishAttached=!ViewAnimation()->View().IsNull();}
};
class WidgetSensitive:public AIS_ViewCubeSensitive {
public:
  WidgetSensitive(const occ::handle<SelectMgr_EntityOwner>& owner,const occ::handle<Graphic3d_ArrayOfTriangles>& triangles):AIS_ViewCubeSensitive(owner,triangles){}
  using AIS_ViewCubeSensitive::isValidRay;
};
static void xyz(const gp_XYZ& point) {std::cout<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z();}
static void camera(const char* kind,int id,const occ::handle<Graphic3d_Camera>& value) {
  std::cout<<kind<<' '<<id;xyz(value->Eye().XYZ());xyz(value->Center().XYZ());xyz(value->Up().XYZ());xyz(value->Direction().XYZ());
  std::cout<<' '<<value->Scale()<<' '<<value->Distance()<<' '<<value->Aspect()<<' '<<int(value->ProjectionType())<<'\n';
}
struct WidgetScene {
  occ::handle<V3d_Viewer> viewer;occ::handle<V3d_View> view;occ::handle<WidgetContext> context;occ::handle<WidgetCube> cube;
  WidgetScene(bool perspective=false) {
    viewer=new V3d_Viewer(new SvDriver());view=viewer->CreateView();auto window=occ::handle<Aspect_NeutralWindow>(new Aspect_NeutralWindow());window->SetSize(640,480);
    view->SetWindow(window);view->SetImmediateUpdate(false);view->SetAutoZFitMode(false);context=new WidgetContext(viewer);context->activate(view);
    auto c=view->Camera();c->SetEyeAndCenter(gp_Pnt(10,20,200),gp_Pnt(2,3,4));c->SetUp(gp_Dir(1,3,1));c->OrthogonalizeUp();c->SetScale(200);c->SetAspect(4.0/3.0);
    c->SetProjectionType(perspective?Graphic3d_Camera::Projection_Perspective:Graphic3d_Camera::Projection_Orthographic);
    cube=new WidgetCube();cube->SetContext(context);
  }
  void clouds(int policy) {
    if(policy==0)return;
    auto points=occ::handle<Graphic3d_ArrayOfPoints>(new Graphic3d_ArrayOfPoints(2));points->AddVertex(gp_Pnt(-2,-3,-4));points->AddVertex(gp_Pnt(1,5,7));
    auto object=occ::handle<AIS_PointCloud>(new AIS_PointCloud());object->SetPoints(points);context->Display(object,false);
    if(policy==2)context->SetSelected(object,false);
  }
};
static void defaults() {
  WidgetScene s;auto cube=s.cube;auto dyn=cube->DynamicHilightAttributes();auto color=dyn->ShadingAspect()->Color();
  std::cout<<"defaults "<<cube->Duration()<<' '<<cube->HasAnimation()<<' '<<cube->ToAutoStartAnimation()<<' '<<cube->IsFixedAnimationLoop()<<' '<<cube->ToFitSelected()<<' '<<cube->ToResetCameraUp()<<' '<<cube->IsAutoHilight()<<' '<<cube->GlobalSelOwner().IsNull()<<' '<<int(dyn->ShadingAspect()->Aspect()->ShadingModel())<<' '<<dyn->ZLayer()<<' '<<color.Red()<<' '<<color.Green()<<' '<<color.Blue()<<' '<<(dyn->Link()==cube->Attributes())<<'\n';
  camera("default-start",0,cube->start());camera("default-end",0,cube->end());
}
static void selection(int policy) {
  WidgetScene s;if(policy==1)s.cube->SetRoundRadius(0.2);if(policy==2){s.cube->SetDrawEdges(false);s.cube->SetDrawVertices(false);}
  auto selected=occ::handle<SelectMgr_Selection>(new SelectMgr_Selection());s.cube->ComputeSelection(selected,0);
  std::cout<<"selection "<<policy<<' '<<selected->Entities().Size()<<'\n';
  for(const auto& item:selected->Entities()) {
    auto entity=item->BaseSensitive();auto owner=occ::down_cast<AIS_ViewCubeOwner>(entity->OwnerId());auto bounds=entity->BoundingBox();
    std::cout<<"sensitive "<<policy<<' '<<int(owner->MainOrientation())<<' '<<owner->Priority()<<' '<<owner->ComesFromDecomposition()<<' '<<owner->IsForcedHilight()<<' '<<entity->SensitivityFactor()<<' '<<entity->NbSubElements();
    xyz(entity->CenterOfGeometry().XYZ());std::cout<<' '<<bounds.CornerMin().x()<<' '<<bounds.CornerMin().y()<<' '<<bounds.CornerMin().z()<<' '<<bounds.CornerMax().x()<<' '<<bounds.CornerMax().y()<<' '<<bounds.CornerMax().z()<<' '<<(owner->Selectable()==s.cube)<<'\n';
  }
  s.cube->ComputeSelection(selected,1);std::cout<<"unsupported "<<policy<<' '<<selected->Entities().Size()<<'\n';
}
static void rayPolicy() {
  WidgetScene s;int nodes=0,tris=0;s.cube->createBoxPartTriangles({},nodes,tris,V3d_Zpos);
  auto array=occ::handle<Graphic3d_ArrayOfTriangles>(new Graphic3d_ArrayOfTriangles(nodes,tris*3));nodes=tris=0;s.cube->createBoxPartTriangles(array,nodes,tris,V3d_Zpos);
  auto owner=occ::handle<AIS_ViewCubeOwner>(new AIS_ViewCubeOwner(s.cube,V3d_Zpos));WidgetSensitive sensitive(owner,array);
  for(double degrees:{0.,79.9,80.,80.1,89.9,90.,99.9,100.,100.1,180.}) {
    double angle=degrees*M_PI/180.;gp_Dir direction(std::sin(angle),0,std::cos(angle));SelectMgr_SelectingVolumeManager mgr;
    auto axis=occ::handle<SelectMgr_AxisIntersector>(new SelectMgr_AxisIntersector());axis->Init(gp_Ax1(gp_Pnt(0,0,100),direction));mgr.InitSelectingVolume(axis);mgr.BuildSelectingVolume();
    SelectBasics_PickResult pick;bool hit=sensitive.Matches(mgr,pick);std::cout<<"ray "<<degrees<<' '<<sensitive.isValidRay(mgr)<<' '<<hit<<' '<<(hit?pick.Depth():-1)<<'\n';
  }
}
static void target(int id) {
  WidgetScene s(id/78!=0);int orientation=id%26,policy=(id/26)%3;s.clouds(policy);s.cube->SetYup(id%2==1,false);s.cube->SetResetCamera(id%3==1);s.cube->SetFitSelected(id%4!=0);
  auto owner=occ::handle<AIS_ViewCubeOwner>(new AIS_ViewCubeOwner(s.cube,V3d_TypeOfOrientation(orientation)));auto original=s.view->Camera();
  auto tick=occ::handle<SeekAnimation>(new SeekAnimation());s.cube->SetViewAnimation(tick);s.cube->StartAnimation(owner);
  camera("start",id,s.cube->start());camera("end",id,s.cube->end());
  std::cout<<"start-state "<<id<<' '<<s.cube->HasAnimation()<<' '<<(tick->View()==s.view)<<' '<<(tick->CameraStart()==s.cube->start())<<' '<<(tick->CameraEnd()==s.cube->end())<<' '<<(s.view->Camera()==original)<<'\n';
  int frameIndex=0;for(double pts:{0.,0.125,0.25,0.5}) {tick->next=pts;bool continuing=s.cube->UpdateAnimation(false);camera("frame",id*4+frameIndex++,s.view->Camera());std::cout<<"step "<<id<<' '<<pts<<' '<<continuing<<' '<<s.cube->frames<<' '<<s.cube->finished<<' '<<s.cube->finishAttached<<' '<<tick->View().IsNull()<<'\n';}
}
static void clicks(int policy) {
  WidgetScene s;auto tick=occ::handle<SeekAnimation>(new SeekAnimation());s.cube->SetViewAnimation(tick);s.cube->SetAutoStartAnimation(policy!=0);s.cube->SetFixedAnimationLoop(policy!=1);
  auto owner=occ::handle<AIS_ViewCubeOwner>(new AIS_ViewCubeOwner(s.cube,V3d_Xpos));bool handled=owner->HandleMouseClick(NCollection_Vec2<int>(13,17),Aspect_VKeyMouse_RightButton,Aspect_VKeyFlags_SHIFT,true);
  std::cout<<"click "<<policy<<' '<<handled<<' '<<s.cube->HasAnimation()<<' '<<s.cube->frames<<' '<<s.cube->finished<<' '<<s.cube->finishAttached<<' '<<tick->View().IsNull()<<'\n';
}
static void highlight(int id) {
  WidgetScene scene;auto cube=scene.cube;HighlightCounts counts;
  auto structures=occ::handle<Graphic3d_StructureManager>(new Graphic3d_StructureManager(new WidgetHighlightDriver(counts)));
  auto manager=occ::handle<PrsMgr_PresentationManager>(new PrsMgr_PresentationManager(structures));
  auto style=occ::handle<Prs3d_Drawer>(new Prs3d_Drawer());style->SetLink(cube->Attributes());style->SetShadingAspect(new Prs3d_ShadingAspect());style->SetZLayer(id%2?7:Graphic3d_ZLayerId_UNKNOWN);
  if(id%6>=3)cube->SetRoundRadius(0.2);if(id>=6){cube->SetDrawEdges(false);cube->SetDrawVertices(false);}
  auto owner=occ::handle<AIS_ViewCubeOwner>(new AIS_ViewCubeOwner(cube,V3d_TypeOfOrientation(id%3==0?0:id%3==1?6:18)));
  if(id<9)manager->BeginImmediateDraw();cube->HilightOwnerWithColor(manager,style,owner);
  auto prs=cube->GetHilightPresentation(manager);
  std::cout<<"highlight "<<id<<' '<<prs->Groups().Length()<<' '<<counts.arrays<<' '<<counts.vertices<<' '<<counts.indices<<' '<<prs->GetZLayer()<<' '<<(prs->TransformPersistence()==cube->TransformPersistence())<<' '<<(prs->CStructure()->ViewAffinity==cube->ViewAffinity())<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);defaults();for(int policy=0;policy<3;++policy)selection(policy);rayPolicy();
  for(int id=0;id<156;++id)target(id);for(int policy=0;policy<3;++policy)clicks(policy);for(int id=0;id<12;++id)highlight(id);
}
