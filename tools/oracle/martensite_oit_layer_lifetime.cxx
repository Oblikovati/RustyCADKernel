// OCCT3d097a0 viewer-local layer IDs retain driver-global registration after view removal.
#include <Aspect_DisplayConnection.hxx>
#include <Graphic3d_ZLayerSettings.hxx>
#include <NCollection_Sequence.hxx>
#include <OpenGl_GraphicDriver.hxx>
#include <Standard_Failure.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <iostream>
static void layer_inventory(const occ::handle<OpenGl_GraphicDriver>& driver) {
  NCollection_Sequence<int> layers; driver->ZLayers(layers);
  std::cout << "layers";
  for (int i=1;i<=layers.Size();++i) std::cout << ' ' << layers.Value(i);
  std::cout << '\n';
}
static void duplicate_after_removed_view(const occ::handle<OpenGl_GraphicDriver>& driver) {
  const occ::handle<V3d_Viewer> next=new V3d_Viewer(driver);
  int id=0;
  try { next->AddZLayer(id); std::cout << "duplicate_not_rejected " << id << '\n'; }
  catch (const Standard_Failure& error) { std::cout << "duplicate_rejected " << error.what() << '\n'; }
}
int main() {
  const occ::handle<OpenGl_GraphicDriver> driver=new OpenGl_GraphicDriver(new Aspect_DisplayConnection,false);
  const occ::handle<V3d_Viewer> first=new V3d_Viewer(driver);
  const auto view=first->CreateView(); int id=0;
  layer_inventory(driver); first->AddZLayer(id); std::cout << "first " << id << '\n';
  view->Remove(); duplicate_after_removed_view(driver); layer_inventory(driver);
  std::cout << "removed " << first->RemoveZLayer(id) << '\n'; layer_inventory(driver);
  const occ::handle<V3d_Viewer> final=new V3d_Viewer(driver); int second=0;
  final->AddZLayer(second); std::cout << "reused " << second << '\n';
  final->RemoveZLayer(second); layer_inventory(driver); return 0;
}
