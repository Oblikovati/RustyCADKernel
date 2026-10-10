// The actual native threshold and nonrecursive exact-group rule, without giant geometry.
#include <Select3D_SensitiveGroup.hxx>
class V3ThresholdEntity : public V3PoolJob {
public:
  int elements;
  V3ThresholdEntity(int count,bool failure):V3PoolJob(failure?1:0),elements(count){}
  int NbSubElements() const override{return elements;}
  Select3D_BndBox3d BoundingBox() override{return Select3D_BndBox3d(NCollection_Vec3<double>(0.,0.,0.),NCollection_Vec3<double>(1.,1.,1.));}
};
void v3_prebuild_threshold(std::istream& input){
  GaReader r{input};int grouped=r.integer(),count=r.integer(),failAt=r.integer();
  std::vector<occ::handle<V3ThresholdEntity>> entities;
  occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(0);
  occ::handle<Select3D_SensitiveGroup> group=new Select3D_SensitiveGroup(nullptr,true);
  for(int i=0;i<count;++i){auto entity=occ::handle<V3ThresholdEntity>(new V3ThresholdEntity(r.integer(),i==failAt));entities.push_back(entity);if(grouped)group->Add(entity);else selection->Add(entity);}
  if(grouped==2){occ::handle<Select3D_SensitiveGroup> outer=new Select3D_SensitiveGroup(nullptr,true);outer->Add(group);selection->Add(outer);}
  else if(grouped)selection->Add(group);
  GA_TRY(StdSelect_BRepSelectionTool::PreBuildBVH(selection))
  for(auto& entity:entities)ga_int(entity->calls.load());ga_end();
}
