// Local OCCT 3d097a0 keyboard oracle. Stopped event timers remove scheduler timing from fixtures.
#include <AIS_ViewController.hxx>
#include <iomanip>
#include <iostream>
#include <limits>
#include <string>
class KeyboardProbe : public AIS_ViewController {
public:
  void Frame(double previous, double current) { myLastEventsTime=previous; myEventTimer.Reset(current); }
  void Times(const char* name) { double previous,current;updateEventsTime(previous,current);std::cout<<name<<' '<<previous<<' '<<current<<'\n'; }
  bool HasThrust() const { return myHasThrust; }
  double LastTime() const { return myLastEventsTime; }
  bool NextFrame() const { return toAskNextFrame(); }
  void RequestNext(bool value) { setAskNextFrame(value); }
  void SeedReset() {
    Frame(1.,2.); myMousePressed=Aspect_VKeyMouse_LeftButton; myMouseModifiers=Aspect_VKeyFlags_CTRL;
    myMouseSingleButton=1; myMouseActiveGesture=AIS_MouseGesture_Zoom;
    myMouseClickTimer.Reset(.75); myMouseClickCounter=2; myHasThrust=true; setAskNextFrame(true);
    KeyDown(Aspect_VKey_NavForward,1.25,.25); KeyDown(Aspect_VKey_Shift,1.5);
    myUI.IsNewGesture=myUI.Orientation.ToFitAll=myUI.Orientation.ToSetViewOrient=myUI.MoveTo.ToHilight=true;
    myUI.Selection.ToApplyTool=myUI.Panning.ToStart=myUI.Panning.ToPan=true;
    myUI.Dragging.ToStart=myUI.Dragging.ToConfirm=myUI.Dragging.ToMove=myUI.Dragging.ToStop=true;
    myUI.OrbitRotation.ToStart=myUI.OrbitRotation.ToRotate=myUI.ViewRotation.ToStart=myUI.ViewRotation.ToRotate=myUI.ZRotate.ToRotate=true;
    myUI.Selection.Points.Append(NCollection_Vec2<int>(1,2)); myUI.ZoomActions.Append(Aspect_ScrollDelta(.5));
  }
  void InputBook(const char* name) const {
    std::cout << name << ' ' << myMousePressed << ' ' << myMouseModifiers << ' ' << myMouseSingleButton
      << ' ' << int(myMouseActiveGesture) << ' ' << myMouseClickCounter << ' ' << myMouseClickTimer.ElapsedTime()
      << ' ' << myMouseClickTimer.IsStarted() << ' ' << myLastEventsTime << ' ' << myHasThrust << ' ' << toAskNextFrame()
      << ' ' << Keys().Modifiers() << ' ' << Keys().IsKeyDown(Aspect_VKey_NavForward) << ' ' << Keys().IsFreeKey(Aspect_VKey_NavForward)
      << ' ' << Keys().IsKeyDown(Aspect_VKey_Shift) << ' ' << Keys().IsFreeKey(Aspect_VKey_Shift);
  }
};
static const Aspect_VKey navigationKeys[] = {Aspect_VKey_NavForward,Aspect_VKey_NavBackward,
  Aspect_VKey_NavSlideLeft,Aspect_VKey_NavSlideRight,Aspect_VKey_NavLookLeft,Aspect_VKey_NavLookRight,
  Aspect_VKey_NavLookUp,Aspect_VKey_NavLookDown,Aspect_VKey_NavRollCCW,Aspect_VKey_NavRollCW,
  Aspect_VKey_NavSlideUp,Aspect_VKey_NavSlideDown};
