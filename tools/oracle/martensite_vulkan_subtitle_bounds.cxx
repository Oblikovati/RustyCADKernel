// Actual native formatter advance/line bounds for Martensite Subtitle; OCCT3d097a0.
#include <Font_FTFont.hxx>
#include <Font_FontMgr.hxx>
#include <Font_TextFormatter.hxx>
#include <iomanip>
#include <iostream>
static const char* subtitle_strings[] = {"AV", "AV   ", " A V ", "A\tV", "AV\nA ", "A\n\nV", "", " \t "};
static void emit_subtitle_bounds(int index) {
  const int text = index % 8, ax = (index / 8) % 3, ay = (index / 24) % 4;
  Font_FTFont::Params params;
  params.PointSize = (index / 96) % 2 == 0 ? 16 : 24;
  params.Resolution = index / 192 == 0 ? 72 : 144;
  Font_FTFont font;
  if (!font.Init(Font_FontMgr::EmbedFallbackFont(), "Embedded Subtitle fixture", params, 0))
    throw Standard_Failure("embedded native Subtitle font failed; expected valid pinned font bytes");
  Font_TextFormatter formatter;
  formatter.SetupAlignment(Graphic3d_HorizontalTextAlignment(ax), Graphic3d_VerticalTextAlignment(ay));
  formatter.Append(NCollection_String(subtitle_strings[text]), font); formatter.Format();
  Font_Rect rect; formatter.BndBox(rect);
  std::cout << text << ' ' << ax << ' ' << ay << ' ' << params.PointSize << ' ' << params.Resolution << ' ';
  std::cout << rect.Left << ' ' << rect.Right << ' ' << rect.Top << ' ' << rect.Bottom << '\n';
}
int main() {
  std::cout << std::setprecision(9);
  for (int index = 0; index < 384; ++index) emit_subtitle_bounds(index);
  return 0;
}
