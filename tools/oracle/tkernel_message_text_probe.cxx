// Message text operations: Message_Msg, Message_MsgFile, Message::FillTime, metric names
// and Message_ExecStatus (twin of oracle_probe/message/text.rs).
//
// msg_run <text> <step>...      Set(text), then s:<bytes> a:<bytes> e:<units> i:<int>
//                               r:<bits> g ; prints value/edited after each step
// msg_set_c <bytes>             Set(const char*) then value/original
// msg_fill_time <h> <m> <bits>  FillTime
// msg_metrics                   every metric name, and the OSD counter mapping
// msg_metric_from <bytes>       MetricFromString (both overloads)
// msg_exec_ops <step>...        s<v> c<v> D W A F d w a f x |<v> &<v> on one ExecStatus
// msg_exec_static <v>...        StatusIndex/LocalStatusIndex/TypeOfStatus of raw values
// msg_exec_by_index <lo> <hi>   StatusByIndex over a range
// msgfile_string <bytes> <length> <key>...   LoadFromString, then HasMsg/Msg per key
// msgfile_file <path> <bytes> <key>...       writes the file, LoadFile, keys
// msgfile_load <dirs> <file> <key>...        Load(dirs, file), keys
// msgfile_env <var> <value|-> <lang|-|N> <file> <key>...  setenv, LoadFromEnv, keys
#include <Message.hxx>
#include <Message_ExecStatus.hxx>
#include <Message_Msg.hxx>
#include <Message_MsgFile.hxx>

#include <cstdlib>
#include <filesystem>
#include <functional>
#include <fstream>

static void msgp_msg_state(std::ostream& out, const Message_Msg& theMsg)
{
  out << ' ' << msgp_units(theMsg.Value()) << '/' << theMsg.IsEdited();
}

static void msgp_msg_step(Message_Msg& theMsg, const std::string& theStep, std::ostream& out)
{
  const std::string aValue = theStep.size() > 2 ? theStep.substr(2) : std::string();
  switch (theStep[0])
  {
    case 's':
      theMsg.Arg(msgp_bytes(aValue).c_str());
      break;
    case 'a':
      theMsg.Arg(TCollection_AsciiString(msgp_bytes(aValue).c_str()));
      break;
    case 'e':
      theMsg.Arg(msgp_ext(aValue));
      break;
    case 'i':
      theMsg.Arg(std::stoi(aValue));
      break;
    case 'r':
      theMsg.Arg(msgp_real(aValue));
      break;
    default:
      out << ' ' << msgp_units(theMsg.Get());
      break;
  }
}

static void msgp_msg_run(std::istream& in, std::ostream& out)
{
  std::string aText;
  in >> aText;
  Message_Msg aMsg;
  aMsg.Set(msgp_ext(aText));
  out << msgp_units(aMsg.Original());
  for (const std::string& aStep : msgp_rest(in))
  {
    try
    {
      msgp_msg_step(aMsg, aStep, out);
    }
    catch (const Standard_Failure& theFailure)
    {
      out << " E" << theFailure.ExceptionType();
      msgp_msg_state(out, aMsg);
      return;
    }
    msgp_msg_state(out, aMsg);
  }
}

static void msgp_metrics(std::ostream& out)
{
  for (int aMetric = Message_MetricType_None; aMetric <= Message_MetricType_MemHeapUsage; ++aMetric)
  {
    OSD_MemInfo::Counter aCounter = OSD_MemInfo::MemCounter_NB;
    const bool aToOsd = Message::ToOSDMetric((Message_MetricType)aMetric, aCounter);
    out << ' ' << Message::MetricToString((Message_MetricType)aMetric) << ':' << aToOsd << ':'
        << (aToOsd ? int(aCounter) : -1);
  }
  for (int aCounter = OSD_MemInfo::MemPrivate; aCounter < OSD_MemInfo::MemCounter_NB; ++aCounter)
  {
    Message_MetricType aMetric = Message_MetricType_None;
    const bool aToMessage = Message::ToMessageMetric((OSD_MemInfo::Counter)aCounter, aMetric);
    out << ' ' << aToMessage << ':' << int(aMetric);
  }
}

static void msgp_metric_from(std::istream& in, std::ostream& out)
{
  std::string aText;
  in >> aText;
  const std::string  aName    = msgp_bytes(aText);
  Message_MetricType aMetric  = Message_MetricType_None;
  const bool         isKnown  = Message::MetricFromString(aName.c_str(), aMetric);
  out << isKnown << ' ' << int(aMetric) << ' ' << int(Message::MetricFromString(aName.c_str()));
}

static void msgp_exec_step(Message_ExecStatus& theStatus, const std::string& theStep)
{
  const int aValue = theStep.size() > 1 ? std::stoi(theStep.substr(1)) : 0;
  switch (theStep[0])
  {
    case 's': theStatus.Set((Message_Status)aValue); break;
    case 'c': theStatus.Clear((Message_Status)aValue); break;
    case 'D': theStatus.SetAllDone(); break;
    case 'W': theStatus.SetAllWarn(); break;
    case 'A': theStatus.SetAllAlarm(); break;
    case 'F': theStatus.SetAllFail(); break;
    case 'd': theStatus.ClearAllDone(); break;
    case 'w': theStatus.ClearAllWarn(); break;
    case 'a': theStatus.ClearAllAlarm(); break;
    case 'f': theStatus.ClearAllFail(); break;
    case 'x': theStatus.Clear(); break;
    case '|': theStatus |= Message_ExecStatus((Message_Status)aValue); break;
    case '&': theStatus &= Message_ExecStatus((Message_Status)aValue); break;
    case '+': theStatus.Add(Message_ExecStatus((Message_Status)aValue)); break;
    default: theStatus.And(Message_ExecStatus((Message_Status)aValue)); break;
  }
}

