// Exercise the real AIS detection algorithms with named observable virtual callbacks.
#include <SelectMgr_Filter.hxx>
// The ordinary probe driver discards invalidation. This fixture retains the same
// invalidation state as Rust's null driver so AIS exercises its full redraw branch.
class V3DetectionView : public SvView {
  bool invalidated=false;
public:
  V3DetectionView(const occ::handle<Graphic3d_StructureManager>& manager,const NCollection_List<occ::handle<Graphic3d_Layer>>& layers):SvView(manager,layers){}
  void Redraw() override {invalidated=false;}
  void Invalidate() override {invalidated=true;}
  bool IsInvalidated() override {return invalidated;}
};
class V3DetectionDriver : public SvDriver {
public:
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& manager) override {return new V3DetectionView(manager,myLayers);}
};
struct V3DetectionTrace {
  std::vector<int> events;bool accept=true,forced=false;int mutation=0,failing=0,mask=7;
};
class V3DetectionObject : public AIS_InteractiveObject {
public:
  V3DetectionTrace& trace;
  V3DetectionObject(V3DetectionTrace& t):trace(t){}
  bool AcceptDisplayMode(int mode) const override {
    trace.events.push_back(100+mode);(void)GetContext()->HasDetected();
    if(trace.mutation){occ::handle<Prs3d_Drawer> style=new Prs3d_Drawer();style->SetDisplayMode(9);
      if(trace.mutation==1)const_cast<V3DetectionObject*>(this)->SetDynamicHilightAttributes(style);
      if(trace.mutation==2)GetContext()->SetHighlightStyle(style);}
    return trace.accept;
  }
  void BoundingBox(Bnd_Box& bounds) override {bounds.Update(-1.,-1.,-3.,1.,1.,1.);}
  void ClearDynamicHighlight(const occ::handle<PrsMgr_PresentationManager>& pm) override {
    trace.events.push_back(300);(void)GetContext()->HasDetected();pm->ClearImmediateDraw();
  }
protected:
  void Compute(const occ::handle<PrsMgr_PresentationManager>&,const occ::handle<Prs3d_Presentation>& prs,int mode) override {trace.events.push_back(400+mode);prs->NewGroup();}
  void ComputeSelection(const occ::handle<SelectMgr_Selection>&,int) override {}
};
class V3DetectionOwner : public SelectMgr_EntityOwner {
public:
  V3DetectionTrace& trace;
  V3DetectionOwner(const occ::handle<SelectMgr_SelectableObject>& object,int id,V3DetectionTrace& t):SelectMgr_EntityOwner(object,id),trace(t){}
  void decomposition(bool value){myFromDecomposition=value;}
  bool IsForcedHilight() const override {return trace.forced;}
  void HilightWithColor(const occ::handle<PrsMgr_PresentationManager>& pm,const occ::handle<Prs3d_Drawer>& style,int mode) override {
    for(int value:{200+Priority(),mode,style->DisplayMode(),style->ZLayer(),int(pm->IsImmediateModeOn())})trace.events.push_back(value);
    if(trace.failing==Priority()+1)throw Standard_Failure("failing owner");
    SelectMgr_EntityOwner::HilightWithColor(pm,style,mode);
  }
};
class V3DetectionFilter : public SelectMgr_Filter {
public:
  V3DetectionTrace& trace;
  V3DetectionFilter(V3DetectionTrace& t):trace(t){}
  bool IsOk(const occ::handle<SelectMgr_EntityOwner>& owner) const override {
    trace.events.push_back(500+owner->Priority());
    if(trace.failing==owner->Priority()+4)throw Standard_Failure("failing filter");
    return (trace.mask&(1<<owner->Priority()))!=0;
  }
};
class V3DetectionSelector : public SelectMgr_ViewerSelector {
public:
  void seed(const std::vector<occ::handle<V3DetectionOwner>>& owners,int mask){
    ClearPicked();for(int id=0;id<3;++id){if(!(mask&(1<<id)))continue;
      SelectMgr_SortCriterion criterion;criterion.Depth=id;mystored.Add(owners[id],criterion);}
    myIsSorted=false;
  }
};
class V3DetectionContext : public AIS_InteractiveContext {
public:
  V3DetectionContext(const occ::handle<V3d_Viewer>& viewer,const occ::handle<V3DetectionSelector>& selector):AIS_InteractiveContext(viewer){
    mgrSelector=new SelectMgr_SelectionManager(selector);myLastActiveView=nullptr;
  }
  using AIS_InteractiveContext::moveTo;
  using AIS_InteractiveContext::getHilightMode;
  using AIS_InteractiveContext::isSlowHiStyle;
  void state(){ga_int(myCurDetected);ga_int(myCurHighlighted);ga_int(myDetectedSeq.Size());for(int rank:myDetectedSeq)ga_int(rank);}
};
static occ::handle<V3d_View> v3_detection_view(const occ::handle<V3d_Viewer>& viewer){
  auto view=viewer->CreateView();occ::handle<Aspect_NeutralWindow> window=new Aspect_NeutralWindow();window->SetSize(80,60);view->SetWindow(window);
  auto camera=view->Camera();camera->SetEye(gp_Pnt(0.,0.,20.));camera->SetCenter(gp_Pnt());camera->SetUp(gp_Dir(0.,1.,0.));camera->SetScale(20.);camera->SetAspect(80./60.);return view;
}
class V3DetectionScene {
public:
  V3DetectionTrace trace;
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DetectionDriver());
  occ::handle<V3DetectionSelector> selector=new V3DetectionSelector();
  occ::handle<V3DetectionContext> ctx=new V3DetectionContext(viewer,selector);
  occ::handle<V3d_View> view=v3_detection_view(viewer);
  occ::handle<V3DetectionObject> object=new V3DetectionObject(trace);
  std::vector<occ::handle<V3DetectionOwner>> owners;
  V3DetectionScene(int flags,int mask,int style,int mutation,int failing){
    trace.mask=mask;trace.mutation=mutation;trace.failing=failing;trace.accept=!(flags&32);trace.forced=(flags&8)!=0;
    object->SetContext(ctx);for(int id=0;id<3;++id)owners.push_back(new V3DetectionOwner(object,id,trace));
    ctx->SetAutomaticHilight(!(flags&2));ctx->SetToHilightSelected(!(flags&4));
    ctx->SetPickingStrategy(flags&16?SelectMgr_PickingStrategy_OnlyTopmost:SelectMgr_PickingStrategy_FirstAcceptable);
    ctx->AddFilter(new V3DetectionFilter(trace));owners[0]->SetSelected((flags&1)!=0);owners[1]->decomposition(true);
    if(flags&64)owners[2]->SetSelectable(nullptr);
    occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer();drawer->SetDisplayMode(style);drawer->SetZLayer(flags&128?Graphic3d_ZLayerId_UNKNOWN:Graphic3d_ZLayerId_Top);
    if(mutation==1)object->SetDynamicHilightAttributes(drawer);else ctx->SetHighlightStyle(drawer);
    ctx->DefaultDrawer()->SetDisplayMode(3);
    if(flags&256){viewer->SetComputedMode(true);view->SetComputedMode(true);}
    if(flags&512)view->Camera()->SetProjectionType(Graphic3d_Camera::Projection_Perspective);
  }
  void geometry(){
    occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
    for(int id=0;id<3;++id)selection->Add(new Select3D_SensitivePoint(owners[id],gp_Pnt(0.,0.,-id)));
    selection->UpdateStatus(SelectMgr_TOU_Partial);selection->UpdateBVHStatus(SelectMgr_TBU_Add);
    object->AddSelection(selection,0);ctx->SelectionManager()->Load(object,0);ctx->SelectionManager()->Activate(object,0);
  }
  void operation(int op,int value){switch(op){
    case 0:selector->seed(owners,value);break;
    case 1:ga_int(ctx->moveTo(view,value!=0));break;
    case 2:ga_int(ctx->ClearDetected(value!=0));break;
    case 3:ctx->InitDetected();break;case 4:ctx->NextDetected();break;
    case 5:ga_int(ctx->HilightNextDetected(view,value!=0));break;
    case 6:ga_int(ctx->HilightPreviousDetected(view,value!=0));break;
    case 7:trace.mask=value;break;case 8:trace.failing=value;break;
    case 9:ga_int(ctx->MoveTo(40+value,30,view,true));break;
    case 10:ga_int(ctx->MoveTo(gp_Ax1(gp_Pnt(value,0.,20.),gp_Dir(0.,0.,-1.)),view,true));break;
    case 11:geometry();break;
    case 13:ga_int(ctx->getHilightMode(object,value < -1 ? occ::handle<Prs3d_Drawer>() : ctx->HighlightStyle(), value < -1 ? -1 : value));break;
    case 14:object->Attributes()->SetDisplayMode(value);break;
    case 15:ga_int(ctx->isSlowHiStyle(owners[value],viewer));break;
    case 12:{occ::handle<V3d_Viewer> other=new V3d_Viewer(new V3DetectionDriver());ga_int(ctx->MoveTo(40,30,v3_detection_view(other),true));break;}
  }}
  int ownerId(const occ::handle<SelectMgr_EntityOwner>& owner){for(int i=0;i<3;++i)if(owner==owners[i])return i;return -1;}
  void state(){
    ctx->state();ga_flag(ctx->HasDetected());ga_flag(ctx->HasNextDetected());ga_flag(ctx->MoreDetected());
    ga_int(ownerId(ctx->DetectedOwner()));ga_int(ownerId(ctx->DetectedCurrentOwner()));
    ga_flag(ctx->HasDetected()&&ctx->DetectedInteractive()==object);ga_flag(ctx->DetectedCurrentObject()==object);
    ga_int(selector->NbPicked());ga_flag(ctx->MainPrsMgr()->IsImmediateModeOn());ga_flag(!ctx->LastActiveView().IsNull());
    ga_flag(view->ComputedMode());ga_flag(view->IsInvalidated());
    ga_int(trace.events.size());for(int event:trace.events)ga_int(event);trace.events.clear();
  }
};
void v3_ais_detection(std::istream& input){
  GaReader r{input};int flags=r.integer(),mask=r.integer(),style=r.integer(),mutation=r.integer(),failing=r.integer();
  V3DetectionScene scene(flags,mask,style,mutation,failing);scene.state();
  for(int count=r.integer();count>0;--count){int op=r.integer(),value=r.integer();GA_TRY(scene.operation(op,value))scene.state();}ga_end();
}
