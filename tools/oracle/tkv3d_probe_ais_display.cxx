// Calls pinned AIS display/erase bodies; fixture subclasses only observe
// virtual callbacks.
static void v3_display_configure(V3LoadScene &scene, int op, int index, int a,
                                 int b) {
  auto object = scene.objects[index];
  switch (op) {
  case 25:
    scene.ctx->SetAutoActivateSelection(a != 0);
    break;
  case 26:
    scene.ctx->SetAutomaticHilight(a != 0);
    break;
  case 27:
    object->SetAutoHilight(a != 0);
    break;
  case 28:
    object->failMode = a;
    break;
  case 29:
    scene.ctx->AddSelect(object);
    break;
  case 30:
    scene.ctx->HilightWithColor(object, scene.ctx->SelectionStyle(), false);
    break;
  case 31:
    scene.ctx->Unhilight(object, false);
    break;
  case 32:
    object->GlobalSelOwner()->SetSelected(a != 0);
    break;
  case 33:
    scene.ctx->picked(object.IsNull() ? occ::handle<SelectMgr_EntityOwner>()
                                      : object->GlobalSelOwner());
    break;
  case 35:
    scene.ctx->SelectionManager()->Load(object, a);
    if (b)
      scene.ctx->SelectionManager()->Activate(object, a);
    break;
  case 36:
    scene.ctx->recordedMode(object, a);
    break;
  case 37: {
    occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow();
    window->SetSize(80, 60);
    scene.views[a]->SetWindow(window);
    break;
  }
  }
}
static void v3_display_operate(V3LoadScene &scene, int op, int index, int a,
                               int b, int c, int d) {
  auto object = scene.objects[index];
  if (op <= 14) {
    scene.operate(op, index, a, b);
    return;
  }
  switch (op) {
  case 20:
    scene.ctx->Display(object, a != 0);
    break;
  case 21:
    scene.ctx->Display(object, a, b, c != 0, PrsMgr_DisplayStatus(d));
    break;
  case 22:
    scene.ctx->Erase(object, a != 0);
    break;
  case 23:
    scene.ctx->EraseGlobal(object, a != 0);
    break;
  case 24:
    scene.ctx->unselectOwners(object);
    break;
  default:
    v3_display_configure(scene, op, index, a, b);
    break;
  }
}
static void
v3_display_presentations(const occ::handle<AIS_InteractiveObject> &object) {
  for (auto prs : object->Presentations()) {
    ga_int(prs->Mode());
    ga_flag(prs->IsDisplayed());
    ga_flag(prs->IsVisible());
    ga_flag(prs->IsHighlighted());
    ga_flag(prs->MustBeUpdated());
    ga_int(prs->Groups().Size());
  }
}
static void
v3_display_selections(V3LoadScene &scene,
                      const occ::handle<AIS_InteractiveObject> &object) {
  for (auto selection : object->Selections()) {
    ga_int(selection->Mode());
    ga_int(selection->GetSelectionState());
    ga_int(selection->UpdateStatus());
    ga_int(selection->Entities().Size());
    ga_flag(
        scene.ctx->SelectionManager()->IsActivated(object, selection->Mode()));
  }
  auto owner = object->GlobalSelOwner();
  ga_flag(!owner.IsNull());
  if (!owner.IsNull())
    ga_flag(owner->IsSelected());
  ga_flag(scene.ctx->IsHilighted(object));
}
static void v3_display_state(V3LoadScene &scene) {
  ga_int(scene.ctx->NbSelected());
  for (auto owner : scene.ctx->Selection()->Objects()) {
    int index = -1;
    for (int i = 0; i < int(scene.objects.size()); ++i)
      if (owner->IsSameSelectable(scene.objects[i])) {
        index = i;
        break;
      }
    ga_int(index);
  }
  for (auto object : scene.objects) {
    scene.state(object);
    if (!object.IsNull()) {
      v3_display_presentations(object);
      v3_display_selections(scene, object);
    }
  }
}
void v3_ais_display(std::istream &input) {
  GaReader r{input};
  V3LoadScene scene;
  for (int count = r.integer(); count > 0; --count) {
    int op = r.integer(), index = r.integer(), a = r.integer(), b = r.integer(),
        c = r.integer(), d = r.integer();
    GA_TRY(v3_display_operate(scene, op, index, a, b, c, d));
    v3_display_state(scene);
  }
  ga_end();
}
