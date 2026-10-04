#include <BRepAlgo_AsDes.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <NCollection_List.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

int main() {
  const TopoDS_Shape a = BRepBuilderAPI_MakeVertex(gp_Pnt(0, 0, 0));
  const TopoDS_Shape b = BRepBuilderAPI_MakeVertex(gp_Pnt(1, 0, 0));
  const TopoDS_Shape c = BRepBuilderAPI_MakeVertex(gp_Pnt(2, 0, 0));
  const TopoDS_Shape d = BRepBuilderAPI_MakeVertex(gp_Pnt(3, 0, 0));
  const TopoDS_Shape e = BRepBuilderAPI_MakeVertex(gp_Pnt(4, 0, 0));
  BRepAlgo_AsDes relations;
  relations.Add(a, b.Oriented(TopAbs_REVERSED));
  relations.Add(c, b);
  relations.Add(a, b);
  NCollection_List<TopoDS_Shape> common;
  const bool shared = relations.HasCommonDescendant(a, c, common);
  std::cout << "initial " << relations.Descendant(a).Extent() << " "
            << relations.Ascendant(b).Extent() << " " << shared << " "
            << common.Extent() << "\n";
  relations.ChangeDescendant(a).Append(d);
  std::cout << "changed " << relations.Descendant(a).Extent() << " "
            << relations.HasAscendant(d) << "\n";
  relations.Replace(a, e);
  std::cout << "replaced " << relations.HasDescendant(a) << " "
            << relations.Descendant(e).Extent() << " " << relations.Ascendant(b).Extent() << "\n";
  relations.Remove(b);
  std::cout << "removed " << relations.HasAscendant(b) << " "
            << relations.Descendant(e).Extent() << " " << relations.Descendant(c).Extent() << "\n";
  relations.Clear();
  std::cout << "cleared " << relations.HasDescendant(e) << " "
            << relations.HasAscendant(b) << "\n";
  NCollection_List<TopoDS_Shape> replacements;
  replacements.Append(b);
  replacements.Append(c);
  relations.Add(a, replacements);
  std::cout << "list_add " << relations.Descendant(a).Extent() << " "
            << relations.Ascendant(b).Extent() << "\n";
  relations.Clear();
  relations.Add(a, b);
  relations.Add(e, d);
  relations.Replace(a, e);
  std::cout << "merge_replace " << relations.Descendant(e).Extent() << " "
            << relations.Ascendant(b).Extent() << "\n";
}
