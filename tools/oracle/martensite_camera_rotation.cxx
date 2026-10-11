// Reuse pinned retained scene, open headset, camera and failure helpers; only old entry points are renamed.
#include "martensite_navigation_motion_oracle_shared.cxx"
static const NCollection_Vec2<double> rotation_pixels[] = {
  NCollection_Vec2<double>(0,0),NCollection_Vec2<double>(80,0),NCollection_Vec2<double>(0,60),NCollection_Vec2<double>(80,60),NCollection_Vec2<double>(-80,-60),NCollection_Vec2<double>(10000,-10000),
  NCollection_Vec2<double>(0,1000),NCollection_Vec2<double>(0,-1000),NCollection_Vec2<double>(1e-14,1e-14),NCollection_Vec2<double>(1e-9,0),NCollection_Vec2<double>(0,1e-9),NCollection_Vec2<double>(320,240)};
static const NCollection_Vec3<double> rotation_extras[] = {
  NCollection_Vec3<double>(0,0,0),NCollection_Vec3<double>(.1,0,.21),NCollection_Vec3<double>(0,.1,.21),NCollection_Vec3<double>(-.2,-.3,-.14),NCollection_Vec3<double>(.2,.4,.42),
  NCollection_Vec3<double>(0,10,.21),NCollection_Vec3<double>(0,-10,.21),NCollection_Vec3<double>(1e-14,1e-14,.21),NCollection_Vec3<double>(1e-9,0,.21)};
