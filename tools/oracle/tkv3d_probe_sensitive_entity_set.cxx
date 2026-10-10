// Native set mutation traces; preserve shared tree and selection lifetimes.
#include <SelectMgr_SensitiveEntitySet.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <BVH_BinnedBuilder.hxx>
#include <BVH_LinearBuilder.hxx>
#include <BVH_Properties.hxx>
#include <Graphic3d_TransformPers.hxx>
#include <gp_Ax2.hxx>
class V3EntitySet : public SelectMgr_SensitiveEntitySet {
public:using SelectMgr_SensitiveEntitySet::SelectMgr_SensitiveEntitySet;
  using SelectMgr_SensitiveEntitySet::addOwner;using SelectMgr_SensitiveEntitySet::removeOwner;
};
class V3RejectedSensitive : public Select3D_SensitivePoint {
public:using Select3D_SensitivePoint::Select3D_SensitivePoint;
  const occ::handle<Standard_Type>& DynamicType() const override{return Standard_Transient::get_type_descriptor();}
};
occ::handle<Select3D_BVHBuilder3d> v3_entity_set_builder(int kind) {
  if(kind==1)return new BVH_BinnedBuilder<double,3,32>(2,32,false);
  if(kind==2)return new BVH_LinearBuilder<double,3>(2,32);return nullptr;
}
void v3_entity_set_flags(const occ::handle<SelectMgr_SensitiveEntity>& entity,int flags) {
  occ::handle<Graphic3d_TransformPers> persistence;if(flags&1)persistence=new Graphic3d_TransformPers(Graphic3d_TMF_ZoomPers,gp::Origin());
  entity->BaseSensitive()->SetTransformPersistence(persistence);entity->BaseSensitive()->SetFlippingOptions((flags&2)!=0,gp_Ax2());
}
void v3_entity_set_box(const BVH_Box<double,3>& box) {
  ga_flag(box.IsValid());for(const auto& p:{box.CornerMin(),box.CornerMax()}){ga_real(p.x());ga_real(p.y());ga_real(p.z());}
}
void v3_entity_set_tree(const BVH_Tree<double,3>& tree) {
  ga_int(tree.Length());ga_int(tree.Depth());
  for(int i=0;i<tree.Length();++i){auto n=tree.NodeInfoBuffer().Value(i);ga_int(n.x());ga_int(n.y());ga_int(n.z());ga_int(n.w());
    for(const auto& p:{tree.MinPoint(i),tree.MaxPoint(i)}){ga_real(p.x());ga_real(p.y());ga_real(p.z());}}
}
struct V3EntitySetFixture {
  occ::handle<V3EntitySet> set;
  std::vector<occ::handle<SelectMgr_SensitiveEntity>> entities;
  std::vector<occ::handle<SelectMgr_EntityOwner>> owners;
  occ::handle<SelectMgr_Selection> selection;
  occ::handle<BVH_Tree<double,3>> retained;
  V3EntitySetFixture(int count,int builder,bool rejected,int flags):set(new V3EntitySet(v3_entity_set_builder(builder))),
    owners{new SelectMgr_EntityOwner(1),new SelectMgr_EntityOwner(2),nullptr},selection(new SelectMgr_Selection(0)),retained(set->BVH()) {
    for(int i=0;i<count;++i){gp_Pnt point((i*17)%13-6,i%3,i%5);occ::handle<Select3D_SensitiveEntity> base;
      if(rejected&&i==count-1)base=new V3RejectedSensitive(owners[i%3],point);else base=new Select3D_SensitivePoint(owners[i%3],point);
      occ::handle<SelectMgr_SensitiveEntity> entity=new SelectMgr_SensitiveEntity(base);entity->SetActiveForSelection();v3_entity_set_flags(entity,(i+flags)%4);entities.push_back(entity);}
  }
  void choose(int first,int count){selection->ChangeEntities().Clear();for(int i=0;i<count;++i)selection->ChangeEntities().Append(entities[(first+i)%entities.size()]);}
  void build(){bool defined=!set->Builder().IsNull()||!set->IsDirty();ga_flag(defined);if(defined)ga_flag(retained==set->BVH());}
  void mutate(int op,int a,int b) {
    if(op==0)set->Append(entities[a]);if(op==1||op==2){choose(a,b);if(op==1)set->Append(selection);else set->Remove(selection);}
    if(op==3)v3_entity_set_flags(entities[a],b);if(op==4)set->Swap(a,b);if(op==5)set->MarkDirty();if(op==6)build();if(op==7)set->SetBuilder(v3_entity_set_builder(a));
    if(op==8)set->addOwner(owners[a]);if(op==9)set->removeOwner(owners[a]);if(op==10)ga_real(set->Center(a,b));if(op==11)v3_entity_set_box(set->Box(a));
    if(op==12)v3_entity_set_box(set->Box());if(op==13)entities[a]->BaseSensitive()->Set(owners[b]);
    if(op==14){occ::handle<BVH_Properties> p;if(a)p=new BVH_Transform<double,4>;set->SetProperties(p);}
  }
  void state(){ga_int(set->Size());ga_flag(set->IsDirty());ga_flag(set->HasEntityWithPersistence());ga_flag(set->HasEntityWithFlipping());ga_flag(!set->Properties().IsNull());
    for(int i=1;i<=set->Sensitives().Extent();++i){auto e=set->Sensitives().FindKey(i);int id=0;while(entities[id]!=e)++id;ga_int(id);}
    for(auto& owner:owners){const int* count=set->Owners().Seek(owner);ga_int(count?*count:0);}for(auto& entity:entities)ga_flag(entity->IsActiveForSelection());v3_entity_set_tree(*retained);}
};
void v3_sensitive_entity_set_case(std::istream& input) {
  GaReader r{input};int count=r.integer(),builder=r.integer();bool rejected=r.flag();int flags=r.integer();
  GA_TRY(V3EntitySetFixture f(count,builder,rejected,flags);ga_text(f.set->DynamicType()->Name());ga_text(SelectMgr_SensitiveEntitySet::get_type_descriptor()->Parent()->Name());
    ga_flag(f.set->IsKind("BVH_ObjectTransient"));ga_flag(f.set->IsKind("Standard_Transient"));f.state();int steps=r.integer();
    for(int i=0;i<steps;++i){int op=r.integer(),a=r.integer(),b=r.integer();GA_TRY(f.mutate(op,a,b))f.state();}
  )ga_end();
}
