// Genuine two-level Image_PixMap destructors replace the same native slot, OCCT 3d097a0.
#include <AIS_TexturedShape.hxx>
#include <Image_PixMap.hxx>
#include <iostream>
struct TextureChainControl {
  occ::handle<AIS_TexturedShape> owner;
  occ::handle<Image_PixMap> nested;
  occ::handle<Image_PixMap> final;
  Image_PixMap* last = nullptr;
};
class TextureChainPixels : public Image_PixMap {
public:
  TextureChainPixels(TextureChainControl& control, int stage) : myControl(control), myStage(stage) {
    InitTrash(Image_Format_Gray, stage + 2, 2);
    ChangeData()[0] = uint8_t(73 + stage);
  }
  ~TextureChainPixels() override {
    const auto& slot = myControl.owner->TexturePixMap();
    std::cout << "POINTEE " << myStage << ' ' << (slot.get() == this) << ' '
      << slot->GetRefCount() << ' ' << slot->SizeX() << ' ' << slot->SizeY() << ' '
      << int(slot->Data()[0]) << '\n';
    if (myStage == 1) runFirstStage();
    if (myStage == 2) runSecondStage();
    std::cout << "CHAIN " << myStage << ' ' << GetRefCount() << ' '
      << slot.IsNull() << ' ' << myControl.last->GetRefCount();
    const char* file = myControl.owner->TextureFile();
    if (*file != '\0') std::cout << ' ' << file;
    std::cout << '\n';
  }
private:
  void runFirstStage() {
    myControl.owner->SetTexturePixMap(myControl.nested);
    myControl.nested.Nullify();
    myControl.owner->SetTextureFileName("inner.raw");
  }
  void runSecondStage() {
    myControl.owner->SetTexturePixMap(myControl.final);
    myControl.final.Nullify();
    // Only the native callback-published slot now owns final; outer EndScope loses that reference.
  }
  TextureChainControl& myControl;
  int myStage;
};
static void runTextureChain(bool outerFile) {
  TextureChainControl control;
  control.owner = new AIS_TexturedShape(TopoDS_Shape());
  control.nested = new TextureChainPixels(control, 2);
  control.final = new TextureChainPixels(control, 3);
  control.last = control.final.get();
  occ::handle<Image_PixMap> old = new TextureChainPixels(control, 1);
  const occ::handle<Image_PixMap> next = new Image_PixMap;
  control.owner->SetTexturePixMap(old); old.Nullify();
  if (outerFile) control.owner->SetTextureFileName("outer.raw");
  else control.owner->SetTexturePixMap(next);
  std::cout << "AFTER " << outerFile << ' ' << control.owner->TexturePixMap().IsNull()
    << ' ' << next->GetRefCount() << ' ' << control.last->GetRefCount();
  const char* file = control.owner->TextureFile();
  if (*file != '\0') std::cout << ' ' << file;
  std::cout << '\n';
}
int main() {
  runTextureChain(false);
  runTextureChain(true);
  return 0;
}
