// Native load/status transitions; the fixture only exposes protected upstream
// entry points.
class V3LoadContext : public V3ContextFixture {
public:
  using AIS_InteractiveContext::ClearGlobal;
  using AIS_InteractiveContext::EraseGlobal;
  void seedDetection(const occ::handle<SelectMgr_ViewerSelector> &selector,
                     int current, int highlighted) {
    mgrSelector = new SelectMgr_SelectionManager(selector);
    myDetectedSeq.Clear();
    for (int rank : {1, 2, 1, 3, 99, 2})
      myDetectedSeq.Append(rank);
    myCurDetected = current;
    myCurHighlighted = highlighted;
  }
  void detectionState() {
    ga_int(myCurDetected);
    ga_int(myCurHighlighted);
    ga_int(myDetectedSeq.Size());
    for (int rank : myDetectedSeq)
      ga_int(rank);
    ga_flag(myLastPicked.IsNull());
  }
  using AIS_InteractiveContext::ClearGlobalPrs;
  using AIS_InteractiveContext::GetDefModes;
  using AIS_InteractiveContext::unselectOwners;
  void picked(const occ::handle<SelectMgr_EntityOwner> &owner) {
    myLastPicked = owner;
  }
  using AIS_InteractiveContext::setObjectStatus;
  using V3ContextFixture::V3ContextFixture;
  occ::handle<AIS_GlobalStatus>
  recorded(const occ::handle<AIS_InteractiveObject> &object) {
    auto found = myObjects.Seek(object);
    return found == nullptr ? occ::handle<AIS_GlobalStatus>() : *found;
  }
};
class V3LoadObject : public V3ContextObject {
public:
  int accepted = -999;
  int expectedKindReferences = -999;
  AIS_KindOfInteractive Type() const override {
    if (expectedKindReferences != -999)
      const_cast<V3LoadObject *>(this)->observe(8000 + GetRefCount() -
                                                expectedKindReferences);
    return AIS_InteractiveObject::Type();
  }
  bool AcceptDisplayMode(int mode) const override {
    return accepted == -999 || accepted == mode;
  }
  void own(bool enabled) { myHasOwnPresentations = enabled; }
  void globalMode(int mode) { setGlobalSelMode(mode); }
};
class V3LoadScene {
public:
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new SvDriver());
  occ::handle<V3LoadContext> ctx = new V3LoadContext(viewer);
  occ::handle<V3LoadContext> foreign = new V3LoadContext(viewer);
  std::vector<occ::handle<V3LoadObject>> objects = {
      new V3LoadObject(), new V3LoadObject(), new V3LoadObject(), nullptr};
  std::vector<occ::handle<V3d_View>> views = {viewer->CreateView(),
                                              viewer->CreateView()};
  void operate(int op, int index, int value, int other) {
    auto object = objects[index];
    switch (op) {
    case 0:
      ctx->Load(object, value);
      break;
    case 1:
      ctx->Load(object, value, other != 0);
      break;
    case 2:
      ctx->setObjectStatus(object, PrsMgr_DisplayStatus(value), other,
                           other + 1);
      break;
    case 3:
      object->status(value);
      break;
    case 4:
      ctx->SetViewAffinity(object, views[value], other != 0);
      break;
    case 5:
      object->SetContext(value ? foreign
                               : occ::handle<AIS_InteractiveContext>());
      break;
    case 6: {
      int d = 17, h = 18, s = 19;
      ctx->GetDefModes(object, d, h, s);
      ga_int(d);
      ga_int(h);
      ga_int(s);
      break;
    }
    case 7:
      object->Attributes()->SetDisplayMode(value);
      break;
    case 8:
      object->SetHilightMode(value);
      break;
    case 9:
      ctx->DefaultDrawer()->SetDisplayMode(value);
      break;
    case 10:
      object->accepted = value;
      break;
    case 11:
      object->own(value != 0);
      break;
    case 12:
      object->globalMode(value);
      break;
    case 13:
      object->AddChild(objects[value]);
      object->SetPropagateVisualState(other != 0);
      break;
    case 14:
      object->ViewAffinity()->SetVisible(value != 0);
      break;
    }
  }
  void state(const occ::handle<V3LoadObject> &object) {
    ga_int(ctx->DisplayStatus(object));
    ga_flag(ctx->IsDisplayed(object));
    for (int mode : {-2, 0, 2, 4, 7})
      ga_flag(ctx->IsDisplayed(object, mode));
    if (object.IsNull())
      return;
    ga_int(object->DisplayStatus());
    ga_flag(object->GetContext() == ctx);
    ga_flag(object->GetContext() == foreign);
    auto status = ctx->recorded(object);
    ga_flag(!status.IsNull());
    if (!status.IsNull()) {
      ga_int(status->DisplayMode());
      ga_int(status->SelectionModes().Size());
      for (int mode : status->SelectionModes())
        ga_int(mode);
    }
    for (auto view : views)
      ga_flag(
          object->ViewAffinity()->IsVisible(view->View()->Identification()));
    ga_flag(viewer->StructureManager()->ObjectAffinity(object) ==
            object->ViewAffinity());
    ga_flag(ctx->SelectionManager()->Contains(object));
    ga_flag(ctx->MainSelector()->Contains(object));
    ga_int(object->Selections().Size());
    ga_int(object->Presentations().Size());
    ga_int(object->events.size());
    for (int event : object->events)
      ga_int(event);
    object->events.clear();
  }
};
void v3_ais_load(std::istream &input) {
  GaReader r{input};
  V3LoadScene scene;
  for (int count = r.integer(); count > 0; --count) {
    int op = r.integer(), index = r.integer(), value = r.integer(),
        other = r.integer();
    GA_TRY(scene.operate(op, index, value, other));
    for (auto object : scene.objects)
      scene.state(object);
  }
  ga_end();
}
