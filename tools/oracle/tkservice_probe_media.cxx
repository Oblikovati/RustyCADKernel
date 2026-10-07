// The Media package without FFmpeg, the Rust twin of crates/tkservice/examples/sv_probe/media.rs.
#include <Image_PixMap.hxx>
#include <Media_BufferPool.hxx>
#include <Media_CodecContext.hxx>
#include <Media_FormatContext.hxx>
#include <Media_Frame.hxx>
#include <Media_Packet.hxx>
#include <Media_Scaler.hxx>
#include <Media_Timer.hxx>
// The FFmpeg-free frame, packet, scaler and codec answers.
void sv_media_objects(int theCode)
{
  occ::handle<Media_Frame> aFrame = new Media_Frame(), anOther = new Media_Frame();
  ga_flag(aFrame->IsEmpty()); ga_int(aFrame->SizeX()); ga_int(aFrame->SizeY()); ga_int(aFrame->Format());
  ga_flag(aFrame->IsFullRangeYUV()); ga_flag(aFrame->Plane(0) == nullptr); ga_int(aFrame->LineSize(0));
  ga_int((int)aFrame->BestEffortTimestamp()); ga_real(aFrame->Pts()); ga_real(aFrame->PixelAspectRatio()); ga_flag(aFrame->IsLocked());
  aFrame->SetPts(2.5); aFrame->SetPixelAspectRatio(1.25f); aFrame->SetLocked(true); Media_Frame::Swap(aFrame, anOther);
  ga_real(aFrame->Pts()); ga_real(aFrame->PixelAspectRatio()); ga_flag(aFrame->IsLocked()); ga_flag(aFrame->Frame() == nullptr);
  occ::handle<Image_PixMap> aPixMap = new Image_PixMap();
  ga_flag(aFrame->InitWrapper(occ::handle<Image_PixMap>())); ga_flag(aFrame->InitWrapper(aPixMap));
  ga_int(Media_Frame::FormatFFmpeg2Occt(theCode)); ga_int(Media_Frame::FormatOcct2FFmpeg(Image_Format_RGB));
  occ::handle<Media_Packet> aPacket = new Media_Packet();
  ga_int(aPacket->Size()); ga_int((int)aPacket->Pts()); ga_int((int)aPacket->Dts()); ga_int((int)aPacket->Duration());
  ga_int(aPacket->StreamIndex()); aPacket->SetKeyFrame(); ga_flag(aPacket->IsKeyFrame()); ga_flag(aPacket->Data() == nullptr);
  aPacket->SetDurationSeconds(0.04); ga_real(aPacket->DurationSeconds());
  Media_Scaler aScaler;
  ga_flag(aScaler.Init(NCollection_Vec2<int>(4, 4), 0, NCollection_Vec2<int>(2, 2), 0)); ga_flag(aScaler.Convert(aFrame, anOther)); ga_flag(aScaler.IsValid());
  occ::handle<Media_CodecContext> aCodec = new Media_CodecContext();
  ga_int(aCodec->SizeX()); ga_int(aCodec->SizeY()); ga_int(aCodec->StreamIndex()); ga_flag(aCodec->CanProcessPacket(aPacket));
  ga_flag(aCodec->CanProcessPacket(occ::handle<Media_Packet>())); ga_flag(aCodec->SendPacket(aPacket)); ga_flag(aCodec->ReceiveFrame(aFrame));
  ga_flag(aCodec->Context() == nullptr);
  occ::handle<Media_FormatContext> aFormat = new Media_FormatContext();
  ga_int((int)aFormat->NbSteams()); GA_TRY(aFormat->Stream(0)); ga_text(aFormat->StreamInfo(0).ToCString());
  ga_flag(aFormat->ReadPacket(aPacket)); ga_flag(aFormat->SeekStream(0, 1.0, false)); ga_flag(aFormat->Seek(1.0, true));
  ga_real(aFormat->PtsStartBase()); ga_real(aFormat->Duration()); ga_flag(aFormat->Context() == nullptr);
}
// `Seconds Duration Code N Size(N)`: the time texts and conversions, a stopped timer, a buffer
// pool through the sizes, then the FFmpeg-free objects.
void sv_media_case(std::istream& input)
{
  GaReader r{input}; const double aT = r.real(), aD = r.real(); const int aCode = r.integer();
  ga_text(Media_FormatContext::FormatTime(aT).ToCString()); ga_text(Media_FormatContext::FormatTimeProgress(aT, aD).ToCString());
  ga_text(Media_FormatContext::FormatAVErrorDescription(aCode).ToCString());
  ga_real(Media_FormatContext::FormatUnitsToSeconds(aCode)); ga_int((int)Media_FormatContext::SecondsToUnits(aT));
  Media_Timer aTimer; aTimer.Seek(aT); ga_real(aTimer.ElapsedTime()); aTimer.SetPlaybackSpeed(aD); ga_real(aTimer.PlaybackSpeed());
  ga_real(aTimer.ElapsedTime()); aTimer.Pause(); ga_real(aTimer.ElapsedTime()); aTimer.Stop(); ga_real(aTimer.ElapsedTime()); ga_flag(aTimer.IsStarted());
  occ::handle<Media_BufferPool> aPool = new Media_BufferPool();
  const int aNb = r.integer();
  for (int i = 0; i < aNb; ++i) { const int aSize = r.integer(); ga_flag(aPool->Init(aSize)); ga_int(aPool->BufferSize()); }
  ga_flag(aPool->GetBuffer() == nullptr); aPool->Release(); ga_int(aPool->BufferSize());
  sv_media_objects(aCode);
  ga_end();
}