static void walkSnapshot(const std::string& name,const KeyboardProbe& c,const AIS_WalkDelta& w) {
  std::cout << name;
  for (int axis=0;axis<3;++axis) { const auto& p=w[static_cast<AIS_WalkTranslation>(axis)]; std::cout << ' ' << p.Value << ' ' << p.Pressure << ' ' << p.Duration; }
  for (int axis=0;axis<3;++axis) { const auto& p=w[static_cast<AIS_WalkRotation>(axis)]; std::cout << ' ' << p.Value << ' ' << p.Pressure << ' ' << p.Duration; }
  std::cout << ' ' << w.IsEmpty() << ' ' << w.ToMove() << ' ' << w.ToRotate() << ' ' << w.IsDefined()
    << ' ' << w.IsJumping() << ' ' << w.IsCrouching() << ' ' << w.IsRunning()
    << ' ' << c.NextFrame() << ' ' << c.HasThrust() << ' ' << c.LastTime() << ' ' << c.EventTime() << ' ' << c.Keys().Modifiers();
  for (auto key:navigationKeys) std::cout << ' ' << c.Keys().IsKeyDown(key) << ' ' << c.Keys().IsFreeKey(key);
  for (auto key:{Aspect_VKey_Shift,Aspect_VKey_NavJump,Aspect_VKey_NavCrouch}) std::cout << ' ' << c.Keys().IsKeyDown(key) << ' ' << c.Keys().IsFreeKey(key);
  std::cout << '\n';
}
static void fetch(const std::string& name,KeyboardProbe& c,double crouch=.5,double run=2.) {
  auto walk=c.FetchNavigationKeys(crouch,run); walkSnapshot(name,c,walk);
}
static void singles() {
  for (int index=0;index<12;++index) {
    KeyboardProbe c;c.Frame(1.,2.);c.KeyDown(navigationKeys[index],1.25,.25);
    fetch("single-"+std::to_string(index),c);
    c.KeyUp(navigationKeys[index],2.25);c.Frame(2.,3.);fetch("released-"+std::to_string(index),c);
    c.Frame(3.,4.);fetch("consumed-"+std::to_string(index),c);
  }
}
static void pairs() {
  for (int pair=0;pair<6;++pair) {
    KeyboardProbe c;c.Frame(1.,2.);c.KeyDown(navigationKeys[pair*2],1.25,.25);c.KeyDown(navigationKeys[pair*2+1],1.5,.75);
    fetch("pair-"+std::to_string(pair),c);
    c.SetInvertPitch(true);c.Frame(2.,3.);fetch("pair-inverted-"+std::to_string(pair),c);
  }
}
static void stances() {
  for (int mask=0;mask<32;++mask) {
    KeyboardProbe c;c.Frame(1.,2.);c.KeyDown(Aspect_VKey_NavForward,1.25,.25);
    c.KeyDown(Aspect_VKey_NavSlideLeft,1.5,.5);c.KeyDown(Aspect_VKey_NavSlideUp,1.75,.75);
    if(mask&1)c.KeyDown(Aspect_VKey_Shift,1.);
    if(mask&2)c.KeyDown(Aspect_VKey_NavCrouch,1.);
    if(mask&4)c.KeyDown(Aspect_VKey_NavJump,1.);
    fetch("stance-"+std::to_string(mask),c,mask&16?.5:1.,mask&8?2.:1.);
  }
  KeyboardProbe c;c.Frame(1.,2.);c.KeyDown(Aspect_VKey_NavJump,1.);
  fetch("jump-first",c);c.Frame(2.,3.);fetch("jump-released",c);c.Frame(3.,4.);fetch("jump-consumed",c);
}
static void ratios() {
  const double nan=std::numeric_limits<double>::quiet_NaN(),inf=std::numeric_limits<double>::infinity();
  const double ratios[][2]={{-1.,-1.},{nan,inf},{-inf,nan},{inf,inf},{0.,2.}};
  for(int index=0;index<5;++index){
    KeyboardProbe c;c.Frame(1.,2.);c.KeyDown(Aspect_VKey_Shift,1.);c.KeyDown(Aspect_VKey_NavCrouch,1.);
    c.KeyDown(Aspect_VKey_NavForward,1.25,.25);c.KeyDown(Aspect_VKey_NavSlideLeft,1.5,.5);c.KeyDown(Aspect_VKey_NavSlideUp,1.75,.75);
    fetch("ratio-"+std::to_string(index),c,ratios[index][0],ratios[index][1]);
  }
  KeyboardProbe c;c.Frame(1.,2.);c.KeyDown(Aspect_VKey_NavForward,1.25,.25);c.KeyDown(Aspect_VKey_NavForward,1.5,.75);fetch("key-repeated",c);
}
static void pressures() {
  const double values[]={-1.,0.,2.,std::numeric_limits<double>::quiet_NaN(),std::numeric_limits<double>::infinity()};
  for(int index=0;index<12;++index)for(int value=0;value<5;++value){
    KeyboardProbe c;c.Frame(1.,2.);c.KeyDown(navigationKeys[index],1.25,values[value]);
    fetch("pressure-"+std::to_string(index)+"-"+std::to_string(value),c);
  }
}
static void timing() {
  const double nan=std::numeric_limits<double>::quiet_NaN(),inf=std::numeric_limits<double>::infinity();
  const double times[][3]={{1.,2.,3.},{2.,1.,0.},{nan,2.,1.},{1.,nan,1.},{1.,2.,nan},{1.,2.,inf},{1.,2.,-inf},{1.,inf,1.}};
  for(int index=0;index<8;++index){
    KeyboardProbe c;c.Frame(times[index][0],times[index][1]);c.KeyDown(Aspect_VKey_NavForward,times[index][2],.25);
    fetch("timing-"+std::to_string(index),c);
  }
}
static void thrust() {
  const float values[]={0.f,-0.f,1.f,-1.f,std::numeric_limits<float>::denorm_min(),std::numeric_limits<float>::infinity(),-std::numeric_limits<float>::infinity(),std::numeric_limits<float>::quiet_NaN()};
  for(int index=0;index<8;++index){
    KeyboardProbe c;c.SetThrustSpeed(values[index]);c.Frame(1.,2.);fetch("thrust-first-"+std::to_string(index),c);
    c.Frame(2.,3.);fetch("thrust-next-"+std::to_string(index),c);
    c.SetThrustSpeed(0.f);c.RequestNext(false);c.Frame(3.,4.);fetch("thrust-stop-"+std::to_string(index),c);
    c.SetThrustSpeed(values[index]);c.Frame(4.,5.);fetch("thrust-resume-"+std::to_string(index),c);
  }
}
static void axes() {
  const double values[]={-.75,.25,0.,std::numeric_limits<double>::quiet_NaN(),std::numeric_limits<double>::infinity(),-std::numeric_limits<double>::infinity()};
  for(int index=0;index<6;++index){
    KeyboardProbe c;c.Frame(1.,2.);c.KeyFromAxis(Aspect_VKey_NavBackward,Aspect_VKey_NavForward,1.25,values[index]);
    fetch("axis-"+std::to_string(index),c);
  }
  KeyboardProbe c;c.Frame(1.,2.);c.KeyFromAxis(Aspect_VKey_NavBackward,Aspect_VKey_NavForward,1.25,-.75);fetch("axis-negative",c);
  c.Frame(2.,3.);c.KeyFromAxis(Aspect_VKey_NavBackward,Aspect_VKey_NavForward,2.25,.25);fetch("axis-positive",c);
  c.Frame(3.,4.);c.KeyFromAxis(Aspect_VKey_NavBackward,Aspect_VKey_NavForward,3.25,0.);fetch("axis-zero",c);
  c.Frame(4.,5.);fetch("axis-consumed",c);
}
static void inputSnapshot(const char* name,const KeyboardProbe& c) {
  c.InputBook(name);const auto& b=c.InputBuffer(AIS_ViewInputBufferType_UI);
  std::cout << ' ' << b.IsNewGesture << ' ' << b.Orientation.ToFitAll << ' ' << b.Orientation.ToSetViewOrient << ' ' << b.MoveTo.ToHilight << ' ' << b.Selection.ToApplyTool
    << ' ' << b.Panning.ToStart << ' ' << b.Panning.ToPan << ' ' << b.Dragging.ToStart << ' ' << b.Dragging.ToConfirm << ' ' << b.Dragging.ToMove << ' ' << b.Dragging.ToStop << ' ' << b.Dragging.ToAbort
    << ' ' << b.OrbitRotation.ToStart << ' ' << b.OrbitRotation.ToRotate << ' ' << b.ViewRotation.ToStart << ' ' << b.ViewRotation.ToRotate << ' ' << b.ZRotate.ToRotate
    << ' ' << b.Selection.Points.Length() << ' ' << b.Selection.Points.First().x() << ' ' << b.Selection.Points.First().y()
    << ' ' << b.ZoomActions.Length() << ' ' << b.ZoomActions.First().Delta << ' ' << c.InputBuffer(AIS_ViewInputBufferType_GL).Dragging.ToAbort << '\n';
}
static void reset() {
  KeyboardProbe c;c.SeedReset();inputSnapshot("input-seeded",c);
  c.ProcessExpose();c.ProcessConfigure(false);c.ProcessConfigure(true);c.ProcessInput();c.ProcessClose();inputSnapshot("input-noops",c);
  c.ProcessFocus(true);inputSnapshot("input-active",c);
  c.ProcessFocus(false);inputSnapshot("input-focus-lost",c);
  c.ResetViewInput();inputSnapshot("input-reset-again",c);
}
static void times(){KeyboardProbe c;c.Frame(0.,1.);c.Times("events-first");c.Frame(1.,2.);c.Times("events-second");}
int main(){std::cout<<std::setprecision(17);singles();pairs();stances();ratios();pressures();timing();thrust();axes();reset();times();}
