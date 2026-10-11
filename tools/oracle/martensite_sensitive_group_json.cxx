// Martensite remaining Group JSON behavior at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#define MARTENSITE_GROUP_EMBEDDED
#include "martensite_sensitive_group_selection.cxx"
#undef MARTENSITE_GROUP_EMBEDDED
#include "martensite_group_json_addresses.hxx"
static void GroupJsonScenario(int kind,int depth) {
  auto owner=new SelectMgr_EntityOwner(13);auto children=GroupChildren(kind==4?1:kind,owner);
  occ::handle<Select3D_SensitiveGroup> group=kind==0?new Select3D_SensitiveGroup(owner):new Select3D_SensitiveGroup(owner,children);
  if(kind==4)group->Remove(children.First());
  std::ostringstream text;int failure=0;try {group->DumpJson(text,depth);}catch(const Standard_Failure&) {failure=1;}
  std::cout<<kind<<' '<<depth<<' '<<failure<<' '<<GroupJsonPointers(text.str())<<'\n';
}
struct GroupDumpContext {
  int stage=0;std::vector<int> trace;occ::handle<Select3D_SensitiveGroup> parent;
};
class GroupDumpChild : public Select3D_SensitiveSphere {
  GroupDumpContext& context;int identity;bool invalidBounds;
public:
  GroupDumpChild(const occ::handle<SelectMgr_EntityOwner>& owner,GroupDumpContext& ctx,int id,bool invalid)
    :Select3D_SensitiveSphere(owner,gp_Pnt(id,0,0),.25),context(ctx),identity(id),invalidBounds(invalid) {
    // Valid virtual children may return empty boxes and mutate their containing group's policies.
  }
  Select3D_BndBox3d BoundingBox() override {
    context.trace.push_back(identity*10+1);
    if(context.stage==4&&identity==2)throw Standard_Failure("group dump bounds callback 2: expected successful child bounds");
    if(invalidBounds&&context.stage==0)return Select3D_BndBox3d();
    return Select3D_SensitiveSphere::BoundingBox();
  }
  void DumpJson(Standard_OStream& output,int) const override {
    context.trace.push_back(identity*10+2);output<<"\"DumpStage\": "<<identity;
    if(identity==2&&context.stage==1)throw Standard_Failure("group dump child callback 2: expected successful diagnostic output");
    if(identity==2&&context.stage==2)throw std::ios_base::failure("group dump stream callback 2: expected successful diagnostic output");
    if(identity==2&&context.stage==3) {context.parent->SetMatchType(false);context.parent->SetCheckOverlapAll(true);}
  }
};
static void GroupJsonFailure(int stage,int depth) {
  auto owner=new SelectMgr_EntityOwner(13);GroupDumpContext context;context.parent=new Select3D_SensitiveGroup(owner,true);
  for(int identity=1;identity<=3;++identity)context.parent->Add(new GroupDumpChild(owner,context,identity,stage==4));
  context.trace.clear();context.stage=stage;std::ostringstream output;int failure=0;
  try {context.parent->DumpJson(output,depth);}catch(const Standard_Failure&) {failure=1;}catch(const std::ios_base::failure&) {failure=2;}
  std::cout<<stage+5<<' '<<depth<<' '<<context.trace.size();for(int event:context.trace)std::cout<<' '<<event;std::cout<<'\n';
  std::cout<<stage+5<<' '<<depth<<' '<<failure<<' '<<GroupJsonPointers(output.str())<<'\n';
  context.parent.Nullify();
}
#ifndef MARTENSITE_GROUP_JSON_EMBEDDED
int main() {
  // Negative/default depth, truncation, NaN centers and failures retain native diagnostic fragments.
  for(int kind=0;kind<=4;++kind)for(int depth:{-2,-1,0,1,2,3})GroupJsonScenario(kind,depth);
  for(int stage=0;stage<=4;++stage)for(int depth:{-2,-1,0,1,2,3})GroupJsonFailure(stage,depth);
  return 0;
}
#endif
