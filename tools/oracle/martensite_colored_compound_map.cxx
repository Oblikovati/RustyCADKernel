// Martensite actual protected IndexedDataMap specialization, borrowed fields and removal ownership.
#include "martensite_colored_compound_map.hxx"
static void retained_compound_aliases()
{
  const occ::handle<Prs3d_Drawer> parent = new Prs3d_Drawer;
  const occ::handle<AIS_ColoredDrawer> drawer = new AIS_ColoredDrawer(parent);
  TopoDS_Compound first,second; BRep_Builder builder;
  builder.MakeCompound(first); builder.MakeCompound(second);
  DrawerCompounds map;
  std::cout << "add " << map.Add(drawer,first) << ' ' << drawer->GetRefCount() << ' ' << first.TShape()->GetRefCount() << '\n';
  const auto& key=map.FindKey(1); const auto& value=map.FindFromIndex(1);
  const auto* keyAddress=&key; const auto* valueAddress=&value;
  std::cout << "duplicate " << map.Add(drawer,second) << ' ' << value.IsSame(first) << '\n';
  map.ReSize(1000);
  std::cout << "rehash " << (&map.FindKey(1)==keyAddress) << ' ' << (&map.FindFromIndex(1)==valueAddress) << '\n';
  map.ChangeFromIndex(1)=second;
  std::cout << "changed " << value.IsSame(second) << ' ' << first.TShape()->GetRefCount() << ' ' << second.TShape()->GetRefCount() << '\n';
  map.Clear(false);
  std::cout << "cleared " << drawer->GetRefCount() << ' ' << second.TShape()->GetRefCount() << ' ' << map.Extent() << '\n';
}
class CompoundKeyDestructorProbe : public AIS_ColoredDrawer
{
public:
  DrawerCompounds* map;
  const TopoDS_Compound* value=nullptr;
  CompoundKeyDestructorProbe(const occ::handle<Prs3d_Drawer>& parent,DrawerCompounds* owner)
    : AIS_ColoredDrawer(parent),map(owner) {}
  ~CompoundKeyDestructorProbe() override
  {
    std::cout << "key-destructor " << map->Extent() << ' ' << value->TShape()->GetRefCount()
      << ' ' << Link()->GetRefCount() << '\n';
  }
};
static void ordered_compound_field_release()
{
  const occ::handle<Prs3d_Drawer> parent = new Prs3d_Drawer;
  TopoDS_Compound compound; BRep_Builder().MakeCompound(compound);
  DrawerCompounds map;
  occ::handle<CompoundKeyDestructorProbe> drawer=new CompoundKeyDestructorProbe(parent,&map);
  occ::handle<AIS_ColoredDrawer> key(drawer.get());
  map.Add(key,compound); drawer->value=&map.FindFromIndex(1);
  drawer.Nullify(); key.Nullify();
  map.Clear(false);
  std::cout << "release-complete " << map.Extent() << ' ' << parent->GetRefCount() << ' ' << compound.TShape()->GetRefCount() << '\n';
}
int main()
{
  retained_compound_aliases();
  ordered_compound_field_release();
}
