// Actual IndexedDataMap iterator keeps its map reference and integer index, including later additions.
#include "martensite_colored_compound_map.hxx"
int main()
{
  const occ::handle<Prs3d_Drawer> parent=new Prs3d_Drawer;
  const occ::handle<AIS_ColoredDrawer> first=new AIS_ColoredDrawer(parent),second=new AIS_ColoredDrawer(parent);
  TopoDS_Compound shape;BRep_Builder().MakeCompound(shape);
  DrawerCompounds map;DrawerCompounds::Iterator cursor(map);
  std::cout<<"cursor-empty "<<cursor.More()<<'\n';
  map.Add(first,shape);
  std::cout<<"cursor-added "<<cursor.More()<<' '<<(cursor.Key()==first)<<'\n';
  const auto* address=&cursor.Value();map.ReSize(1000);map.Add(second,shape);
  std::cout<<"cursor-resized "<<cursor.More()<<' '<<(&cursor.Value()==address)<<'\n';
  cursor.Next();
  std::cout<<"cursor-next "<<cursor.More()<<' '<<(cursor.Key()==second)<<'\n';
  map.Clear(false);
  std::cout<<"cursor-cleared "<<cursor.More()<<'\n';
}
