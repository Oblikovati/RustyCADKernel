// Font_FontMgr, Font_FTFont and Font_TextFormatter, the Rust twin of
// crates/tkservice/examples/sv_probe/font.rs. The fonts are the system fonts of the machine
// that runs both probes (fontconfig, FreeType); the messages are silenced.
#include <Font_FTFont.hxx>
#include <Font_FontMgr.hxx>
#include <Font_TextFormatter.hxx>
#include <Message.hxx>
#include <Message_Messenger.hxx>
#include <NCollection_String.hxx>
void sv_font_quiet(){Message::DefaultMessenger()->ChangePrinters().Clear();}
const char* const SV_FONT_NAMES[]={"sans-serif","monospace","serif","DejaVu Sans","Courier","cjk","korean","no-such-font","Times-Bold","arial","",""};
// The UTF-32 code points `N (C)*` as a UTF-8 string.
NCollection_String sv_font_text(GaReader& r){
 int n=r.integer();std::vector<char32_t> c(n+1,0);for(int i=0;i<n;++i)c[i]=(char32_t)r.integer();
 return NCollection_String(c.data(),-1);}
void sv_font_rect(const Font_Rect& b){ga_real(b.Left);ga_real(b.Right);ga_real(b.Top);ga_real(b.Bottom);}
// FNV-1a of the glyph image pixels, row by row (bytes of SizeX * pixel size per row).
void sv_font_image(const Image_PixMap& img){
 ga_int((int)img.SizeX());ga_int((int)img.SizeY());ga_int((int)img.Format());ga_flag(img.IsTopDown());
 uint32_t h=2166136261u;const size_t aRow=img.SizeX()*img.SizePixelBytes();
 for(size_t y=0;y<img.SizeY();++y){const uint8_t* p=img.Row(y);for(size_t x=0;x<aRow;++x){h=(h^p[x])*16777619u;}}
 ga_int((int)h);}
// `0`: the font database (every font's description), the aliases and FindFont over names,
// strict levels and aspects.
void sv_fontmgr_case(std::istream& input){
 GaReader r{input};(void)r;sv_font_quiet();occ::handle<Font_FontMgr> m=Font_FontMgr::GetInstance();
 NCollection_List<occ::handle<Font_SystemFont>> fonts=m->GetAvailableFonts();ga_int(fonts.Size());
 for(const occ::handle<Font_SystemFont>& f:fonts){ga_text(f->ToString().ToCString());ga_flag(f->IsSingleStrokeFont());}
 NCollection_Sequence<occ::handle<TCollection_HAsciiString>> aliases;m->GetAllAliases(aliases);ga_int(aliases.Size());
 for(const occ::handle<TCollection_HAsciiString>& a:aliases){ga_text(a->ToCString());
  NCollection_Sequence<occ::handle<TCollection_HAsciiString>> names;m->GetFontAliases(names,a->String());ga_int(names.Size());
  for(const occ::handle<TCollection_HAsciiString>& n:names)ga_text(n->ToCString());}
 for(const char* name:SV_FONT_NAMES){for(int s=0;s<3;++s){for(int asp=-1;asp<4;++asp){
  Font_FontAspect anAspect=(Font_FontAspect)asp;occ::handle<Font_SystemFont> f=m->FindFont(name,(Font_StrictLevel)s,anAspect);
  ga_flag(!f.IsNull());if(!f.IsNull())ga_text(f->FontName().ToCString());ga_int(anAspect);}}
  for(int sub=0;sub<4;++sub){occ::handle<Font_SystemFont> f=m->FindFallbackFont((Font_UnicodeSubset)sub,Font_FontAspect_Regular);
   ga_flag(!f.IsNull());if(!f.IsNull())ga_text(f->FontName().ToCString());}}
 ga_end();}
void sv_ftfont_metrics(const occ::handle<Font_FTFont>& f){
 ga_flag(f->IsValid());ga_real(f->Ascender());ga_real(f->Descender());ga_real(f->LineSpacing());
 ga_int((int)f->GlyphMaxSizeX(false));ga_int((int)f->GlyphMaxSizeY(false));ga_int(f->GlyphsNumber(false));
 ga_int((int)f->PointSize());ga_flag(f->ToSynthesizeItalic());ga_flag(f->IsSingleStrokeFont());ga_real(f->WidthScaling());}
void sv_ftfont_outline(const std::optional<Font_GlyphOutline>& o){
 ga_flag(o.has_value());if(!o.has_value())return;
 ga_real(o->UnitsPerEm());ga_int((int)o->Rule());ga_flag(o->HasPossibleOverlaps());ga_flag(o->IsSingleStroke());
 ga_int((int)o->Segments().Size());ga_int((int)o->Contours().Size());
 for(const Font_GlyphOutline::Segment& s:o->Segments()){ga_int((int)s.Type());ga_xy(s.Start());ga_xy(s.Control1());ga_xy(s.Control2());ga_xy(s.End());}
 for(const Font_GlyphOutline::Contour& c:o->Contours()){ga_int((int)c.FirstSegment());ga_int((int)c.NbSegments());}}
