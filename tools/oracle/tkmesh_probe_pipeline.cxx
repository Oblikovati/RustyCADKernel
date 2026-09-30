// Controlled native strategies exercise orchestration, not meshing approximations.
#include <IMeshTools_Context.hxx>
#include <IMeshTools_MeshBuilder.hxx>
#include <BRepMeshData_Model.hxx>
#include <Message_ProgressIndicator.hxx>
#include <array>
class RecordingModelBuilder : public IMeshTools_ModelBuilder {
public:
  RecordingModelBuilder(int outcome, std::vector<int>& calls) : myOutcome(outcome), myCalls(calls) {}
protected:
  occ::handle<IMeshData_Model> performInternal(const TopoDS_Shape& shape, const IMeshTools_Parameters&) override {
    myCalls.push_back(0);
    if (myOutcome == -2) throw Standard_Failure("deliberate model construction failure");
    if (myOutcome == -1) { SetStatus(Message_Fail1); return nullptr; }
    if (myOutcome == 0) return nullptr;
    SetStatus(Message_Done1); return new BRepMeshData_Model(shape);
  }
private:
  int myOutcome; std::vector<int>& myCalls;
};
class RecordingModelStage : public IMeshTools_ModelAlgo {
public:
  RecordingModelStage(int outcome, int index, std::vector<int>& calls):myOutcome(outcome),myIndex(index),myCalls(calls) {}
protected:
  bool performInternal(const occ::handle<IMeshData_Model>&,const IMeshTools_Parameters&,const Message_ProgressRange& range) override {
    myCalls.push_back(myIndex);myCalls.push_back(range.IsActive());
    if (myOutcome < 0) throw Standard_Failure("deliberate model stage failure");
    return myOutcome != 0;
  }
private:
  int myOutcome, myIndex; std::vector<int>& myCalls;
};
class PipelineProgress : public Message_ProgressIndicator {
public:
  bool cancelled; std::vector<double> positions;
  explicit PipelineProgress(bool cancel):cancelled(cancel) {}
  bool UserBreak() override { return cancelled; }
protected:
  void Show(const Message_ProgressScope&, const bool) override { positions.push_back(GetPosition()); }
};
occ::handle<IMeshTools_Context> mesh_pipeline_context(int builderOutcome,const std::array<int,5>& stages,bool clean,std::vector<int>& calls) {
  occ::handle<IMeshTools_Context> context=new IMeshTools_Context;
  context->ChangeParameters().CleanModel=clean;
  if (builderOutcome!=-3) context->SetModelBuilder(new RecordingModelBuilder(builderOutcome,calls));
  std::array<occ::handle<IMeshTools_ModelAlgo>,5> algorithms;
  for(int i=0;i<5;++i) if(stages[i]!=-2) algorithms[i]=new RecordingModelStage(stages[i],i+1,calls);
  context->SetEdgeDiscret(algorithms[0]);context->SetModelHealer(algorithms[1]);context->SetPreProcessor(algorithms[2]);
  context->SetFaceDiscret(algorithms[3]);context->SetPostProcessor(algorithms[4]);return context;
}
void mesh_pipeline(std::istream& input) {
  int builderOutcome,clean,cancel;std::array<int,5> stages;input>>builderOutcome;for(auto& outcome:stages)input>>outcome;input>>clean>>cancel;
  std::vector<int> calls;auto context=mesh_pipeline_context(builderOutcome,stages,clean,calls);
  occ::handle<IMeshTools_MeshBuilder> builder=new IMeshTools_MeshBuilder;
  if(builderOutcome!=-4)builder->SetContext(context);
  occ::handle<PipelineProgress> progress=new PipelineProgress(cancel);builder->SetStatus(Message_Warn3);builder->Perform(progress->Start());
  for(auto status:{Message_Done1,Message_Warn1,Message_Warn3,Message_Fail1,Message_Fail2,Message_Fail3,Message_Fail4,Message_Fail5,Message_Fail6,Message_Fail7,Message_Fail8})ga_flag(builder->GetStatus().IsSet(status));
  ga_flag(context->GetModel().IsNull());ga_flag(builder->GetContext().IsNull());ga_flag(occ::down_cast<Message_Algorithm>(builder).IsNull());ga_flag(occ::down_cast<IMeshTools_Context>(context).IsNull());
  ga_int(calls.size());for(auto value:calls)ga_int(value);ga_int(progress->positions.size());for(auto value:progress->positions)ga_real(value);ga_end();
}
