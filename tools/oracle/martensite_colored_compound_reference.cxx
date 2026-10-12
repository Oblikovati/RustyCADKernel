// Native scoped const Value reads and genuine owning copies preserve TShape identity/counts.
#include "martensite_colored_compound_map.hxx"
int main()
{
  const occ::handle<Prs3d_Drawer> parent=new Prs3d_Drawer;
  const occ::handle<AIS_ColoredDrawer> drawer=new AIS_ColoredDrawer(parent);
  TopoDS_Compound shape;BRep_Builder().MakeCompound(shape);
  DrawerCompounds map;map.Add(drawer,shape);
  const auto& reference=map.FindFromIndex(1);
  std::cout<<"referenced "<<reference.IsEqual(shape)<<' '<<reference.TShape()->GetRefCount()<<'\n';
  {
    TopoDS_Shape owned=reference;
    std::cout<<"copied "<<owned.IsEqual(shape)<<' '<<reference.TShape()->GetRefCount()<<'\n';
  }
  std::cout<<"copy-released "<<reference.TShape()->GetRefCount()<<'\n';
  map.Clear(false);
  std::cout<<"reference-finished "<<shape.TShape()->GetRefCount()<<'\n';
}
