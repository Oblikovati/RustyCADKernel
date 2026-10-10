// Native protected result storage, without replacing any AIS operation.
class V3LifecycleSelector : public SelectMgr_ViewerSelector {
public:
  void seed(V3LoadScene &scene) {
    for (int i = 0; i < 3; ++i)
      mystored.Add(new SelectMgr_EntityOwner(scene.objects[i], 0),
                   SelectMgr_SortCriterion());
    myIndexes.Resize(1, 3, false);
    for (int i = 1; i <= 3; ++i)
      myIndexes(i) = i;
    myIsSorted = true;
  }
};
// Execute pinned native methods; initial list prefixes make accidental clearing
// observable.
static void v3_lifecycle_query(V3LoadScene &scene, int op, int a, int b,
                               int c) {
  NCollection_List<occ::handle<AIS_InteractiveObject>> objects;
  objects.Append(scene.objects[2]);
  const auto kind = AIS_KindOfInteractive(a);
  const auto status = PrsMgr_DisplayStatus(c);
  switch (op) {
  case 50:
    scene.ctx->DisplayedObjects(objects);
    break;
  case 51:
    scene.ctx->ErasedObjects(objects);
    break;
  case 52:
    scene.ctx->DisplayedObjects(kind, b, objects);
    break;
  case 53:
    scene.ctx->ErasedObjects(kind, b, objects);
    break;
  case 54:
    scene.ctx->ObjectsByDisplayStatus(status, objects);
    break;
  case 55:
    scene.ctx->ObjectsByDisplayStatus(kind, b, status, objects);
    break;
  case 56:
    scene.ctx->ObjectsInside(objects, kind, b);
    break;
  case 57:
    scene.ctx->ObjectsForView(objects, scene.views[a], b != 0, status);
    break;
  }
  ga_int(objects.Size());
  ga_int(objects.First() == scene.objects[2]);
  for (auto object : scene.objects) {
    int count = 0;
    for (auto found : objects)
      if (found == object)
        ++count;
    ga_int(count);
  }
}
static void v3_lifecycle_operate(V3LoadScene &scene, int op, int index, int a,
                                 int b, int c, int d) {
  auto object = scene.objects[index];
  switch (op) {
  case 61:
    object->expectedKindReferences = object->GetRefCount();
    break;
  case 58: {
    occ::handle<V3LifecycleSelector> selector = new V3LifecycleSelector();
    selector->seed(scene);
    scene.ctx->seedDetection(selector, a, b);
    break;
  }
  case 59: {
    for (int i = 0; i < 3; ++i)
      scene.ctx->Load(scene.objects[i]);
    NCollection_List<occ::handle<AIS_InteractiveObject>> ordered;
    scene.ctx->ObjectsInside(ordered);
    int i = 0;
    for (auto entry : ordered)
      scene.objects[i++] = occ::down_cast<V3LoadObject>(entry);
    break;
  }
  case 40:
    scene.ctx->EraseAll(a != 0);
    break;
  case 41:
    scene.ctx->DisplayAll(a != 0);
    break;
  case 42:
    scene.ctx->DisplaySelected(a != 0);
    break;
  case 43:
    scene.ctx->EraseSelected(a != 0);
    break;
  case 44:
    scene.ctx->Remove(object, a != 0);
    break;
  case 45:
    scene.ctx->RemoveAll(a != 0);
    break;
  case 46:
    scene.ctx->ClearGlobal(object, a != 0);
    break;
  case 47:
    scene.ctx->ClearGlobalPrs(object, a, b != 0);
    break;
  case 48:
    scene.ctx->ClearPrs(object, a, b != 0);
    break;
  case 49:
    scene.ctx->Display(object, a, b, c != 0, index % 2 == 0,
                       PrsMgr_DisplayStatus(d));
    break;
  case 50:
  case 51:
  case 52:
  case 53:
  case 54:
  case 55:
  case 56:
  case 57:
    v3_lifecycle_query(scene, op, a, b, c);
    break;
  default:
    v3_display_operate(scene, op, index, a, b, c, d);
    break;
  }
}
void v3_ais_lifecycle(std::istream &input) {
  GaReader r{input};
  V3LoadScene scene;
  for (int count = r.integer(); count > 0; --count) {
    int op = r.integer(), index = r.integer(), a = r.integer(), b = r.integer(),
        c = r.integer(), d = r.integer();
    GA_TRY(v3_lifecycle_operate(scene, op, index, a, b, c, d));
    v3_display_state(scene);
    scene.ctx->detectionState();
  }
  ga_end();
}
