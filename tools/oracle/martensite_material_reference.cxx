// Test-only packing oracle using pinned OCCT 3d097a0; no production OCCT linkage.
#include <OpenGl_Context.hxx>
#include <OpenGl_Caps.hxx>
#include <OpenGl_Material.hxx>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <string>

static void emit(const std::string& key, const OpenGl_Material& material) {
  std::cout << "{\"case\":\"" << key << "\",\"bits\":[";
  const float* common = reinterpret_cast<const float*>(material.PackedCommon());
  const float* pbr = reinterpret_cast<const float*>(material.PackedPbr());
  for (int index = 0; index < 56; ++index) {
    uint32_t bits;
    float value = index < 32 ? common[index] : pbr[index - 32];
    std::memcpy(&bits, &value, sizeof(bits));
    if (index) std::cout << ',';
    std::cout << bits;
  }
  std::cout << "]}\n";
}
static OpenGl_Material initialized(OpenGl_Context& context, int rank, bool alias, bool modified) {
  Graphic3d_MaterialAspect front(static_cast<Graphic3d_NameOfMaterial>(rank));
  Graphic3d_MaterialAspect back(static_cast<Graphic3d_NameOfMaterial>(25 - rank));
  if (modified) {
    front.SetAlpha(0.375f);
    front.SetShininess(0.375f);
    front.SetColor(Quantity_Color(0.125, 0.5, 0.875, Quantity_TOC_RGB));
  }
  OpenGl_Material packed;
  packed.Pbr[0].Params = NCollection_Vec4<float>(7, 8, 9, 10);
  packed.Pbr[1].Params = NCollection_Vec4<float>(11, 12, 13, 14);
  packed.Init(context, front, Quantity_Color(0.25, 0.5, 0.75, Quantity_TOC_RGB),
              alias ? front : back, Quantity_Color(0.875, 0.125, 0.5, Quantity_TOC_RGB));
  return packed;
}
static void variants(OpenGl_Context& context, int rank, bool linear, bool alias) {
  for (bool modified : {false, true}) {
    std::string key = std::to_string(rank) + "-" + (linear ? "true" : "false")
      + "-" + (alias ? "true" : "false") + "-" + (modified ? "true" : "false");
    OpenGl_Material material = initialized(context, rank, alias, modified);
    emit(key, material);
    material.SetColor(NCollection_Vec3<float>(0.75f, 0.25f, 0.125f));
    emit(key + "-highlight", material);
  }
}
int main() {
  OpenGl_Context context;
  // Emulate an sRGB-capable ES context without explicit sRGB write control:
  // SetFrameBufferSRGB then updates its actual state without a GL function call.
  context.hasTexSRGB = true;
  context.hasFboSRGB = true;
  context.hasSRGBControl = false;
  context.caps->sRGBDisable = false;
  context.caps->ffpEnable = false;
  emit("default", OpenGl_Material());
  for (int rank = 0; rank < 26; ++rank) {
    for (bool linear : {false, true}) {
      context.SetFrameBufferSRGB(true, linear);
      if (context.IsFrameBufferSRGB() != linear) return 2;
      variants(context, rank, linear, false);
      variants(context, rank, linear, true);
    }
  }
}
