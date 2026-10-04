#include <BRepAlgo_Image.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <Standard_ConstructionError.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

TopoDS_Shape vertex(double x) {
  return BRepBuilderAPI_MakeVertex(gp_Pnt(x, 0.0, 0.0)).Shape();
}

int main() {
  const TopoDS_Shape source = vertex(0.0);
  const TopoDS_Shape middle = vertex(1.0);
  const TopoDS_Shape image = vertex(2.0);
  BRepAlgo_Image links;
  links.SetRoot(source);
  links.Bind(source, middle);
  links.SetRoot(middle);
  links.Bind(middle, image);
  NCollection_List<TopoDS_Shape> last;
  links.LastImage(source, last);
  std::cout << "chain " << links.Root(image).IsSame(source) << ' '
            << links.Image(source).Extent() << ' ' << last.Extent() << ' '
            << last.First().IsSame(image) << '\n';
  links.Compact();
  std::cout << "compact " << links.Image(source).Extent() << ' '
            << links.Image(source).First().IsSame(image) << ' '
            << links.ImageFrom(image).IsSame(middle) << '\n';

  BRepAlgo_Image partial;
  const TopoDS_Shape secondSource = vertex(3.0);
  partial.Bind(source, middle);
  bool constructionError = false;
  try {
    partial.Bind(secondSource, middle);
  } catch (const Standard_ConstructionError&) {
    constructionError = true;
  }
  std::cout << "duplicate-image " << constructionError << ' '
            << partial.HasImage(secondSource) << ' '
            << partial.Image(secondSource).Extent() << ' '
            << partial.ImageFrom(middle).IsSame(secondSource) << '\n';
  BRepAlgo_Image replacement;
  const TopoDS_Shape newRoot = vertex(4.0);
  replacement.SetRoot(source);
  replacement.Bind(source, image);
  replacement.ReplaceRoot(source, newRoot);
  std::cout << "replace-root " << replacement.Roots().Extent() << ' '
            << replacement.ImageFrom(image).IsSame(newRoot) << '\n';
  replacement.RemoveRoot(newRoot);
  std::cout << "remove-root " << replacement.Roots().Extent() << ' '
            << replacement.HasImage(newRoot) << ' ' << replacement.IsImage(image) << '\n';
  BRepAlgo_Image fallback;
  std::cout << "unbound " << fallback.Image(source).Extent() << ' '
            << fallback.Image(middle).Extent() << '\n';
}
