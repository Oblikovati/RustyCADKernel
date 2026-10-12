// Native owner callbacks retain genuine context viewer/manager aliases while mutating context fields.
#include "tkservice_probe_null_driver.cxx"
#include <AIS_InteractiveContext.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <Prs3d_Drawer.hxx>
#include <V3d_Viewer.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <stdexcept>
class ViewerManagerContext : public AIS_InteractiveContext {
public:
  ViewerManagerContext(const occ::handle<V3d_Viewer>& viewer):AIS_InteractiveContext(viewer) {}
  void replace_fields(const occ::handle<V3d_Viewer>& viewer,const occ::handle<PrsMgr_PresentationManager>& manager) {
    myMainVwr=viewer;myMainPM=manager;
    SetDefaultDrawer(new Prs3d_Drawer);
  }
};
class ViewerManagerCallbackOwner : public SelectMgr_EntityOwner {
public:
  ViewerManagerContext* context=nullptr;int mode=0;bool armed=true;int records[8]={};
  const occ::handle<V3d_Viewer>* replacement=nullptr;
  const occ::handle<PrsMgr_PresentationManager>* replacementManager=nullptr;
  ViewerManagerCallbackOwner():SelectMgr_EntityOwner(0) {}
  void UpdateHighlightTrsf(const occ::handle<V3d_Viewer>& viewer,
      const occ::handle<PrsMgr_PresentationManager>& manager,const int) override {
    if(!armed)return;
    armed=false;records[0]=&viewer==&context->CurrentViewer();records[1]=&manager==&context->MainPrsMgr();
    records[2]=viewer->GetRefCount();records[3]=manager->GetRefCount();
    context->replace_fields(*replacement,*replacementManager);
    records[4]=viewer==*replacement;records[5]=manager==*replacementManager;
    records[6]=viewer->GetRefCount();records[7]=manager->GetRefCount();
    if(mode==1)throw Standard_Failure("viewer manager callback fault=1; expected success");
    if(mode==2)throw std::logic_error("viewer manager callback unrelated");
  }
};
static int apply_viewer_manager(const occ::handle<ViewerManagerCallbackOwner>& owner,
    const occ::handle<AIS_InteractiveContext>& context) {
  try{owner->UpdateHighlightTrsf(context->CurrentViewer(),context->MainPrsMgr(),0);return 0;}
  catch(const Standard_Failure&){return 1;}catch(const std::logic_error&){return 2;}
}
static void emit_viewer_manager(int mode,int frame,int failure,const ViewerManagerCallbackOwner& owner,
    const occ::handle<AIS_InteractiveContext>& context,const occ::handle<V3d_Viewer>& original,
    const occ::handle<V3d_Viewer>& replacement,const occ::handle<PrsMgr_PresentationManager>& first,
    const occ::handle<PrsMgr_PresentationManager>& second) {
  std::cout<<mode<<' '<<frame<<' '<<failure;
  for(int value:owner.records)std::cout<<' '<<value;
  std::cout<<' '<<(context->CurrentViewer()==replacement)<<' '<<(context->MainPrsMgr()==second)
    <<' '<<original->GetRefCount()<<' '<<replacement->GetRefCount()<<' '<<first->GetRefCount()<<' '<<second->GetRefCount()<<'\n';
}
static void viewer_manager_case(int mode) {
  occ::handle<V3d_Viewer> original=new V3d_Viewer(new SvDriver),replacement=new V3d_Viewer(new SvDriver);
  occ::handle<AIS_InteractiveContext> context=new ViewerManagerContext(original);
  auto first=context->MainPrsMgr();occ::handle<PrsMgr_PresentationManager> second=new PrsMgr_PresentationManager(replacement->StructureManager());
  occ::handle<ViewerManagerCallbackOwner> owner=new ViewerManagerCallbackOwner;
  owner->context=static_cast<ViewerManagerContext*>(context.get());owner->mode=mode;
  owner->replacement=&replacement;owner->replacementManager=&second;
  for(int frame=0;frame<2;++frame) {
    int failure=apply_viewer_manager(owner,context);
    emit_viewer_manager(mode,frame,failure,*owner,context,original,replacement,first,second);
  }
}
int main() {
  for(int mode=0;mode<3;++mode)viewer_manager_case(mode);
  return 0;
}
