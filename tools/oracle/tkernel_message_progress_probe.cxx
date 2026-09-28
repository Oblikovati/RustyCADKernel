// Message progress differential scripts: indicator, range, scope, sentry and lazy scope.
// The Rust twin is crates/tkernel/examples/oracle_probe/message_progress.rs.
//
// Request: `msgprog <allowed_polls> <start|null> <tokens...>`. The indicator breaks once
// UserBreak() was polled more than allowed_polls times (-1: never). Tokens run against the
// current range (top level: the Start() range) or the current scope; `[ ... ]` bodies nest
// and every object dies at the end of its body, so destructor side effects are compared.
//   range body:  S <kind> <max> <inf> [ scope-body ]   scope from the range (kind L S E N Z)
//                k [ range-body ]                       copy (disarms the source)
//                x  explicit Close()     r  print IsActive/UserBreak/More
//                y <min> <max> <step> <inf> <span> [ scope-body ]   deprecated sentry
//                z <max> <patch> <inf> <count>          lazy scope, Next() count times
//   scope body:  n <step> [ range-body ]   Next(step)   c  Close()   w  Show()
//                v  print Value/IsActive/MaxValue/GetPortion/IsInfinite/Name/parent Name
//                b  print UserBreak/More   a <s1> <s2>  r1=Next(s1); r2=Next(s2); r1=r2
#include <Message_LazyProgressScope.hxx>
#include <Message_ProgressIndicator.hxx>
#include <Message_ProgressScope.hxx>
#include <Message_ProgressSentry.hxx>
#include <cstdint>
#include <cstring>
#include <istream>
#include <ostream>
#include <string>
#include <vector>

