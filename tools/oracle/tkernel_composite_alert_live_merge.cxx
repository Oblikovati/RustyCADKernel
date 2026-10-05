#include <Message_Alert.hxx>
#include <Message_CompositeAlerts.hxx>
#include <Message_Gravity.hxx>

#include <iostream>
#include <string>
#include <vector>

class LiveMergeAlert : public Message_Alert
{
public:
  LiveMergeAlert(Message_CompositeAlerts* theComposite,
                 const std::string& theLabel,
                 bool theCanMerge,
                 bool theAppendOnSupports,
                 bool theAppendOnFirstMerge,
                 bool theRemoveFirstOnDynamicType,
                 const std::string& theSucceedOn,
                 std::vector<std::string>* theEvents,
                 int* theDynamicCalls,
                 const occ::handle<Message_Alert>& theRemoveTarget = {})
      : myComposite(theComposite),
        myLabel(theLabel),
        myCanMerge(theCanMerge),
        myAppendOnSupports(theAppendOnSupports),
        myAppendOnFirstMerge(theAppendOnFirstMerge),
        myRemoveFirstOnDynamicType(theRemoveFirstOnDynamicType),
        mySucceedOn(theSucceedOn),
        myEvents(theEvents),
        myDynamicCalls(theDynamicCalls),
        myRemoveTarget(theRemoveTarget)
  {
  }

  bool SupportsMerge() const override
  {
    if (myAppendOnSupports)
      append_target("support-target");
    return myCanMerge;
  }

  bool Merge(const occ::handle<Message_Alert>& theTarget) override
  {
    occ::handle<LiveMergeAlert> aTarget = occ::down_cast<LiveMergeAlert>(theTarget);
    myEvents->push_back(aTarget->myLabel);
    if (myAppendOnFirstMerge && aTarget->myLabel == "first")
      append_target("late-target");
    return aTarget->myLabel == mySucceedOn;
  }

  static constexpr const char* get_type_name() { return "LiveMergeAlert"; }

  static const occ::handle<Standard_Type>& get_type_descriptor()
  {
    static const occ::handle<Standard_Type> aType =
      Standard_Type::Register(typeid(LiveMergeAlert),
                              get_type_name(),
                              sizeof(LiveMergeAlert),
                              Message_Alert::get_type_descriptor());
    return aType;
  }

  const occ::handle<Standard_Type>& DynamicType() const override
  {
    ++*myDynamicCalls;
    if (myRemoveFirstOnDynamicType && !myRemoveTarget.IsNull())
      myComposite->RemoveAlert(Message_Info, myRemoveTarget);
    return get_type_descriptor();
  }

private:
  void append_target(const std::string& theLabel) const
  {
    myComposite->AddAlert(Message_Info,
                          new LiveMergeAlert(myComposite,
                                             theLabel,
                                             false,
                                             false,
                                             false,
                                             false,
                                             "",
                                             myEvents,
                                             myDynamicCalls));
  }

  Message_CompositeAlerts* myComposite;
  std::string myLabel;
  bool myCanMerge;
  bool myAppendOnSupports;
  bool myAppendOnFirstMerge;
  bool myRemoveFirstOnDynamicType;
  std::string mySucceedOn;
  std::vector<std::string>* myEvents;
  int* myDynamicCalls;
  occ::handle<Message_Alert> myRemoveTarget;
};

static void print_events(const std::vector<std::string>& theEvents)
{
  for (size_t anIndex = 0; anIndex < theEvents.size(); ++anIndex)
    std::cout << (anIndex == 0 ? "" : ",") << theEvents[anIndex];
}

