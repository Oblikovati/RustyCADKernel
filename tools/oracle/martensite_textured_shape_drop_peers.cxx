// Actual native pixmap destructor observes derived texture peers before AIS_Shape releases its drawer.
#include <AIS_TexturedShape.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Graphic3d_Texture2D.hxx>
#include <Image_PixMap.hxx>
#include <Prs3d_Drawer.hxx>
#include <iostream>

class NativeDropPeerPixels : public Image_PixMap {
public:
  NativeDropPeerPixels(const occ::handle<Prs3d_Drawer>& drawer,
                       const occ::handle<Graphic3d_AspectFillArea3d>& aspect,
                       const occ::handle<Graphic3d_Texture2D>& texture)
    : myDrawer(drawer), myAspect(aspect), myTexture(texture) {
    // Real handles keep every observed peer alive throughout the destructor callback.
  }
  ~NativeDropPeerPixels() override {
    std::cout << "MARTENSITE_TEXTURED_DROP " << myDrawer->GetRefCount() << ' '
      << myAspect->GetRefCount() << ' ' << myTexture->GetRefCount() << '\n';
  }
private:
  occ::handle<Prs3d_Drawer> myDrawer;
  occ::handle<Graphic3d_AspectFillArea3d> myAspect;
  occ::handle<Graphic3d_Texture2D> myTexture;
};
class NativeDropPeerShape : public AIS_TexturedShape {
public:
  NativeDropPeerShape()
    : AIS_TexturedShape(TopoDS_Shape()) {
    // Null topology isolates the real derived-member/base destruction ordering.
  }
  void installPeers(const occ::handle<Graphic3d_AspectFillArea3d>& aspect,
                    const occ::handle<Graphic3d_Texture2D>& texture) {
    myAspect = aspect;
    myTexture = texture;
  }
};
int main() {
  occ::handle<NativeDropPeerShape> object = new NativeDropPeerShape();
  const occ::handle<Prs3d_Drawer> drawer = object->Attributes();
  const occ::handle<Graphic3d_AspectFillArea3d> aspect = new Graphic3d_AspectFillArea3d();
  const occ::handle<Graphic3d_Texture2D> texture = new Graphic3d_Texture2D(TCollection_AsciiString());
  occ::handle<NativeDropPeerPixels> pixels = new NativeDropPeerPixels(drawer, aspect, texture);
  object->SetTexturePixMap(pixels);
  object->installPeers(aspect, texture);
  pixels.Nullify();
  object.Nullify();
  return 0;
}