namespace message_progress
{
static uint64_t real_bits(double value)
{
  uint64_t result;
  std::memcpy(&result, &value, 8);
  return result;
}

static std::string quoted(const char* name)
{
  return name == nullptr ? std::string("~") : "'" + std::string(name) + "'";
}

class RecordingIndicator : public Message_ProgressIndicator
{
public:
  explicit RecordingIndicator(int allowed) : myAllowed(allowed), myPolls(0) {}
  int                             Polls() const { return myPolls; }
  const std::vector<std::string>& Shows() const { return myShows; }

protected:
  bool UserBreak() override
  {
    ++myPolls;
    return myAllowed >= 0 && myPolls > myAllowed;
  }
  void Show(const Message_ProgressScope& theScope, const bool isForce) override
  {
    myShows.push_back(std::to_string(real_bits(GetPosition())) + ' ' + quoted(theScope.Name())
                      + ' ' + std::to_string(isForce));
  }

private:
  int                      myAllowed;
  int                      myPolls;
  std::vector<std::string> myShows;
};

struct Script
{
  std::istream& in;
  std::ostream& out;
};

static void run_range(Script& s, Message_ProgressRange& range);

static void skip_block(Script& s)
{
  std::string token;
  s.in >> token; // "["
  for (int depth = 1; depth > 0 && s.in >> token;)
    depth += (token == "[") - (token == "]");
}

static void open_block(Script& s)
{
  std::string token;
  s.in >> token; // "["
}

static void scope_value(Script& s, const Message_ProgressScope& scope)
{
  const Message_ProgressScope* parent = scope.Parent();
  s.out << "V " << real_bits(scope.Value()) << ' ' << scope.IsActive() << ' '
        << real_bits(scope.MaxValue()) << ' ' << real_bits(scope.GetPortion()) << ' '
        << scope.IsInfinite() << ' ' << quoted(scope.Name()) << ' '
        << (parent == nullptr ? std::string("-") : quoted(parent->Name())) << ' ';
}

static void run_scope(Script& s, Message_ProgressScope& scope)
{
  std::string token;
  while (s.in >> token && token != "]")
  {
    if (token == "n")
    {
      double step;
      s.in >> step;
      open_block(s);
      Message_ProgressRange range = scope.Next(step);
      run_range(s, range);
    }
    else if (token == "c")
      scope.Close();
    else if (token == "w")
      scope.Show();
    else if (token == "v")
      scope_value(s, scope);
    else if (token == "b")
      s.out << "B " << scope.UserBreak() << ' ' << scope.More() << ' ';
    else if (token == "a")
    {
      double first, second;
      s.in >> first >> second;
      Message_ProgressRange target = scope.Next(first);
      Message_ProgressRange source = scope.Next(second);
      target                       = source;
      s.out << "A " << target.IsActive() << ' ' << source.IsActive() << ' ';
    }
    else
      throw std::runtime_error("unknown scope token " + token);
  }
}

static void scope_case(Script& s, const Message_ProgressRange& range)
{
  std::string kind;
  double      max;
  int         infinite;
  s.in >> kind >> max >> infinite;
  open_block(s);
  if (kind == "L")
  {
    Message_ProgressScope scope(range, "Lit", max, infinite != 0);
    run_scope(s, scope);
  }
  else if (kind == "Z")
  {
    Message_ProgressScope scope(range, "", max, infinite != 0);
    run_scope(s, scope);
  }
  else if (kind == "N")
  {
    Message_ProgressScope scope(range, nullptr, max, infinite != 0);
    run_scope(s, scope);
  }
  else
  {
    TCollection_AsciiString name(kind == "S" ? "Str" : "");
    Message_ProgressScope   scope(range, name, max, infinite != 0);
    run_scope(s, scope);
  }
}

static void sentry_case(Script& s, const Message_ProgressRange& range)
{
  double min, max, step, span;
  int    infinite;
  s.in >> min >> max >> step >> infinite >> span;
  try
  {
    Message_ProgressSentry sentry(range, "Sen", min, max, step, infinite != 0, span);
    open_block(s);
    run_scope(s, sentry);
  }
  catch (const Standard_Failure& error)
  {
    s.out << "E " << error.ExceptionType() << ' ';
    skip_block(s);
  }
}

static void lazy_case(Script& s, const Message_ProgressRange& range)
{
  double max, patch;
  int    infinite, count;
  s.in >> max >> patch >> infinite >> count;
  Message_LazyProgressScope lazy(range, "Lazy", max, patch, infinite != 0);
  for (int i = 0; i < count; ++i)
    lazy.Next();
  const bool more = lazy.More();
  s.out << "Z " << more << ' ' << lazy.IsAborted() << ' ' << lazy.More() << ' ';
}

static void run_range(Script& s, Message_ProgressRange& range)
{
  std::string token;
  while (s.in >> token && token != "]")
  {
    if (token == "S")
      scope_case(s, range);
    else if (token == "k")
    {
      open_block(s);
      Message_ProgressRange copy(range);
      run_range(s, copy);
    }
    else if (token == "x")
      range.Close();
    else if (token == "r")
      s.out << "R " << range.IsActive() << ' ' << range.UserBreak() << ' ' << range.More() << ' ';
    else if (token == "y")
      sentry_case(s, range);
    else if (token == "z")
      lazy_case(s, range);
    else
      throw std::runtime_error("unknown range token " + token);
  }
}
} // namespace message_progress

static bool message_progress_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op != "msgprog")
    return false;
  int         allowed;
  std::string mode;
  in >> allowed >> mode;
  occ::handle<message_progress::RecordingIndicator> indicator =
    new message_progress::RecordingIndicator(allowed);
  message_progress::Script script{in, out};
  {
    Message_ProgressRange range = Message_ProgressIndicator::Start(
      mode == "null" ? occ::handle<Message_ProgressIndicator>() : indicator);
    message_progress::run_range(script, range);
  }
  out << "P " << indicator->Polls() << ' ' << indicator->Shows().size() << ' ';
  for (const auto& show : indicator->Shows())
    out << show << ' ';
  out << message_progress::real_bits(indicator->GetPosition());
  return true;
}
