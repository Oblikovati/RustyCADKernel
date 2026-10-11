// Local Martensite oracle for full native touch-to-click transitions.
#include <AIS_ViewController.hxx>
#include <iostream>
#include <iomanip>
#include <limits>
class TouchProbe : public AIS_ViewController {
public:
  int clicks=0;bool twice=false;NCollection_Vec2<int> clicked;
  bool UpdateMouseClick(const NCollection_Vec2<int>& p,unsigned b,unsigned m,bool d) override {
    ++clicks;twice=d;clicked=p;return AIS_ViewController::UpdateMouseClick(p,b,m,d);
  }
  void Seed(double elapsed) { myTouchDoubleTapTimer.Reset(elapsed);myTouchDoubleTapTimer.Start(); }
  void Configure(int index) {
    const float scales[]={1,1,1,0,-1,2,std::numeric_limits<float>::infinity(),std::numeric_limits<float>::quiet_NaN()};
    SetTouchToleranceScale(scales[index%8]);SetAllowDragging(index%2==0);
    myUI.MoveTo.ToHilight=true;myUpdateStartPointPan=myUpdateStartPointRot=false;
  }
  void Snapshot(int index,int step,int op,int id,double x,double y,bool flag,bool result) {
    std::cout<<index<<' '<<step<<' '<<op<<' '<<id<<' '<<x<<' '<<y<<' '<<flag<<' '<<result
      <<' '<<myTouchPoints.Extent()<<' '<<myTouchClick.From.x()<<' '<<myTouchClick.From.y()
      <<' '<<myStartRotCoord.x()<<' '<<myStartRotCoord.y()<<' '<<myStartPanCoord.x()<<' '<<myStartPanCoord.y()
      <<' '<<myUpdateStartPointRot<<' '<<myUpdateStartPointPan<<' '<<myUI.MoveTo.ToHilight
      <<' '<<myUI.IsNewGesture<<' '<<myUI.Dragging.ToStart<<' '<<myUI.Dragging.ToAbort<<' '<<myUI.Dragging.ToStop
      <<' '<<myUI.Dragging.PointStart.x()<<' '<<myUI.Dragging.PointStart.y()<<' '<<myUI.Selection.ToApplyTool
      <<' '<<int(myUI.Selection.Tool)<<' '<<int(myUI.Selection.Scheme)<<' '<<myUI.Selection.Points.Size()
      <<' '<<clicks<<' '<<twice<<' '<<clicked.x()<<' '<<clicked.y()<<' '<<myTouchDoubleTapTimer.IsStarted();
    for(int i=1;i<=3;++i) {
      if(i<=myTouchPoints.Extent()) { const auto& t=myTouchPoints.FindFromIndex(i);
        std::cout<<' '<<myTouchPoints.FindKey(i)<<' '<<t.From.x()<<' '<<t.From.y()<<' '<<t.To.x()<<' '<<t.To.y()<<' '<<t.IsPreciseDevice; }
      else std::cout<<" -1 0 0 0 0 0";
    }
    std::cout<<'\n';
  }
  void Action(int index,int step,int op,int id,double x,double y,bool flag=false) {
    bool result=true;size_t key=id<0?size_t(-1):size_t(id);
    if(op==0) AddTouchPoint(key,NCollection_Vec2<double>(x,y),flag);
    else if(op==1) UpdateTouchPoint(key,NCollection_Vec2<double>(x,y));
    else result=RemoveTouchPoint(key,flag);
    Snapshot(index,step,op,id,x,y,flag,result);
  }
};
int main() {
  std::cout<<std::setprecision(17);
  for(int index=0;index<16;++index) {
    TouchProbe c;c.Configure(index);double start=index==2?-10.8:10.8;
    double delta=index<8?3.0:3.0001;
    c.Action(index,0,0,1,start,20.2);c.Action(index,1,1,1,start+delta,20.2);
    c.Action(index,2,2,1,0,0,index%2==0);c.Action(index,3,1,2,30.5,40.5);
    c.Action(index,4,0,2,35,45);c.Action(index,5,0,3,50,60);c.Action(index,6,0,4,70,80);
    c.Action(index,7,1,3,55,65);c.Action(index,8,2,4,0,0);c.Action(index,9,2,2,0,0);
    c.Action(index,10,2,3,0,0);c.Action(index,11,2,-1,0,0,true);
    c.Action(index,12,0,1,1.8,2.8,true);c.Action(index,13,0,2,3.8,4.8,true);
    c.Action(index,14,2,99,0,0,true);
  }
  for(int index=16;index<20;++index) {
    TouchProbe c;c.Configure(index);c.Seed(index==16?.2:(index==17?1.0:0.0));
    if(index==18) c.SetMouseDoubleClickInterval(-1);
    if(index==19) c.SetMouseDoubleClickInterval(std::numeric_limits<double>::quiet_NaN());
    c.Action(index,0,0,1,1.5,2.5);c.Action(index,1,2,1,0,0);
  }
  for(int index=20;index<22;++index) {
    TouchProbe c;c.Configure(index);
    c.Action(index,0,0,1,1,2);
    c.Action(index,1,1,1,index==20?std::numeric_limits<double>::quiet_NaN():200,
      index==20?200:std::numeric_limits<double>::quiet_NaN());
    c.Action(index,2,1,1,1,200);
  }

}
