// Calls the unchanged native pool; asynchronous outcomes are observed only after WaitThreads.
#include <SelectMgr_BVHThreadPool.hxx>
#include <Message.hxx>
#include <Message_Printer.hxx>
#include <atomic>
#include <algorithm>
#include <thread>
#include <condition_variable>
class V3PoolJob : public Select3D_SensitiveEntity {
public:
  std::atomic<int> calls{0}; bool buildable=true; int outcome;
  V3PoolJob(int result):Select3D_SensitiveEntity(nullptr),outcome(result){}
  bool Matches(SelectBasics_SelectingVolumeManager&,SelectBasics_PickResult&) override {return false;}
  int NbSubElements() const override {return 1;}
  Select3D_BndBox3d BoundingBox() override {return {};}
  gp_Pnt CenterOfGeometry() const override {return {};}
  bool ToBuildBVH() const override {return buildable;}
  void BVH() override {
    ++calls;
    if(outcome==1)throw Standard_Failure("fixture failure");
    if(outcome==2)throw 7;
  }
};
class V3PoolPrinter : public Message_Printer {
public:
  mutable std::mutex mutex; mutable std::vector<std::string> messages;
protected:
  void send(const TCollection_AsciiString& text,const Message_Gravity) const override {
    std::lock_guard<std::mutex> lock(mutex);messages.push_back(text.ToCString());
  }
};
// The pinned pool has a shutdown/empty-queue wake race. Park every worker in
// a BVH call before stopping, so the unchanged native StopThreads is observable.
class V3ParkedPoolJob : public V3PoolJob {
public:
  std::mutex mutex;std::condition_variable changed;int entered=0;bool released=false;
  V3ParkedPoolJob():V3PoolJob(0){}
  void BVH() override {
    std::unique_lock<std::mutex> lock(mutex);++entered;changed.notify_all();
    changed.wait(lock,[&](){return released;});
  }
  void waitEntered(int count){std::unique_lock<std::mutex> lock(mutex);changed.wait(lock,[&](){return entered==count;});}
  void release(){std::lock_guard<std::mutex> lock(mutex);released=true;changed.notify_all();}
};
void v3_pool_state(const occ::handle<SelectMgr_BVHThreadPool>& pool,const std::vector<occ::handle<V3PoolJob>>& jobs);
class V3ObservablePool : public SelectMgr_BVHThreadPool {
public:
  V3ObservablePool(int count):SelectMgr_BVHThreadPool(count){}
  void stopWithParkedWorkers(const std::vector<occ::handle<V3PoolJob>>& jobs){
    if(!myIsStarted){v3_pool_state(this,jobs);StopThreads();return;}
    occ::handle<V3ParkedPoolJob> parked=new V3ParkedPoolJob();
    for(int i=1;i<=Threads().Size();++i)AddEntity(parked);
    parked->waitEntered(Threads().Size());v3_pool_state(this,jobs);myWakeEvent.Reset();
    std::thread stopper([&](){StopThreads();});
    myWakeEvent.Wait();parked->release();stopper.join();
  }
};
void v3_pool_state(const occ::handle<SelectMgr_BVHThreadPool>& pool,const std::vector<occ::handle<V3PoolJob>>& jobs){
  ga_int(pool->Threads().Lower());ga_int(pool->Threads().Upper());
  for(const auto& job:jobs)ga_int(job->calls.load());
  for(auto& worker:pool->Threads())ga_flag(worker.GetId()!=0);
}
void v3_bvh_thread_pool(std::istream& input){
  GaReader r{input};int threads=r.integer(),count=r.integer(),rounds=r.integer(),seed=r.integer();
  auto messenger=Message::DefaultMessenger();auto saved=messenger->Printers();
  occ::handle<V3PoolPrinter> printer=new V3PoolPrinter();messenger->ChangePrinters().Clear();messenger->AddPrinter(printer);
  occ::handle<V3ObservablePool> pool=new V3ObservablePool(threads);
  std::vector<occ::handle<V3PoolJob>> jobs;
  for(int i=0;i<count;++i){occ::handle<V3PoolJob> job=new V3PoolJob((i+seed)%3);job->buildable=(i+seed)%4!=0;jobs.push_back(job);}
  v3_pool_state(pool,jobs);
  for(int round=0;round<rounds;++round){
    {SelectMgr_BVHThreadPool::Sentry lock(pool);std::vector<bool> acquired;
      std::thread observer([&](){for(auto& worker:pool->Threads()){bool ok=worker.BVHMutex().try_lock();acquired.push_back(ok);if(ok)worker.BVHMutex().unlock();}});
      observer.join();for(bool ok:acquired)ga_flag(ok);}
    for(int i=0;i<count*2;++i)pool->AddEntity(jobs[(i+round)%count]);
    pool->WaitThreads();pool->stopWithParkedWorkers(jobs);v3_pool_state(pool,jobs);
  }
  std::sort(printer->messages.begin(),printer->messages.end());ga_int(int(printer->messages.size()));
  for(const auto& message:printer->messages)ga_text(message);
  messenger->ChangePrinters()=saved;ga_end();
}
