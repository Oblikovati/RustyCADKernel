// Native location transitions, including empty identity and absolute parent placement.
class V3LocationOwner : public SelectMgr_EntityOwner {
public:
  std::vector<int> modes;
  V3LocationOwner(const occ::handle<SelectMgr_SelectableObject>& object)
    : SelectMgr_EntityOwner(object, 0) {}
  void UpdateHighlightTrsf(const occ::handle<V3d_Viewer>&,
                          const occ::handle<PrsMgr_PresentationManager>&, int mode) override {
    modes.push_back(mode);
  }
};
void v3_context_location(std::istream& input) {
  GaReader reader{input}; int flags=reader.integer(), op=reader.integer();
  double initial=reader.real(), final=reader.real();
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());
  occ::handle<V3LoadContext> context=new V3LoadContext(viewer);
  occ::handle<V3ContextObject> object=new V3ContextObject(), parent=new V3ContextObject();
  gp_Trsf transform; transform.SetTranslation(gp_Vec(initial,0,0));
  if (initial!=0 || (flags&4)) object->SetLocalTransformation(transform);
  if (flags&2) {
    gp_Trsf placement; placement.SetTranslation(gp_Vec(10,0,0));
    parent->SetLocalTransformation(placement); parent->AddChild(object);
  }
  if (flags&1) context->Display(object,false);
  if (flags&16) context->Erase(object,false);
  occ::handle<V3LocationOwner> owner=new V3LocationOwner(object);
  if (flags&8) { object->SetHilightMode(7); context->picked(owner); }
  object->events.clear();
  transform.SetTranslation(gp_Vec(final,0,0));
  if (op==0) context->SetLocation(object,TopLoc_Location());
  if (op==1) context->SetLocation(object,TopLoc_Location(gp_Trsf()));
  if (op==2) context->SetLocation(object,TopLoc_Location(transform));
  if (op==3) context->ResetLocation(object);
  ga_flag(context->HasLocation(object)); ga_flag(context->Location(object).IsIdentity());
  ga_flag(object->LocalTransformationGeom().IsNull());
  ga_xyz(object->LocalTransformation().TranslationPart());
  ga_xyz(context->Location(object).Transformation().TranslationPart());
  ga_int(owner->modes.size()); for(int mode:owner->modes) ga_int(mode);
  ga_int(object->Selections().Size());
  for (const auto& selection:object->Selections()) {
    ga_int(selection->UpdateStatus()); ga_int(selection->BVHUpdateStatus());
  }
  ga_int(object->events.size()); for(int event:object->events) ga_int(event); ga_end();
}
