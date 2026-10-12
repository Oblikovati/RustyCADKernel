// Pinned AIS_TexturedShape borrowed TextureFile buffer: read only valid native aliases.
#include <AIS_TexturedShape.hxx>
#include <TCollection_AsciiString.hxx>
#include <TopoDS_Shape.hxx>
#include <cstring>
#include <iostream>
#include <utility>

class NativeBorrowedTextureFile : public AIS_TexturedShape {
public:
  NativeBorrowedTextureFile()
    : AIS_TexturedShape(TopoDS_Shape()) {
    // No viewer is needed to inspect genuine source storage and intrusive ownership.
  }
  TCollection_AsciiString& storedFile() {
    // Direct protected mutation proves the adapter is not merely a setter epoch.
    return myTextureFile;
  }
};

void emit_buffer(const char* label, const occ::handle<NativeBorrowedTextureFile>& object,
                 const char* captured) {
  std::cout << label << ' ' << (captured == object->TextureFile()) << ' '
    << object->GetRefCount() << ' ' << object->storedFile().Length() << ' '
    << std::strlen(captured);
  if (captured[0] != '\0') {
    std::cout << ' ';
  }
  for (size_t index = 0; index < std::strlen(captured); ++index) {
    std::cout << static_cast<unsigned int>(static_cast<unsigned char>(captured[index])) << ',';
  }
  std::cout << '\n';
}

void probe_reused_buffer(const occ::handle<NativeBorrowedTextureFile>& object) {
  object->SetTextureFileName("abcdefghi.png");
  const char* captured = object->TextureFile();
  emit_buffer("original", object, captured);
  object->SetTextureFileName("qwertyuio.dat");
  emit_buffer("same-length", object, captured);
  object->SetTextureFileName("good.dat");
  emit_buffer("shorter", object, captured);
  TCollection_AsciiString embedded;
  embedded.Copy("ABC\0tail", 8);
  object->SetTextureFileName(embedded);
  emit_buffer("embedded-nul", object, captured);
  object->storedFile().SetValue(1, 'Z');
  emit_buffer("protected-in-place", object, captured);
  object->storedFile().Trunc(0);
  emit_buffer("owned-truncated-empty", object, captured);
}

void probe_static_buffer(const occ::handle<NativeBorrowedTextureFile>& object) {
  const char* captured = object->TextureFile();
  emit_buffer("initial-static", object, captured);
  object->SetTextureFileName("abc");
  // Native static-empty aliases remain valid even after the source allocates a buffer.
  emit_buffer("retained-static", object, captured);
  object->SetTextureFileName("0");
  emit_buffer("predefined-static", object, captured);
  object->SetTextureFileName("abc");
  object->SetTexturePixMap(occ::handle<Image_PixMap>());
  emit_buffer("pixmap-static", object, captured);
}

void probe_transferred_buffer(const occ::handle<NativeBorrowedTextureFile>& object) {
  object->SetTextureFileName("move-owner.png");
  const char* captured = object->TextureFile();
  TCollection_AsciiString moved(std::move(object->storedFile()));
  emit_buffer("moved-owner", object, captured);
  moved.Copy("short.png");
  emit_buffer("moved-owner-edit", object, captured);
  TCollection_AsciiString other("swapped");
  moved.Swap(other);
  emit_buffer("swapped-owner", object, captured);
  // The captured allocation is now owned by other, and becomes invalid when it clears.
  other.Clear();
  emit_buffer("fresh-after-transferred-release", object, object->TextureFile());
}

int main() {
  const occ::handle<NativeBorrowedTextureFile> object = new NativeBorrowedTextureFile();
  probe_static_buffer(object);
  probe_reused_buffer(object);
  // Copy to empty or longer bytes releases storage: never inspect the old dangling pointer.
  object->SetTextureFileName("a-longer-reallocated-file.png");
  emit_buffer("fresh-after-release", object, object->TextureFile());
  probe_transferred_buffer(object);
  return 0;
}
