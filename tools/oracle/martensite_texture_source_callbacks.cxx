// Actual retiring-slot observations at OCCT 3d097a0; no owning copies of the returned field.
#include <AIS_TexturedShape.hxx>
#include <Image_PixMap.hxx>
#include <functional>
#include <iostream>
#include <string>

class NativeTextureSlotShape : public AIS_TexturedShape {
public:
  NativeTextureSlotShape()
    : AIS_TexturedShape(TopoDS_Shape()) {
    // Null topology keeps this oracle independent of rendering and codecs.
  }
  void seedFile() {
    myTextureFile = "prior.raw";
    myPredefTexture = Graphic3d_NOT_2D_MATRA;
  }
  int predefined() const {
    // Protected native storage is inspected without changing ownership.
    return static_cast<int>(myPredefTexture);
  }
};
static void printTextureState(const NativeTextureSlotShape& owner) {
  std::cout << owner.predefined();
  const char* file = owner.TextureFile();
  if (*file != '\0') std::cout << ' ' << file;
  std::cout << '\n';
}
class NativeTextureSlotPixels : public Image_PixMap {
public:
  NativeTextureSlotPixels(NativeTextureSlotShape& owner, const std::string& label,
                          const Image_PixMap* incoming, std::function<void()> callback = {})
    : myOwner(owner), myLabel(label), myIncoming(incoming), myCallback(callback) {
    // The owner remains alive throughout every explicit source operation.
  }
  ~NativeTextureSlotPixels() override {
    if (myCallback) myCallback();
    const auto& slot = myOwner.TexturePixMap();
    std::cout << "MARTENSITE_TEXTURE_SLOT " << myLabel << ' '
      << (slot.get() == this) << ' ' << slot.IsNull() << ' ' << GetRefCount() << ' '
      << (myIncoming ? myIncoming->GetRefCount() : -1) << ' '
      ;
    printTextureState(myOwner);
  }
private:
  NativeTextureSlotShape& myOwner;
  std::string myLabel;
  const Image_PixMap* myIncoming;
  std::function<void()> myCallback;
};
static void runRetiringSlot(const std::string& label, bool replace, bool keepOwner) {
  const occ::handle<NativeTextureSlotShape> owner = new NativeTextureSlotShape;
  const occ::handle<Image_PixMap> incoming = replace ? new Image_PixMap : nullptr;
  occ::handle<Image_PixMap> pixels = new NativeTextureSlotPixels(*owner, label, incoming.get());
  owner->SetTexturePixMap(pixels); owner->seedFile();
  const auto& alias = owner->TexturePixMap(); const auto* slotAddress = &alias;
  if (!keepOwner) pixels.Nullify();
  if (replace) owner->SetTexturePixMap(incoming);
  else owner->SetTextureFileName(TCollection_AsciiString("next.raw"));
  std::cout << "MARTENSITE_TEXTURE_AFTER " << label << ' '
    << (&owner->TexturePixMap() == slotAddress) << ' ' << alias.IsNull() << ' '
    << (incoming.IsNull() ? -1 : incoming->GetRefCount()) << ' '
    ;
  printTextureState(*owner);
  pixels.Nullify();
}
static void runSameSlot() {
  const occ::handle<NativeTextureSlotShape> owner = new NativeTextureSlotShape;
  occ::handle<Image_PixMap> pixels = new NativeTextureSlotPixels(*owner, "same-slot", nullptr);
  owner->SetTexturePixMap(pixels); pixels.Nullify(); owner->seedFile();
  const auto& alias = owner->TexturePixMap(); const auto* address = alias.get();
  owner->SetTexturePixMap(alias);
  std::cout << "MARTENSITE_TEXTURE_SAME " << (alias.get() == address) << ' '
    << alias->GetRefCount() << ' ';
  printTextureState(*owner);
  owner->SetTextureFileName(TCollection_AsciiString("cleanup.raw"));
}
static void runIncomingSlotMutation() {
  const occ::handle<NativeTextureSlotShape> owner = new NativeTextureSlotShape;
  const occ::handle<NativeTextureSlotShape> input = new NativeTextureSlotShape;
  const occ::handle<Image_PixMap> original = new Image_PixMap;
  const occ::handle<Image_PixMap> next = new Image_PixMap;
  input->SetTexturePixMap(original);
  const auto& alias = input->TexturePixMap();
  const auto callback = [&]() { input->SetTexturePixMap(next); };
  occ::handle<Image_PixMap> pixels = new NativeTextureSlotPixels(*owner,"incoming-slot",original.get(),callback);
  owner->SetTexturePixMap(pixels); pixels.Nullify();
  owner->SetTexturePixMap(alias);
  std::cout << "MARTENSITE_TEXTURE_CAPTURED " << (owner->TexturePixMap()==original) << ' '
    << original->GetRefCount() << ' ' << (alias==next) << ' ' << next->GetRefCount() << '\n';
}
int main() {
  runRetiringSlot("file-sole", false, false);
  runRetiringSlot("pixmap-sole", true, false);
  runRetiringSlot("file-retained", false, true);
  runRetiringSlot("pixmap-retained", true, true);
  runSameSlot();
  runIncomingSlotMutation();
  return 0;
}