static void supports_append_case()
{
  occ::handle<Message_CompositeAlerts> aComposite = new Message_CompositeAlerts;
  aComposite->AddAlert(Message_Info, new Message_Alert);
  std::vector<std::string> anEvents;
  int aDynamicCalls = 0;
  occ::handle<LiveMergeAlert> anIncoming = new LiveMergeAlert(
    aComposite.get(), "incoming", true, true, false, false, "support-target", &anEvents, &aDynamicCalls);
  const bool isAdded = aComposite->AddAlert(Message_Info, anIncoming);
  std::cout << "supports_append:events=";
  print_events(anEvents);
  std::cout << ";added=" << isAdded << ";size=" << aComposite->Alerts(Message_Info).Size()
            << ";has_incoming=" << aComposite->HasAlert(anIncoming)
            << ";dynamic_calls=" << aDynamicCalls << '\n';
}

static void merge_append_case()
{
  occ::handle<Message_CompositeAlerts> aComposite = new Message_CompositeAlerts;
  std::vector<std::string> anEvents;
  int aDynamicCalls = 0;
  occ::handle<LiveMergeAlert> aFirst = new LiveMergeAlert(
    aComposite.get(), "first", false, false, false, false, "", &anEvents, &aDynamicCalls);
  aComposite->AddAlert(Message_Info, aFirst);
  occ::handle<LiveMergeAlert> anIncoming = new LiveMergeAlert(
    aComposite.get(), "incoming", true, false, true, false, "late-target", &anEvents, &aDynamicCalls);
  const bool isAdded = aComposite->AddAlert(Message_Info, anIncoming);
  std::cout << "merge_append:events=";
  print_events(anEvents);
  std::cout << ";added=" << isAdded << ";size=" << aComposite->Alerts(Message_Info).Size()
            << ";has_incoming=" << aComposite->HasAlert(anIncoming)
            << ";dynamic_calls=" << aDynamicCalls << '\n';
}

static void empty_list_case()
{
  occ::handle<Message_CompositeAlerts> aComposite = new Message_CompositeAlerts;
  std::vector<std::string> anEvents;
  int aDynamicCalls = 0;
  occ::handle<LiveMergeAlert> anIncoming = new LiveMergeAlert(
    aComposite.get(), "incoming", true, false, false, false, "", &anEvents, &aDynamicCalls);
  const bool isAdded = aComposite->AddAlert(Message_Info, anIncoming);
  std::cout << "empty_list:added=" << isAdded << ";size="
            << aComposite->Alerts(Message_Info).Size() << ";dynamic_calls=" << aDynamicCalls
            << '\n';
}

static void dynamic_type_removal_case()
{
  occ::handle<Message_CompositeAlerts> aComposite = new Message_CompositeAlerts;
  std::vector<std::string> anEvents;
  int aDynamicCalls = 0;
  occ::handle<LiveMergeAlert> aFirst = new LiveMergeAlert(
    aComposite.get(), "first", false, false, false, false, "", &anEvents, &aDynamicCalls);
  occ::handle<LiveMergeAlert> aSecond = new LiveMergeAlert(
    aComposite.get(), "second", false, false, false, false, "", &anEvents, &aDynamicCalls);
  aComposite->AddAlert(Message_Info, aFirst);
  aComposite->AddAlert(Message_Info, aSecond);
  occ::handle<LiveMergeAlert> anIncoming = new LiveMergeAlert(aComposite.get(),
                                                              "incoming",
                                                              true,
                                                              false,
                                                              false,
                                                              true,
                                                              "first",
                                                              &anEvents,
                                                              &aDynamicCalls,
                                                              aFirst);
  const bool isAdded = aComposite->AddAlert(Message_Info, anIncoming);
  std::cout << "dynamic_type_removal:events=";
  print_events(anEvents);
  std::cout << ";added=" << isAdded << ";size=" << aComposite->Alerts(Message_Info).Size()
            << ";has_incoming=" << aComposite->HasAlert(anIncoming)
            << ";dynamic_calls=" << aDynamicCalls << '\n';
}

int main()
{
  supports_append_case();
  merge_append_case();
  empty_list_case();
  dynamic_type_removal_case();
}
