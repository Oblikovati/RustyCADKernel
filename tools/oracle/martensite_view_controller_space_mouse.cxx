// Local Martensite OCCT 3d097a0 oracle; stopped event timer removes scheduler variation.
#include <AIS_ViewController.hxx>
#include <WNT_HIDSpaceMouse.hxx>
#include <array>
#include <iostream>
#include <iomanip>
#include <limits>
class SpaceMouseProbe : public AIS_ViewController {
public:
  void Configure(int index) {
    const float translation[]={2,0,-2,std::numeric_limits<float>::infinity(),std::numeric_limits<float>::quiet_NaN()};
    const float rotation[]={4,0,-1,std::numeric_limits<float>::infinity(),std::numeric_limits<float>::quiet_NaN()};
    my3dMouseIsQuadric=index%2==0;my3dMouseAccelTrans=translation[index%5];my3dMouseAccelRotate=rotation[index%5];
    SetAllowRotation(index%3!=0);
    my3dMouseNoRotate.SetValues((index/3)&1,(index/3)&2,(index/3)&4);
    my3dMouseToReverse.SetValues((index/7)&1,(index/7)&2,(index/7)&4);
  }
  void Action(int index,int step,unsigned long product,const std::array<unsigned char,13>& bytes,int count) {
    const double time=1.+step*.5;myEventTimer.Reset(time);
    WNT_HIDSpaceMouse report(product,bytes.data(),count);
    bool result=Update3dMouse(report);
    std::cout<<index<<' '<<step<<' '<<product<<' '<<count;
    for(unsigned char byte:bytes) std::cout<<' '<<unsigned(byte);
    std::cout<<' '<<result<<' '<<report.RawValueRange()<<' '<<Keys().Modifiers();
    for(bool pressed:my3dMouseButtonState) std::cout<<' '<<pressed;
    for(unsigned key=0;key<=Aspect_VKey_Upper;++key) PrintKey(key,time+.125);
    std::cout<<'\n';
  }
  void PrintKey(unsigned key,double time) {
    double duration=0,pressure=-1;
    bool down=Keys().IsKeyDown(key),free=Keys().IsFreeKey(key);
    bool held=ChangeKeys().HoldDuration(key,time,duration,pressure);
    std::cout<<' '<<down<<' '<<free<<' '<<held<<' '<<duration<<' '<<pressure;
  }
};
static std::array<unsigned char,13> AxisReport(int type,int first,int second,int third) {
  std::array<unsigned char,13> bytes{};bytes[0]=type;
  const int values[]={first,second,third,-third,-first,second};
  for(int axis=0;axis<6;++axis) {bytes[1+axis*2]=values[axis]&255;bytes[2+axis*2]=(unsigned(values[axis])>>8)&255;}
  return bytes;
}
static std::array<unsigned char,13> ButtonReport(unsigned state) {
  std::array<unsigned char,13> bytes{};bytes[0]=3;
  for(int byte=0;byte<4;++byte) bytes[byte+1]=(state>>(byte*8))&255;
  return bytes;
}
static void RunSpaceCase(int index) {
  SpaceMouseProbe c;c.Configure(index);const unsigned long products[]={0xC626,0xC627,0xC628,0xC62B,0xC62D,0x1234};
  const unsigned long product=products[index%6];
  c.Action(index,0,product,AxisReport(1,350,-175,1000),7);
  c.Action(index,1,product,AxisReport(1,-1000,30,0),13);
  c.Action(index,2,product,AxisReport(2,-350,175,0),7);
  c.Action(index,3,product,AxisReport(1,0,0,0),13);
  c.Action(index,4,product,AxisReport(2,0,0,0),7);
  c.Action(index,5,product,ButtonReport(0xFFFFFFFF),5);
  c.Action(index,6,product,ButtonReport(0xFFFFFFFF),5);
  c.Action(index,7,product,ButtonReport(0),5);
  c.Action(index,8,product,ButtonReport(2),5);
  c.Action(index,9,product,ButtonReport(0x80000000),5);
  c.Action(index,10,product,ButtonReport(0),5);
  c.Action(index,11,product,AxisReport(9,350,175,0),7);
}
int main() {
  std::cout<<std::setprecision(17);
  for(int index=0;index<24;++index) RunSpaceCase(index);
}