void sv_ftfont_char(const occ::handle<Font_FTFont>& f,char32_t c,char32_t next){
 ga_flag(f->HasSymbol(c));ga_flag(f->RenderGlyph(c));Font_Rect rect;f->GlyphRect(rect);sv_font_rect(rect);sv_font_image(f->GlyphImage());
 ga_real(f->AdvanceX(c,next));ga_real(f->AdvanceY(c,next));
 std::optional<Font_GlyphOutline::Glyph> g=f->ResolveGlyph(c);ga_flag(g.has_value());
 if(g.has_value()){ga_int((int)g->Face());ga_int((int)g->Index());ga_flag(f->IsGlyphValid(*g));}
 sv_ftfont_outline(f->LoadGlyphOutline(c));}
// `Name Aspect Strict Size Resolution Hinting Italic Single Width Fallback Text`: a font
// found by name (-1: the embedded fallback font), its metrics, each character (bitmap, advances, glyph, outline) and the
// text bounding box by alignment.
void sv_ftfont_case(std::istream& input){
 GaReader r{input};sv_font_quiet();int name=r.integer(),asp=r.integer(),strict=r.integer();
 Font_FTFont::Params p;p.PointSize=(uint32_t)r.integer();p.Resolution=(uint32_t)r.integer();p.FontHinting=(Font_Hinting)r.integer();
 p.ToSynthesizeItalic=r.flag();p.IsSingleStrokeFont=r.flag();float width=(float)r.real();bool fallback=r.flag();
 NCollection_String text=sv_font_text(r);
 occ::handle<Font_FTFont> f;
 if(name<0){f=new Font_FTFont();if(!f->Init(Font_FontMgr::EmbedFallbackFont(),"Embedded fallback",p,0))f.Nullify();}
 else f=Font_FTFont::FindAndCreate(SV_FONT_NAMES[name],(Font_FontAspect)asp,p,(Font_StrictLevel)strict);
 ga_flag(!f.IsNull());if(f.IsNull()){ga_end();return;}
 ga_flag(f->SetWidthScaling(width));f->SetUseUnicodeSubsetFallback(fallback);sv_ftfont_metrics(f);
 std::vector<char32_t> chars;for(NCollection_UtfIterator<char> it=text.Iterator();*it!=0;++it)chars.push_back(*it);
 for(size_t i=0;i<chars.size();++i)sv_ftfont_char(f,chars[i],i+1<chars.size()?chars[i+1]:0);
 ga_int((int)f->GlyphMaxSizeX(true));ga_int((int)f->GlyphMaxSizeY(true));ga_int(f->GlyphsNumber(true));
 for(int ax=0;ax<3;++ax)for(int ay=0;ay<4;++ay)GA_TRY(sv_font_rect(f->BoundingBox(text,(Graphic3d_HorizontalTextAlignment)ax,(Graphic3d_VerticalTextAlignment)ay)))
 ga_end();}
// `Name AlignX AlignY Wrapping WordWrap Size Text`: a formatted text, its corners, lines and
// glyph cells.
void sv_textfmt_case(std::istream& input){
 GaReader r{input};sv_font_quiet();int name=r.integer(),ax=r.integer(),ay=r.integer();float wrapping=(float)r.real();bool word=r.flag();
 Font_FTFont::Params p;p.PointSize=(uint32_t)r.integer();NCollection_String text=sv_font_text(r);
 occ::handle<Font_FTFont> f=Font_FTFont::FindAndCreate(SV_FONT_NAMES[name],Font_FontAspect_Regular,p,Font_StrictLevel_Any);
 ga_flag(!f.IsNull());if(f.IsNull()){ga_end();return;}
 Font_TextFormatter fmt;fmt.SetupAlignment((Graphic3d_HorizontalTextAlignment)ax,(Graphic3d_VerticalTextAlignment)ay);
 fmt.SetWrapping(wrapping);fmt.SetWordWrapping(word);GA_TRY(fmt.Append(text,*f))GA_TRY(fmt.Format())
 const int n=fmt.Corners().Length();ga_int(n);for(int i=0;i<n;++i){ga_real(fmt.BottomLeft(i).x());ga_real(fmt.BottomLeft(i).y());}
 ga_int(fmt.NewLines().Length());for(int i=0;i<fmt.NewLines().Length();++i)ga_real(fmt.NewLines()[i]);
 ga_real(fmt.ResultWidth());ga_real(fmt.ResultHeight());ga_real(fmt.MaximumSymbolWidth());ga_real(fmt.FirstPosition());
 Font_Rect box;fmt.BndBox(box);sv_font_rect(box);ga_real(fmt.LineHeight(0));ga_real(fmt.LineHeight(1));
 for(int i=-1;i<=fmt.NewLines().Length()+1;++i)ga_real(fmt.LineWidth(i));
 for(int i=-1;i<=n;++i){Font_Rect g;bool ok=fmt.GlyphBoundingBox(i,g);ga_flag(ok);if(ok)sv_font_rect(g);}
 for(int i=0;i<n;++i){ga_flag(fmt.IsLFSymbol(i));ga_int(fmt.LinePositionIndex(i));ga_int(fmt.LineIndex(i));}
 for(Font_TextFormatter::Iterator it(fmt,Font_TextFormatter::IterationFilter_ExcludeInvisible);it.More();it.Next()){
  ga_int((int)it.Symbol());ga_int(it.SymbolPosition());ga_flag(it.HasNext());ga_int((int)it.SymbolNext());}
 ga_end();}
