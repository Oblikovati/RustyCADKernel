// The retiring native TShape destructor reads the still-old Value slot before incoming retention.
#include "martensite_colored_compound_map.hxx"
#include <TopoDS_TCompound.hxx>
class CompoundValueDestructorProbe:public TopoDS_TCompound
{
public:
  DrawerCompounds* map;
  const TopoDS_Compound* replacement;
  CompoundValueDestructorProbe(DrawerCompounds* owner,const TopoDS_Compound* incoming):map(owner),replacement(incoming){}
  ~CompoundValueDestructorProbe() override
  {
    const auto& value=map->FindFromIndex(1);
    std::cout<<"value-destructor "<<map->Extent()<<' '<<(value.TShape().get()==this)
      <<' '<<value.TShape()->GetRefCount()<<' '<<replacement->TShape()->GetRefCount()<<'\n';
  }
};
int main()
{
  const occ::handle<Prs3d_Drawer> parent=new Prs3d_Drawer;
  const occ::handle<AIS_ColoredDrawer> drawer=new AIS_ColoredDrawer(parent);
  DrawerCompounds map;
  TopoDS_Compound replacement;BRep_Builder().MakeCompound(replacement);
  TopoDS_Compound old;old.TShape(new CompoundValueDestructorProbe(&map,&replacement));
  map.Add(drawer,old);old.Nullify();
  const auto& alias=map.FindFromIndex(1);
  map.ChangeFromIndex(1)=replacement;
  std::cout<<"assigned "<<alias.IsSame(replacement)<<' '<<replacement.TShape()->GetRefCount()<<'\n';
  map.Clear(false);
  std::cout<<"assignment-cleared "<<replacement.TShape()->GetRefCount()<<'\n';
}
