// Real context/object calls expose update order and state retained after
// callback failures.
#include <AIS_InteractiveObject.hxx>
#include <Prs3d_ArrowAspect.hxx>
#include <Prs3d_TextAspect.hxx>
#include <PrsMgr_Presentation.hxx>
class V3ContextObject : public AIS_InteractiveObject {
public:
  std::vector<int> events;
  int failMode = -999;
  void status(int value) { SetDisplayStatus(PrsMgr_DisplayStatus(value)); }
  void ClearSelected() override {
    observe(400);
    SelectMgr_SelectableObject::ClearSelected();
  }
  void ClearDynamicHighlight(
      const occ::handle<PrsMgr_PresentationManager> &manager) override {
    observe(401);
    SelectMgr_SelectableObject::ClearDynamicHighlight(manager);
  }
  void RecomputeTransformation(const occ::handle<Graphic3d_Camera> &) override {
    observe(300);
  }

protected:
  void observe(int event) {
    events.push_back(event);
    if (HasInteractiveContext())
      (void)GetContext()->MainPrsMgr();
  }
  void Compute(const occ::handle<PrsMgr_PresentationManager> &,
               const occ::handle<Prs3d_Presentation> &prs, int mode) override {
    observe(100 + mode);
    if (failMode == mode)
      throw Standard_Failure("presentation fixture failure");
    if (!prs.IsNull())
      prs->NewGroup();
  }
  void ComputeSelection(const occ::handle<SelectMgr_Selection> &selection,
                        int mode) override {
    observe(200 + mode);
    if (failMode == mode + 10)
      throw Standard_Failure("selection fixture failure");
    selection->Add(new Select3D_SensitivePoint(
        new SelectMgr_EntityOwner(this, mode), gp_Pnt()));
  }
};
class V3ContextFixture : public AIS_InteractiveContext {
public:
  V3ContextFixture(const occ::handle<V3d_Viewer> &viewer)
      : AIS_InteractiveContext(viewer) {
    // Native 3d097a0 leaves this raw pointer uninitialized; never read an
    // indeterminate value.
    myLastActiveView = nullptr;
  }
  void registerObject(const occ::handle<AIS_InteractiveObject> &object,
                      int mode) {
    if (!myObjects.IsBound(object))
      myObjects.Bind(object, new AIS_GlobalStatus());
    myObjects(object)->AddSelectionMode(mode);
  }
  void recordedMode(const occ::handle<AIS_InteractiveObject> &object,
                    int mode) {
    if (myObjects.IsBound(object))
      myObjects(object)->RemoveSelectionMode(mode);
  }
  void remember(const occ::handle<V3d_View> &view) {
    myLastActiveView = view.get();
  }
};
static void v3_object_aspect(const occ::handle<V3ContextObject> &object,
                             int kind) {
  occ::handle<Prs3d_BasicAspect> aspect;
  occ::handle<Graphic3d_Aspects> graphic;
  switch (kind) {
  case 1: {
    occ::handle<Prs3d_ShadingAspect> a = new Prs3d_ShadingAspect();
    aspect = a;
    graphic = a->Aspect();
    break;
  }
  case 2: {
    occ::handle<Prs3d_LineAspect> a =
        new Prs3d_LineAspect(Quantity_NOC_RED, Aspect_TOL_SOLID, 3.);
    aspect = a;
    graphic = a->Aspect();
    break;
  }
  case 3: {
    occ::handle<Prs3d_PointAspect> a =
        new Prs3d_PointAspect(Aspect_TOM_PLUS, Quantity_NOC_RED, 2.);
    aspect = a;
    graphic = a->Aspect();
    break;
  }
  case 4: {
    occ::handle<Prs3d_TextAspect> a = new Prs3d_TextAspect();
    aspect = a;
    graphic = a->Aspect();
    break;
  }
  case 5:
    aspect = new Prs3d_ArrowAspect();
    break;
  }
  auto prs = object->Presentation();
  std::vector<occ::handle<Graphic3d_Aspects>> before;
  if (!prs.IsNull())
    for (auto group : prs->Groups())
      before.push_back(group->Aspects());
  object->SetAspect(aspect);
  ga_int(before.size());
  if (prs.IsNull())
    return;
  int i = 0;
  for (auto group : prs->Groups()) {
    ga_flag(group->Aspects() == before[i++]);
    ga_flag(group->Aspects() == graphic);
  }
}
static void v3_object_dump(const occ::handle<V3ContextObject> &object,
                           int depth) {
  std::ostringstream stream;
  object->DumpJson(stream, depth);
  ga_text(v3_named_dump_pointers(stream.str(), {"CTXPtr", "Owner"}));
}
class V3ContextScene {
public:
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new SvDriver());
  occ::handle<V3ContextFixture> ctx = new V3ContextFixture(viewer);
  occ::handle<V3ContextObject> object = new V3ContextObject();
  occ::handle<Prs3d_Drawer> original = ctx->DefaultDrawer(),
                            extra = new Prs3d_Drawer();
  occ::handle<Standard_Transient> owner = new Standard_Transient();
  occ::handle<V3d_View> view;
  V3ContextScene(int active) {
    if (!active)
      return;
    view = viewer->CreateView();
    occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow();
    window->SetSize(80, 60);
    view->SetWindow(window);
  }
  void operation(int op, int value, int other) {
    switch (op) {
    case 0:
      object->SetContext(value ? ctx : occ::handle<AIS_InteractiveContext>());
      break;
    case 1:
      ctx->registerObject(object, value);
      break;
    case 2:
      object->status(value);
      break;
    case 3: {
      auto prs = ctx->MainPrsMgr()->Presentation(object, value, true);
      if (other)
        prs->Display();
      break;
    }
    case 4:
      ctx->SelectionManager()->Load(object, value);
      if (other)
        ctx->SelectionManager()->Activate(object, value);
      break;
    case 5:
      object->Redisplay(value != 0);
      break;
    case 6:
      ctx->RecomputePrsOnly(object, value != 0, other != 0);
      break;
    case 7:
      ctx->RecomputeSelectionOnly(object);
      break;
    case 8:
      ctx->Update(object, value != 0);
      break;
    case 9:
      object->failMode = value;
      break;
    case 10:
      object->Attributes()->SetDisplayMode(value);
      break;
    case 11:
      if (value)
        object->SetOwner(owner);
      else
        object->ClearOwner();
      break;
    case 12:
      object->Attributes()->Link(extra);
      break;
    case 13:
      ctx->SetDefaultDrawer(value ? extra : original);
      break;
    case 14:
      ctx->recordedMode(object, value);
      break;
    case 15:
      object->SetToUpdate();
      break;
    case 16:
      ctx->Redisplay(object, value != 0, other != 0);
      break;
    case 17:
      ctx->remember(value ? view : occ::handle<V3d_View>());
      break;
    case 19:
      v3_object_dump(object, value);
      break;
    case 20:
      v3_object_aspect(object, value);
      break;
    case 21:
      if (auto prs = object->Presentation()) {
        if (value)
          prs->NewGroup();
        else
          prs->Clear();
      }
      break;
    case 22:
      if (!view.IsNull()) {
        if (value)
          viewer->SetViewOn(view);
        else
          viewer->SetViewOff(view);
      }
      break;
    case 18:
      ctx->Redisplay(nullptr, value != 0, other != 0);
      ctx->Update(nullptr, true);
      ctx->RecomputeSelectionOnly(nullptr);
      break;
    }
  }
  void state() {
    ga_flag(object->HasInteractiveContext());
    ga_flag(object->InteractiveContext() == ctx.get());
    ga_flag(object->GetContext() == ctx);
    ga_flag(object->HasOwner());
    ga_flag(object->GetOwner() == owner);
    ga_int(object->DisplayStatus());
    ga_flag(object->Attributes()->Link() == original);
    ga_flag(object->Attributes()->Link() == extra);
    ga_flag(object->HasPresentation());
    ga_flag(!object->Presentation().IsNull());
    ga_flag(!ctx->LastActiveView().IsNull());
    NCollection_List<int> modes;
    modes.Append(-5);
    ctx->ActivatedModes(object, modes);
    ga_int(modes.Size());
    for (int mode : modes)
      ga_int(mode);
    for (int mode = 0; mode < 3; ++mode) {
      auto prs = ctx->MainPrsMgr()->Presentation(object, mode, false);
      ga_flag(!prs.IsNull());
      if (!prs.IsNull()) {
        ga_flag(prs->MustBeUpdated());
        ga_flag(prs->IsDisplayed());
        ga_int(prs->Groups().Size());
      }
    }
    ga_flag(ctx->SelectionManager()->Contains(object));
    ga_int(object->Selections().Size());
    for (auto selection : object->Selections()) {
      ga_int(selection->Mode());
      ga_int(selection->UpdateStatus());
      ga_int(selection->BVHUpdateStatus());
      ga_int(selection->GetSelectionState());
      ga_int(selection->Entities().Size());
      ga_flag(ctx->SelectionManager()->IsActivated(object, selection->Mode()));
    }
    ga_int(object->events.size());
    for (int event : object->events)
      ga_int(event);
    object->events.clear();
  }
};
void v3_ais_object(std::istream &input) {
  GaReader r{input};
  V3ContextScene scene(r.integer());
  ga_int(scene.object->Type());
  ga_int(scene.object->Signature());
  for (int action = 0; action < 5; ++action)
    ga_flag(scene.object->ProcessDragging(
        scene.ctx, nullptr, nullptr, NCollection_Vec2<int>(0, 0),
        NCollection_Vec2<int>(1, 2), AIS_DragAction(action)));
  scene.state();
  for (int count = r.integer(); count > 0; --count) {
    int op = r.integer(), value = r.integer(), other = r.integer();
    GA_TRY(scene.operation(op, value, other)) scene.state();
  }
  ga_end();
}
class V3ObservedSelection : public AIS_Selection {
public:
  int clears = 0;
  bool failClear = false;
  void Clear() override {
    ++clears;
    if (failClear)
      throw Standard_Failure("fixture Clear failure");
  }
};
void v3_ais_context_shutdown(std::istream &input) {
  GaReader r{input};
  int registered = r.integer(), selected = r.integer(), custom = r.integer();
  occ::handle<V3ContextFixture> ctx =
      new V3ContextFixture(new V3d_Viewer(new SvDriver()));
  occ::handle<V3ContextObject> object = new V3ContextObject();
  object->SetContext(ctx);
  if (registered)
    ctx->registerObject(object, 1);
  ctx->SelectionManager()->Load(object, 1);
  auto selection = object->Selection(1);
  selection->UpdateBVHStatus(SelectMgr_TBU_None);
  occ::handle<SelectMgr_EntityOwner> owner =
      new SelectMgr_EntityOwner(object, 0);
  auto original = ctx->Selection();
  if (selected)
    original->AddSelect(owner);
  occ::handle<V3ObservedSelection> replacement = new V3ObservedSelection();
  replacement->failClear = custom == 2;
  if (custom)
    ctx->SetSelection(replacement);
  ga_flag(ctx->Selection() == replacement);
  ga_int(replacement->clears);
  ctx.Nullify();
  ga_flag(owner->IsSelected());
  ga_int(original->Extent());
  ga_int(replacement->clears);
  ga_flag(object->HasInteractiveContext());
  ga_int(selection->BVHUpdateStatus());
  for (auto kind :
       {AIS_KindOfInteractive_None, AIS_KindOfInteractive_Datum,
        AIS_KindOfInteractive_Shape, AIS_KindOfInteractive_Object,
        AIS_KindOfInteractive_Relation, AIS_KindOfInteractive_Dimension,
        AIS_KindOfInteractive_LightSource, AIS_KOI_None, AIS_KOI_Datum,
        AIS_KOI_Shape, AIS_KOI_Object, AIS_KOI_Relation, AIS_KOI_Dimension})
    ga_int(kind);
  for (auto action :
       {AIS_DragAction_Start, AIS_DragAction_Confirmed, AIS_DragAction_Update,
        AIS_DragAction_Stop, AIS_DragAction_Abort})
    ga_int(action);
  ga_end();
}
