// Martensite virtual Group geometry queries in qualified diagnostics and distance at pinned OCCT 3d097a0.
#define MARTENSITE_GROUP_HOOKS_EMBEDDED
#include "martensite_sensitive_group_hooks.cxx"
#undef MARTENSITE_GROUP_HOOKS_EMBEDDED
#include "martensite_group_json_addresses.hxx"
static void GroupSubclassJson(int geometry,int depth) {
  auto owner=new SelectMgr_EntityOwner(13);auto children=GroupChildren(2,owner);occ::handle<GroupHookProbe> group=new GroupHookProbe(owner,true);
  group->Add(children);group->geometry=geometry;std::ostringstream output;group->DumpJson(output,depth);
  const auto text=GroupJsonPointers(output.str());std::cout<<geometry<<' '<<depth<<' '<<text<<'\n';
  auto volume=QuadricVolume(0,0,0,false);std::cout<<geometry<<' '<<depth<<' '<<group->CoreDistance(volume)<<'\n';
}
int main() {
  // Each geometry override reaches qualified base diagnostics without changing the stored cache tail.
  std::cout<<std::setprecision(17);
  for(int geometry=0;geometry<=3;++geometry)for(int depth:{-1,0,1,2,3})GroupSubclassJson(geometry,depth);
  return 0;
}
