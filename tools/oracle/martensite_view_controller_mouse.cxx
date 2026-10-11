// Local Martensite oracle: mouse transitions against pinned OCCT 3d097a0.
#include <AIS_ViewController.hxx>
#include <iostream>
#include <iomanip>
#include <limits>
class MouseProbe : public AIS_ViewController {
public:
  void Configure(int index) {
    if(index>=100) {
      if(index>=120) {myMouseClickCounter=1;myMouseClickTimer.Reset(index%2==0?.1:1.0);myMouseClickTimer.Start();}
      if(index==124) SetMouseDoubleClickInterval(-1);
      if(index==125) SetMouseDoubleClickInterval(std::numeric_limits<double>::quiet_NaN());
      return;
    }
    const float scales[]={1,0,-1,2,std::numeric_limits<float>::infinity(),std::numeric_limits<float>::quiet_NaN()};
    SetTouchToleranceScale(scales[(index/10)%6]);
    ChangeMouseGestureMap().Clear();
    ChangeMouseGestureMap().Bind(Aspect_VKeyMouse_LeftButton,AIS_MouseGesture(index%10));
    SetAllowRotation(index<60 || index%4!=0);SetAllowPanning(index<60 || index%4!=1);
    SetAllowZooming(index<60 || index%4!=2);SetAllowDragging(index<60 || index%4!=3);
    if(index>=80) SetNavigationMode(AIS_NavigationMode_FirstPersonWalk);
    SetOrbitAcceleration(index%2==0?1.5f:-.5f);SetMouseAcceleration(2.f);
  }
  void Snapshot(bool result) {
    std::cout<<' '<<result<<' '<<int(myMouseActiveGesture)<<' '<<myMouseSingleButton<<' '<<myMouseClickCounter
      <<' '<<myMouseClickTimer.IsStarted()<<' '<<myMousePressed<<' '<<myMouseModifiers
      <<' '<<myMousePressPoint.x()<<' '<<myMousePressPoint.y()<<' '<<myMouseProgressPoint.x()<<' '<<myMouseProgressPoint.y()
      <<' '<<myMousePositionLast.x()<<' '<<myMousePositionLast.y()<<' '<<myMouseActiveIdleRotation<<' '<<myMouseStopDragOnUnclick
      <<' '<<myUpdateStartPointRot<<' '<<myUpdateStartPointPan<<' '<<myUI.IsNewGesture
      ;
    Requests();Selection();Motion();
    std::cout<<'\n';
  }
  void Requests() {
    std::cout      <<' '<<myUI.Dragging.ToStart<<' '<<myUI.Dragging.ToConfirm<<' '<<myUI.Dragging.ToMove<<' '<<myUI.Dragging.ToStop<<' '<<myUI.Dragging.ToAbort
      <<' '<<myUI.Dragging.PointStart.x()<<' '<<myUI.Dragging.PointStart.y()<<' '<<myUI.Dragging.PointTo.x()<<' '<<myUI.Dragging.PointTo.y()
      <<' '<<myUI.MoveTo.ToHilight<<' '<<myUI.MoveTo.Point.x()<<' '<<myUI.MoveTo.Point.y()
;
  }
  void Selection() {
    std::cout      <<' '<<int(myUI.Selection.Tool)<<' '<<int(myUI.Selection.Scheme)<<' '<<myUI.Selection.ToApplyTool<<' '<<myUI.Selection.Points.Size();
    for(int i=1;i<=3;++i) {
      if(i<=myUI.Selection.Points.Size()) std::cout<<' '<<myUI.Selection.Points.Value(i).x()<<' '<<myUI.Selection.Points.Value(i).y();
      else std::cout<<" 0 0";
    }
  }
  void Motion() {
    std::cout<<' '<<myUI.Panning.ToStart<<' '<<myUI.Panning.PointStart.x()<<' '<<myUI.Panning.PointStart.y()
      <<' '<<myUI.Panning.ToPan<<' '<<myUI.Panning.Delta.x()<<' '<<myUI.Panning.Delta.y();
    Rotation(myUI.OrbitRotation);Rotation(myUI.ViewRotation);
    std::cout<<' '<<myUI.ZoomActions.Size();
    if(!myUI.ZoomActions.IsEmpty()) {const auto& z=myUI.ZoomActions.Last();std::cout<<' '<<z.Delta<<' '<<z.Point.x()<<' '<<z.Point.y()<<' '<<z.Flags;}
    else std::cout<<" 0 0 0 0";
  }
  template<class R> void Rotation(const R& r) {
    std::cout<<' '<<r.ToStart<<' '<<r.PointStart.x()<<' '<<r.PointStart.y()<<' '<<r.ToRotate<<' '<<r.PointTo.x()<<' '<<r.PointTo.y();
  }
  void Action(int index,int step,int op,int x,int y,unsigned buttons,unsigned modifiers,bool emulated) {
    bool result=op==0?UpdateMouseButtons(NCollection_Vec2<int>(x,y),buttons,modifiers,emulated):UpdateMousePosition(NCollection_Vec2<int>(x,y),buttons,modifiers,emulated);
    std::cout<<index<<' '<<step<<' '<<op<<' '<<x<<' '<<y<<' '<<buttons<<' '<<modifiers<<' '<<emulated;
    Snapshot(result);
  }
};
void MotionCases();
void ClickCases();
int main() {
  std::cout<<std::setprecision(17);
  MotionCases();ClickCases();
}

void MotionCases() {
  for(int index=0;index<100;++index) {
    MouseProbe c;c.Configure(index);bool e=index>=10 && index<60;
    c.Action(index,0,0,10,20,Aspect_VKeyMouse_LeftButton,0,e);
    c.Action(index,1,1,10,20,Aspect_VKeyMouse_LeftButton,0,e);
    c.Action(index,2,1,13,20,Aspect_VKeyMouse_LeftButton,0,e);
    c.Action(index,3,1,30,27,Aspect_VKeyMouse_LeftButton,0,e);
    c.Action(index,4,1,35,25,Aspect_VKeyMouse_LeftButton,0,e);
    c.Action(index,5,0,35,25,0,0,e);
    c.Action(index,6,1,36,26,0,0,e);
    c.Action(index,7,0,36,26,Aspect_VKeyMouse_LeftButton,0,e);
    c.Action(index,8,0,36,26,Aspect_VKeyMouse_LeftButton,0,e);
    c.Action(index,9,1,38,28,Aspect_VKeyMouse_LeftButton,Aspect_VKeyFlags_SHIFT,e);
    c.Action(index,10,0,38,28,0,Aspect_VKeyFlags_SHIFT,e);
  }
}
void ClickCases() {
  for(int index=100;index<130;++index) {
    MouseProbe c;c.Configure(index);
    const unsigned buttons[]={Aspect_VKeyMouse_LeftButton,Aspect_VKeyMouse_RightButton,Aspect_VKeyMouse_MiddleButton,
      Aspect_VKeyMouse_LeftButton|Aspect_VKeyMouse_RightButton,1};
    const unsigned b=buttons[index%5];const unsigned m=index%2==0?0:Aspect_VKeyFlags_SHIFT;
    const int x=index<110?2:(index<120?3:0);
    c.Action(index,0,0,0,0,b,m,false);c.Action(index,1,0,x,0,0,m,false);
    c.Action(index,2,0,x,0,b,m,false);c.Action(index,3,0,x,0,0,m,false);
  }}