static bool rotation_is_orbit(int index) {
  const int orbitCaseLimit=1920;
  return index<orbitCaseLimit;
}
static int rotation_local(int index) {
  const int viewCaseOffset=1920;
  return rotation_is_orbit(index)?index:index-viewCaseOffset;
}
static int rotation_flags(int index) {
  const int local=rotation_local(index);
  return (local/5)%4;
}
static int rotation_motion(int index) {
  const int count=rotation_is_orbit(index)?12:9;
  return (rotation_local(index)/20)%count;
}
static int rotation_options(int index) {
  const int stride=rotation_is_orbit(index)?240:180;
  return rotation_local(index)/stride;
}
class RotationProbe : public AIS_ViewController {
public:
  void seed(int index,const occ::handle<Graphic3d_Camera>& camera) {
    seed_buffer(myUI,index,0);seed_buffer(myGL,index,1);
    const int flags=rotation_flags(index),options=rotation_options(index);
    myGL.OrbitRotation.ToStart=myGL.ViewRotation.ToStart=(flags&1)!=0;
    myGL.OrbitRotation.ToRotate=myGL.ViewRotation.ToRotate=(flags&2)!=0;
    myGL.OrbitRotation.PointStart=myGL.ViewRotation.PointStart=NCollection_Vec2<double>(137,91);
    const auto delta=rotation_pixels[rotation_motion(index)];
    myGL.OrbitRotation.PointTo=myGL.ViewRotation.PointTo=myGL.OrbitRotation.PointStart+delta;
    SetAllowRotation(options<4);setAskNextFrame(false);
    myRotatePnt3d=gp_Pnt(7,-3,20);myCamStartOpUp=camera->Up();myCamStartOpDir=camera->Direction();
    myCamStartOpEye=camera->Eye();myCamStartOpCenter=camera->Center();myCurrentRollAngle=.21;
  }
  void frame(int frame) {
    if(frame<2)return;
    myGL.OrbitRotation.ToStart=myGL.ViewRotation.ToStart=false;
    myGL.OrbitRotation.PointTo+=NCollection_Vec2<double>(13,-17);
    myGL.ViewRotation.PointTo+=NCollection_Vec2<double>(13,-17);
  }
  void apply(int index,const occ::handle<V3d_View>& view) {
    if(rotation_is_orbit(index)){handleOrbitRotation(view,gp_Pnt(9,-5,12),rotation_options(index)%2==0);return;}
    const auto extra=rotation_extras[rotation_motion(index)];
    handleViewRotation(view,extra.x(),extra.y(),extra.z(),rotation_options(index)%2!=0);
  }
  void apply_mixed(int index,int frame,const occ::handle<V3d_View>& view) {
    if(frame==2){handleViewRotation(view,.15,-.25,-.14,index%2!=0);return;}
    handleOrbitRotation(view,gp_Pnt(9,-5,12),rotation_options(index)%2==0);
  }
  void degenerate_starts() {
    myCamStartOpUp=myCamStartOpDir=gp::DX();
    myGL.OrbitRotation.ToStart=myGL.ViewRotation.ToStart=false;
    myGL.OrbitRotation.ToRotate=myGL.ViewRotation.ToRotate=true;
  }
  void apply_passive(const occ::handle<V3d_View>& view) {
    // Matching retained roll isolates native start-only and empty request guards.
    handleViewRotation(view,0,0,myCurrentRollAngle,false);
  }
  void emit_state() const {
    for(auto p:{myRotatePnt3d,myCamStartOpEye,myCamStartOpCenter})std::cout<<' '<<p.X()<<' '<<p.Y()<<' '<<p.Z();
    for(auto d:{myCamStartOpUp,myCamStartOpDir})std::cout<<' '<<d.X()<<' '<<d.Y()<<' '<<d.Z();
    std::cout<<' '<<myCurrentRollAngle<<' '<<toAskNextFrame();
    emit_buffer(myUI);emit_buffer(myGL);
  }
};
static PrimitiveScene rotation_scene(int index) {
  auto scene=navigation_cpu_scene();auto camera=scene.view->Camera();
  camera->SetEyeAndCenter(gp_Pnt(20,30,100),gp_Pnt(1,-2,3));camera->SetUp(gp_Dir(0,1,.2));
  camera->SetProjectionType(Graphic3d_Camera::Projection(rotation_local(index)%5));
  if(rotation_options(index)%4>=2)navigation_xr(scene,1440);
  return scene;
}
static void rotation_snapshot(int index,int frame,const RotationProbe& c,const PrimitiveScene& s,bool failed) {
  std::cout<<index<<' '<<frame<<' '<<failed<<' '<<s.view->View()->IsInvalidated()
    <<' '<<c.ViewAnimation()->IsStopped()<<' '<<c.ViewAnimation()->View().IsNull();
  emit_camera(s.view->Camera());navigation_optional_camera(s.view->View()->BaseXRCamera());
  navigation_optional_camera(s.view->View()->PosedXRCamera());
  c.emit_state();std::cout<<'\n';
}
static bool rotation_meaningful(int index) {
  if(rotation_options(index)<4)return true;
  return rotation_local(index)%5==0&&rotation_motion(index)==0&&rotation_options(index)%4<2;
}
static bool rotation_apply_failed(int index,RotationProbe& c,const PrimitiveScene& scene) {
  try{c.apply(index,scene.view);}catch(const Standard_Failure&){return true;}
  return false;
}
static void rotation_case(int index,bool failure) {
  auto scene=rotation_scene(index);RotationProbe c;c.seed(index,scene.view->Camera());
  c.ViewAnimation()->SetView(scene.view);
  if(failure)c.ViewAnimation()->Add(new PrimitiveStopFailure());
  c.ViewAnimation()->Start(false);
  for(int frame=0;frame<4;++frame){
    scene.view->View()->Redraw();c.frame(frame);bool failed=false;
    if(frame)failed=rotation_apply_failed(index,c,scene);
    if(failed&&frame!=1)throw Standard_Failure("rotation failure expected only before first camera change");
    if(failed&&!failure)throw Standard_Failure("rotation expected success but failed");
    rotation_snapshot(index+(failure?10000:0),frame,c,scene,failed);
  }
}
static void rotation_mixed_case(int index) {
  auto scene=rotation_scene(index);RotationProbe c;c.seed(index,scene.view->Camera());
  c.ViewAnimation()->SetView(scene.view);c.ViewAnimation()->Start(false);
  for(int frame=0;frame<4;++frame){
    scene.view->View()->Redraw();c.frame(frame);
    if(frame)c.apply_mixed(index,frame,scene.view);
    rotation_snapshot(20000+index,frame,c,scene,false);
  }
}
static void rotation_degenerate_case(int index) {
  auto scene=rotation_scene(index);RotationProbe c;c.seed(index,scene.view->Camera());c.degenerate_starts();
  c.ViewAnimation()->SetView(scene.view);c.ViewAnimation()->Start(false);
  for(int frame=0;frame<4;++frame){
    scene.view->View()->Redraw();c.frame(frame);
    bool failed=frame!=0&&rotation_apply_failed(index,c,scene);
    rotation_snapshot(30000+index,frame,c,scene,failed);
  }
}
static void rotation_passive_case(int index) {
  auto scene=rotation_scene(index);RotationProbe c;c.seed(index,scene.view->Camera());
  c.ViewAnimation()->SetView(scene.view);c.ViewAnimation()->Start(false);
  for(int frame=0;frame<4;++frame){
    scene.view->View()->Redraw();c.frame(frame);
    if(frame)c.apply_passive(scene.view);
    rotation_snapshot(40000+index,frame,c,scene,false);
  }
}
int main(int argc,char** argv) {
  std::cout<<std::setprecision(17);
  if(argc>1&&std::string(argv[1])=="passive"){for(int options:{0,2,4})for(int flags=0;flags<4;++flags)rotation_passive_case(1920+180*options+5*flags);return 0;}
  if(argc>1&&std::string(argv[1])=="degenerate"){for(int index:{60,300,540,780,1980,2160,2340,2520})rotation_degenerate_case(index);return 0;}
  if(argc>1&&std::string(argv[1])=="mixed"){for(int offset=0;offset<4;++offset)for(int projection=0;projection<5;++projection)rotation_mixed_case(15+240*offset+projection);return 0;}
  if(argc>1){for(int index:{0,5,10,15,250,255,490,495,730,735,1920,1935,2120,2300,2660,2900})rotation_case(index,true);return 0;}
  for(int index=0;index<3360;++index)if(rotation_meaningful(index))rotation_case(index,false);
  return 0;
}
