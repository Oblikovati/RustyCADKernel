// Native destructor-time pointee access and nested destination assignment, OCCT 3d097a0.
#include <AIS_TexturedShape.hxx>
#include <Image_PixMap.hxx>
#include <iostream>
#include <string>
class RetirementProjectionPixels : public Image_PixMap {
public:
  RetirementProjectionPixels(AIS_TexturedShape& owner, std::string mode,
                             const occ::handle<Image_PixMap>& nested)
    : myOwner(owner), myMode(mode), myNested(nested) {
    InitTrash(Image_Format_Gray, 3, 2);
    ChangeData()[0] = 73;
  }
  ~RetirementProjectionPixels() override {
    const auto& slot = myOwner.TexturePixMap();
    std::cout << "POINTEE " << myMode << ' ' << (slot.get() == this) << ' '
      << slot->GetRefCount() << ' ' << slot->SizeX() << ' ' << slot->SizeY() << ' '
      << int(slot->Data()[0]) << '\n';
    if (myMode == "same") myOwner.SetTexturePixMap(slot);
    if (myMode == "file") myOwner.SetTextureFileName("inner.raw");
    if (myMode == "pixmap") myOwner.SetTexturePixMap(myNested);
    std::cout << "NESTED " << myMode << ' ' << GetRefCount() << ' '
      << (slot.get() == this) << ' ' << slot.IsNull() << ' '
      << (myNested.IsNull() ? -1 : myNested->GetRefCount()) << ' '
      << myOwner.TextureFile() << '\n';
  }
private:
  AIS_TexturedShape& myOwner;
  std::string myMode;
  const occ::handle<Image_PixMap>& myNested;
};
static void runRetirementProjection(const std::string& mode, bool outerFile) {
  const occ::handle<AIS_TexturedShape> owner = new AIS_TexturedShape(TopoDS_Shape());
  const occ::handle<Image_PixMap> next = new Image_PixMap;
  const occ::handle<Image_PixMap> nested = new Image_PixMap;
  occ::handle<Image_PixMap> old = new RetirementProjectionPixels(*owner, mode, nested);
  owner->SetTexturePixMap(old); old.Nullify();
  if (outerFile) owner->SetTextureFileName("outer.raw");
  else owner->SetTexturePixMap(next);
  const auto& slot = owner->TexturePixMap();
  std::cout << "AFTER " << mode << ' ' << outerFile << ' ' << slot.IsNull() << ' '
    << (slot == next) << ' ' << next->GetRefCount() << ' ' << nested->GetRefCount()
    << ' ' << owner->TextureFile() << '\n';
}
int main() {
  for (const auto& mode : {"none", "same", "file", "pixmap"}) {
    runRetirementProjection(mode, false);
    runRetirementProjection(mode, true);
  }
  return 0;
}
