// Graphic3d_MediaTexture and Graphic3d_MediaTextureSet without FFmpeg, the Rust twin of
// crates/tkservice/examples/sv_probe/mtex.rs.
#include <Graphic3d_MediaTexture.hxx>
#include <Graphic3d_MediaTextureSet.hxx>
#include <Graphic3d_ShaderObject.hxx>
#include <Graphic3d_ShaderProgram.hxx>
#include <Graphic3d_TextureParams.hxx>
#include <Image_SupportedFormats.hxx>
#include <Media_Frame.hxx>
// Exposes the protected YUV programs.
class SvMediaSet : public Graphic3d_MediaTextureSet
{
public:
  const occ::handle<Graphic3d_ShaderProgram>& Yuv(bool theFull) const { return theFull ? myShaderYUVJ : myShaderYUV; }
};
// Prints a program: header, light and plane limits, stage types and sources.
void sv_mtex_program(const occ::handle<Graphic3d_ShaderProgram>& theProgram)
{
  ga_text(theProgram->Header().ToCString()); ga_int(theProgram->NbLightsMax()); ga_int(theProgram->NbClipPlanesMax());
  for (NCollection_Sequence<occ::handle<Graphic3d_ShaderObject>>::Iterator anIt(theProgram->ShaderObjects()); anIt.More(); anIt.Next())
  { ga_int(anIt.Value()->Type()); ga_text(anIt.Value()->Source().ToCString()); }
}
// `LockCount`: a media texture set: its textures, programs and frame queue (the frames are
// locked and released `LockCount` times), then the swap without player.
void sv_mtex_case(std::istream& input)
{
  GaReader r{input}; const int aNbLocks = r.integer();
  occ::handle<SvMediaSet> aSet = new SvMediaSet();
  ga_int(aSet->Size());
  for (int i = aSet->Lower(); i <= aSet->Upper(); ++i)
  {
    occ::handle<Graphic3d_MediaTexture> aTexture = occ::down_cast<Graphic3d_MediaTexture>(aSet->Value(i));
    const occ::handle<Graphic3d_TextureParams>& aParams = aTexture->GetParams();
    ga_int(aParams->TextureUnit()); ga_flag(aParams->IsModulate()); ga_flag(aParams->IsRepeat()); ga_int(aParams->Filter());
    ga_flag(aTexture->Frame().IsNull()); ga_flag(aTexture->GetImage(new Image_SupportedFormats()).IsNull());
  }
  sv_mtex_program(aSet->Yuv(false)); sv_mtex_program(aSet->Yuv(true));
  ga_flag(aSet->ShaderProgram().IsNull()); ga_flag(aSet->IsPlanarYUV()); ga_flag(aSet->IsFullRangeYUV());
  Media_IFrameQueue* aQueue = aSet.get();
  for (int i = 0; i < aNbLocks; ++i)
  {
    occ::handle<Media_Frame> aFrame = aQueue->LockFrame(); ga_flag(aFrame.IsNull());
    if (aFrame.IsNull()) continue;
    ga_flag(aFrame->IsLocked()); ga_flag(aQueue->LockFrame().IsNull());
    if (i % 2 == 0) { aQueue->ReleaseFrame(aFrame); ga_flag(aFrame->IsLocked()); }
  }
  ga_flag(aSet->SwapFrames()); ga_flag(aSet->PlayerContext().IsNull());
  aSet->OpenInput("", false); ga_text(aSet->Input().ToCString()); ga_flag(aSet->PlayerContext().IsNull());
  ga_int(aSet->FrameSize().x()); ga_int(aSet->FrameSize().y()); ga_real(aSet->Progress()); ga_real(aSet->Duration());
  ga_end();
}
