// Pinned OCCT 3d097a0: public fields accept direct assignment without view validation.
#include <V3d_ImageDumpOptions.hxx>
#include <cassert>
#include <iostream>

static void emit(const char* label, const V3d_ImageDumpOptions& options)
{
  std::cout << label << ' ' << options.Width << ' ' << options.Height << ' '
            << int(options.BufferType) << ' ' << int(options.StereoOptions) << ' '
            << options.TileSize << ' ' << options.ToAdjustAspect << ' '
            << options.TargetZLayerId << ' ' << options.IsSingleLayer << '\n';
}

static void assign(V3d_ImageDumpOptions& options, const int index)
{
  options.Width = -index;
  options.Height = 640 + index;
  options.BufferType = Graphic3d_BufferType(index % 6);
  options.StereoOptions = V3d_StereoDumpOptions(index % 4);
  options.TileSize = -20 + index;
  options.ToAdjustAspect = (index % 2) != 0;
  options.TargetZLayerId = -100 + index;
  options.IsSingleLayer = (index % 2) == 0;
}

int main()
{
  V3d_ImageDumpOptions options;
  assert(options.LightName != nullptr && options.LightName[0] == '\0');
  emit("default", options);
  for (int index = 0; index < 12; ++index)
  {
    assign(options, index);
    emit("assigned", options);
  }
  V3d_ImageDumpOptions copy = options;
  assign(options, 42);
  emit("copy", copy);
  emit("original", options);
  return 0;
}