static void msgp_exec_ops(std::istream& in, std::ostream& out)
{
  Message_ExecStatus aStatus;
  for (const std::string& aStep : msgp_rest(in))
  {
    msgp_exec_step(aStatus, aStep);
  }
  out << aStatus.IsDone() << aStatus.IsWarn() << aStatus.IsAlarm() << aStatus.IsFail() << ' ';
  for (int anIndex = Message_ExecStatus::FirstStatus; anIndex < Message_ExecStatus::LastStatus; ++anIndex)
  {
    out << aStatus.IsSet(Message_ExecStatus::StatusByIndex(anIndex));
  }
}

static void msgp_exec_static(std::istream& in, std::ostream& out)
{
  for (const std::string& aToken : msgp_rest(in))
  {
    const Message_Status aStatus = (Message_Status)std::stoi(aToken);
    out << ' ' << Message_ExecStatus::StatusIndex(aStatus) << ':'
        << Message_ExecStatus::LocalStatusIndex(aStatus) << ':'
        << int(Message_ExecStatus::TypeOfStatus(aStatus));
  }
}

static void msgp_exec_by_index(std::istream& in, std::ostream& out)
{
  int aLow = 0, aHigh = 0;
  in >> aLow >> aHigh;
  for (int anIndex = aLow; anIndex <= aHigh; ++anIndex)
  {
    out << ' ' << int(Message_ExecStatus::StatusByIndex(anIndex));
  }
}

static void msgp_keys(std::istream& in, std::ostream& out)
{
  for (const std::string& aKey : msgp_rest(in))
  {
    const TCollection_AsciiString aName(msgp_bytes(aKey).c_str());
    out << ' ' << Message_MsgFile::HasMsg(aName);
    out << ':' << msgp_units(Message_MsgFile::Msg(aName));
  }
}

static void msgp_load_guarded(std::ostream& out, const std::function<bool()>& theLoad)
{
  try
  {
    out << theLoad();
  }
  catch (const Standard_Failure& theFailure)
  {
    out << 'E' << theFailure.ExceptionType();
  }
}

static void msgp_msgfile_string(std::istream& in, std::ostream& out)
{
  std::string aContent;
  int         aLength = -1;
  in >> aContent >> aLength;
  const std::string aBytes = msgp_bytes(aContent);
  msgp_load_guarded(out, [&] { return Message_MsgFile::LoadFromString(aBytes.c_str(), aLength); });
  msgp_keys(in, out);
}

static void msgp_msgfile_file(std::istream& in, std::ostream& out)
{
  std::string aPath, aContent;
  in >> aPath >> aContent;
  const std::string aFile = msgp_bytes(aPath);
  std::filesystem::create_directories(std::filesystem::path(aFile).parent_path());
  std::ofstream(aFile, std::ios::binary) << msgp_bytes(aContent);
  msgp_load_guarded(out, [&] { return Message_MsgFile::LoadFile(aFile.c_str()); });
  msgp_keys(in, out);
}

static void msgp_msgfile_load(std::istream& in, std::ostream& out)
{
  std::string aDirs, aFile;
  in >> aDirs >> aFile;
  const std::string aDirList = msgp_bytes(aDirs), aName = msgp_bytes(aFile);
  msgp_load_guarded(out, [&] { return Message_MsgFile::Load(aDirList.c_str(), aName.c_str()); });
  msgp_keys(in, out);
}

static void msgp_msgfile_env(std::istream& in, std::ostream& out)
{
  std::string aVar, aValue, aLang, aFile;
  in >> aVar >> aValue >> aLang >> aFile;
  if (aValue == "-")
  {
    unsetenv(aVar.c_str());
  }
  else
  {
    setenv(aVar.c_str(), msgp_bytes(aValue).c_str(), 1);
  }
  const std::string aLangText = msgp_bytes(aLang == "N" ? "-" : aLang), aName = msgp_bytes(aFile);
  const char* aLangPtr = aLang == "N" ? nullptr : aLangText.c_str();
  msgp_load_guarded(out, [&] { return Message_MsgFile::LoadFromEnv(aVar.c_str(), aName.c_str(), aLangPtr); });
  msgp_keys(in, out);
}

static bool message_text_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "msg_run") msgp_msg_run(in, out);
  else if (op == "msg_set_c")
  {
    std::string aText;
    in >> aText;
    Message_Msg aMsg;
    aMsg.Set(msgp_bytes(aText).c_str());
    out << msgp_units(aMsg.Value()) << ' ' << msgp_units(aMsg.Original());
  }
  else if (op == "msg_fill_time")
  {
    int         anHour = 0, aMinute = 0;
    std::string aSecond;
    in >> anHour >> aMinute >> aSecond;
    out << msgp_ascii(Message::FillTime(anHour, aMinute, msgp_real(aSecond)));
  }
  else if (op == "msg_metrics") msgp_metrics(out);
  else if (op == "msg_metric_from") msgp_metric_from(in, out);
  else if (op == "msg_exec_ops") msgp_exec_ops(in, out);
  else if (op == "msg_exec_static") msgp_exec_static(in, out);
  else if (op == "msg_exec_by_index") msgp_exec_by_index(in, out);
  else if (op == "msgfile_string") msgp_msgfile_string(in, out);
  else if (op == "msgfile_file") msgp_msgfile_file(in, out);
  else if (op == "msgfile_load") msgp_msgfile_load(in, out);
  else if (op == "msgfile_env") msgp_msgfile_env(in, out);
  else return false;
  return true;
}
