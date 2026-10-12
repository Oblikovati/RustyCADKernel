// Pinned OCCT 3d097a0: reads copy C bytes only at the native AsciiString boundary.
#include <V3d_ImageDumpOptions.hxx>
#include <TCollection_AsciiString.hxx>
#include <Standard_NullObject.hxx>
#include <cstring>
#include <cstddef>
#include <iomanip>
#include <iostream>

static void emit(const char* label, const V3d_ImageDumpOptions& options, const char* anchor)
{
  std::cout << label << ' ' << (options.LightName == anchor);
  try
  {
    TCollection_AsciiString consumed(options.LightName);
    std::cout << ' ' << consumed.Length();
    for (int index = 1; index <= consumed.Length(); ++index)
      std::cout << ' ' << int(static_cast<unsigned char>(consumed.Value(index)));
  }
  catch (const Standard_NullObject&)
  {
    std::cout << " Standard_NullObject";
  }
  std::cout << '\n';
}

static void alias_cases()
{
  char anchor[8] = "sun";
  V3d_ImageDumpOptions options;
  options.LightName = anchor;
  V3d_ImageDumpOptions copy = options;
  emit("original", options, anchor);
  emit("copy", copy, anchor);
  std::memcpy(anchor, "moon\0xxx", 8);
  emit("mutated", options, anchor);
  emit("mutated-copy", copy, anchor);
  anchor[1] = '\0';
  emit("truncated", copy, anchor);
  options.LightName = "rebound";
  emit("rebound", options, copy.LightName);
  emit("still-anchored", copy, anchor);
}

static void encoding_cases()
{
  const char anchor[5] = {char(0xff), char(0x80), 'x', '\0', 'y'};
  V3d_ImageDumpOptions options;
  options.LightName = anchor;
  emit("bytes", options, anchor);
  options.LightName = anchor + 1;
  emit("interior", options, anchor + 1);
  options.LightName = anchor + 3;
  emit("empty-interior", options, anchor + 3);
}

static void layout_record()
{
  std::cout << "layout " << sizeof(V3d_ImageDumpOptions) << ' ' << alignof(V3d_ImageDumpOptions)
    << ' ' << offsetof(V3d_ImageDumpOptions, Width) << ' ' << offsetof(V3d_ImageDumpOptions, Height)
    << ' ' << offsetof(V3d_ImageDumpOptions, BufferType) << ' ' << offsetof(V3d_ImageDumpOptions, StereoOptions)
    << ' ' << offsetof(V3d_ImageDumpOptions, TileSize) << ' ' << offsetof(V3d_ImageDumpOptions, ToAdjustAspect)
    << ' ' << offsetof(V3d_ImageDumpOptions, TargetZLayerId) << ' ' << offsetof(V3d_ImageDumpOptions, IsSingleLayer)
    << ' ' << offsetof(V3d_ImageDumpOptions, LightName) << '\n';
}

int main()
{
  V3d_ImageDumpOptions options;
  emit("default", options, options.LightName);
  options.LightName = nullptr;
  emit("null", options, nullptr);
  alias_cases();
  encoding_cases();
  layout_record();
  return 0;
}
