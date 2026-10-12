// Same-length protected role-action Array1 rebasing preserves genuine const Handle& arguments.
#include <Aspect_OpenVRSession.hxx>
#include <Aspect_XRAction.hxx>
#include <iostream>
class RebasedActionSession : public Aspect_OpenVRSession {
public:
  auto& actions() {
    // The real protected role array retains its element storage across rebasing.
    return myRoleActions[Aspect_XRTrackedDeviceRole_LeftHand];
  }
};
static void emit_action_rebase(int phase,RebasedActionSession& session,
    const occ::handle<Aspect_XRAction>& alias,const occ::handle<Aspect_XRAction>& first,
    const occ::handle<Aspect_XRAction>& second,int originalLower) {
  const auto& actions=session.actions();const int offset=Aspect_XRGenericAction_OutputHaptic-originalLower;
  std::cout<<phase<<' '<<actions.Lower()<<' '<<actions.Upper()<<' '<<(&alias==&actions[actions.Lower()+offset])<<' '
    <<(alias==first)<<' '<<(alias==second)<<' '<<first->GetRefCount()<<' '<<second->GetRefCount()<<' '
    <<session.GenericAction(Aspect_XRTrackedDeviceRole_LeftHand,Aspect_XRGenericAction_OutputHaptic).IsNull()<<'\n';
}
int main() {
  RebasedActionSession session;int lower=session.actions().Lower(),upper=session.actions().Upper();
  occ::handle<Aspect_XRAction> first=new Aspect_XRAction("/first",Aspect_XRActionType_OutputHaptic);
  occ::handle<Aspect_XRAction> second=new Aspect_XRAction("/second",Aspect_XRActionType_OutputHaptic);
  session.actions()[Aspect_XRGenericAction_OutputHaptic]=first;
  const auto& alias=session.GenericAction(Aspect_XRTrackedDeviceRole_LeftHand,Aspect_XRGenericAction_OutputHaptic);
  emit_action_rebase(0,session,alias,first,second,lower);
  session.actions().Resize(lower+9,upper+9,false);emit_action_rebase(1,session,alias,first,second,lower);
  session.actions()[Aspect_XRGenericAction_OutputHaptic+9]=second;emit_action_rebase(2,session,alias,first,second,lower);
  session.actions().Resize(lower,upper,false);emit_action_rebase(3,session,alias,first,second,lower);
  return 0;
}
