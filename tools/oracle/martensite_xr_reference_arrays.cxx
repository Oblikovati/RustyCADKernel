// Source-pinned native array-object, transform and generic-action references; no headset/GPU.
#include <Aspect_OpenVRSession.hxx>
#include <iomanip>
#include <iostream>
class XrArrayReferenceProbe : public Aspect_OpenVRSession {
public:
  void resize(int lower,int upper) {
    // The original protected Array1 is retained; only its bounds/storage change.
    myTrackedPoses.Resize(lower,upper,false);
  }
  void pose(int device,double x) {
    myTrackedPoses[device].Orientation.SetTranslation(gp_Vec(x,2,3));
    myTrackedPoses[device].IsValidPose=true;
  }
  void head(double x) {
    myHeadPose.SetTranslation(gp_Vec(x,5,6));
  }
  void shift_actions() {
    myRoleActions[Aspect_XRTrackedDeviceRole_LeftHand].Resize(8,24,false);
  }
};
static void emit_xr_array(int scenario,int frame,XrArrayReferenceProbe& session,const NCollection_Array1<Aspect_TrackedDevicePose>& alias,const gp_Trsf& head) {
  int first=alias.Lower();bool valid=alias.IsEmpty()?false:alias[first].IsValidPose;
  double x=alias.IsEmpty()?0:alias[first].Orientation.TranslationPart().X();
  std::cout<<scenario<<' '<<frame<<' '<<alias.Lower()<<' '<<alias.Upper()<<' '<<alias.Length()<<' '<<alias.IsEmpty()<<' '<<valid<<' '<<x<<' '<<head.TranslationPart().X()<<' '<<(&alias==&session.TrackedPoses())<<' '<<(&head==&session.HeadPose())<<'\n';
}
static void xr_array_case(int scenario,int lower,int upper) {
  XrArrayReferenceProbe session;const auto& alias=session.TrackedPoses();const auto& head=session.HeadPose();
  emit_xr_array(scenario,0,session,alias,head);session.resize(lower,upper);
  if(!alias.IsEmpty())session.pose(lower,7+scenario);
  session.head(11+scenario);emit_xr_array(scenario,1,session,alias,head);
  if(!alias.IsEmpty())session.pose(lower,17+scenario);
  session.head(21+scenario);emit_xr_array(scenario,2,session,alias,head);
}
static void xr_action_bounds_case() {
  XrArrayReferenceProbe session;session.shift_actions();int failed=0;
  try {session.GenericAction(Aspect_XRTrackedDeviceRole_LeftHand,Aspect_XRGenericAction_InputTriggerClick);}
  catch(const Standard_OutOfRange&) {failed=1;}
  std::cout<<4<<' '<<0<<' '<<failed<<' '<<session.GenericAction(Aspect_XRTrackedDeviceRole_LeftHand,Aspect_XRGenericAction_OutputHaptic).IsNull()<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);xr_array_case(0,-3,2);xr_array_case(1,5,5);
  xr_array_case(2,9,8);xr_array_case(3,10,12);xr_action_bounds_case();return 0;
}
