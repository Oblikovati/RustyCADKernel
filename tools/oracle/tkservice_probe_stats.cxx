// Graphic3d_FrameStats and Graphic3d_FrameStatsData, the Rust twin of
// crates/tkservice/examples/sv_probe/stats.rs.
#include <Graphic3d_FrameStats.hxx>
#include <Graphic3d_FrameStatsData.hxx>
// The abstract statistics with an empty updateStatistics, exposing the maxima.
class SvFrameStats : public Graphic3d_FrameStats
{
public:
  Graphic3d_FrameStatsData& Max() { return myCountersMax; }
protected:
  void updateStatistics(const occ::handle<Graphic3d_CView>&, bool) override {}
};
void sv_size(size_t v){std::cout<<v<<' ';}
// Reads the 27 counters, the 5 timers and the 4 rates of a data frame.
void sv_stats_fill(GaReader& r,Graphic3d_FrameStatsDataTmp& tmp){
 for(int i=0;i<Graphic3d_FrameStatsCounter_NB;++i)tmp[(Graphic3d_FrameStatsCounter)i]=(size_t)r.real();
 for(int i=0;i<Graphic3d_FrameStatsTimer_NB;++i)tmp[(Graphic3d_FrameStatsTimer)i]=r.real();
 tmp.ChangeFrameRate()=r.real();tmp.ChangeFrameRateCpu()=r.real();
 tmp.ChangeImmediateFrameRate()=r.real();tmp.ChangeImmediateFrameRateCpu()=r.real();}
void sv_stats_print(const Graphic3d_FrameStatsData& d){
 for(int i=0;i<Graphic3d_FrameStatsCounter_NB;++i)sv_size(d[(Graphic3d_FrameStatsCounter)i]);
 for(int i=0;i<Graphic3d_FrameStatsTimer_NB;++i)ga_real(d[(Graphic3d_FrameStatsTimer)i]);
 ga_real(d.FrameRate());ga_real(d.FrameRateCpu());ga_real(d.ImmediateFrameRate());ga_real(d.ImmediateFrameRateCpu());}
// `LongLine Flags Frame(36) Max(36) NbFrames`: the text and dictionary statistics of a
// frame with the given maxima, then FlushTimers, FillMax and Reset on the data frames.
void sv_stats_case(std::istream& input){
 GaReader r{input};occ::handle<SvFrameStats> stats=new SvFrameStats();
 stats->SetLongLineFormat(r.flag());int flags=r.integer();
 sv_stats_fill(r,stats->ActiveDataFrame());stats->ChangeDataFrames().ChangeFirst()=stats->ActiveDataFrame();
 Graphic3d_FrameStatsDataTmp maxTmp;sv_stats_fill(r,maxTmp);stats->Max()=maxTmp;size_t nb=(size_t)r.real();
 ga_text(stats->FormatStats((Graphic3d_RenderingParams::PerfCounters)flags).ToCString());
 NCollection_IndexedDataMap<TCollection_AsciiString,TCollection_AsciiString> dict;
 dict.Add("Other","o");dict.Add("FPS","x");
 stats->FormatStats(dict,(Graphic3d_RenderingParams::PerfCounters)flags);
 ga_int(dict.Extent());
 for(int i=1;i<=dict.Extent();++i){ga_text(dict.FindKey(i).ToCString());ga_text(dict.FindFromIndex(i).ToCString());}
 ga_flag(stats->HasCulledLayers());ga_flag(stats->HasCulledStructs());ga_flag(stats->IsLongLineFormat());
 sv_size(stats->CounterValue(Graphic3d_FrameStatsCounter_NbLayers));ga_real(stats->TimerValue(Graphic3d_FrameStatsTimer_CpuFrame));
 ga_real(stats->FrameRate());ga_real(stats->FrameRateCpu());ga_int(stats->LastDataFrameIndex());
 ga_real(stats->UpdateInterval());ga_real(stats->FrameDuration());ga_int(stats->DataFrames().Length());
 Graphic3d_FrameStatsDataTmp& tmp=stats->ActiveDataFrame();tmp.FlushTimers(nb,false);sv_stats_print(tmp);
 tmp.FlushTimers(nb,true);sv_stats_print(tmp);
 Graphic3d_FrameStatsData filled;filled.FillMax(tmp);sv_stats_print(filled);
 filled=stats->Max();filled.FillMax(tmp);sv_stats_print(filled);
 tmp=stats->Max();sv_stats_print(tmp);tmp.Reset();sv_stats_print(tmp);filled.Reset();sv_stats_print(filled);
 ga_end();}
