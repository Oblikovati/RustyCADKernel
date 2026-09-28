// Differential oracle driver. OCCT is used only by this test executable.
#include <Quantity_Date.hxx>
#include <Quantity_Period.hxx>
#include <Standard_GUID.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <sstream>
#include <string>

#include "tkernel_ncollection_probe.cxx"
#include "tkernel_message_probe.cxx"
#include "tkernel_quantity_probe.cxx"
#include "tkernel_resource_probe.cxx"
#include "tkernel_message_progress_probe.cxx"
#include "tkernel_units_probe.cxx"
#include "tkernel_storage_probe.cxx"

static Quantity_Date date(std::istream& in) {
  int mm, dd, yy, hh, mn, ss, mis, mics;
  if (!(in >> mm >> dd >> yy >> hh >> mn >> ss >> mis >> mics)) throw std::runtime_error("invalid date input");
  return Quantity_Date(mm, dd, yy, hh, mn, ss, mis, mics);
}
static void date_out(std::ostream& out, Quantity_Date value) {
  int mm, dd, yy, hh, mn, ss, mis, mics;
  value.Values(mm, dd, yy, hh, mn, ss, mis, mics);
  out << mm << ' ' << dd << ' ' << yy << ' ' << hh << ' ' << mn << ' ' << ss << ' ' << mis << ' ' << mics;
}
static void period_out(std::ostream& out, Quantity_Period value) {
  int ss, us; value.Values(ss, us); out << ss << ' ' << us;
}
int main() {
  std::string line;
  while (std::getline(std::cin, line)) {
    std::istringstream in(line);
    std::ostringstream out;
    std::string op; in >> op;
    try {
      if (op == "period") {
        int dd, hh, mn, ss, mis, us;
        if (!(in >> dd >> hh >> mn >> ss >> mis >> us)) return 2;
        Quantity_Period p(dd, hh, mn, ss, mis, us);
        period_out(out, p);
        p.Values(dd, hh, mn, ss, mis, us);
        out << ' ' << dd << ' ' << hh << ' ' << mn << ' ' << ss << ' ' << mis << ' ' << us;
      } else if (op == "period_math") {
        int s1,u1,s2,u2; if (!(in >> s1 >> u1 >> s2 >> u2)) return 2;
        Quantity_Period a(s1,u1), b(s2,u2);
        period_out(out, a.Add(b)); out << ' '; period_out(out, a.Subtract(b));
        out << ' ' << a.IsEqual(b) << ' ' << a.IsShorter(b) << ' ' << a.IsLonger(b);
      } else if (op == "date") {
        auto a = date(in); date_out(out, a);
        out << ' '; period_out(out, a.Difference(Quantity_Date()));
      } else if (op == "date_add" || op == "date_sub") {
        auto a = date(in); int ss, us; if (!(in >> ss >> us)) return 2;
        Quantity_Period p(ss,us); date_out(out, op == "date_add" ? a.Add(p) : a.Subtract(p));
      } else if (op == "date_diff") {
        auto a = date(in); auto b = date(in); period_out(out, a.Difference(b));
        out << ' ' << a.IsEqual(b) << ' ' << a.IsEarlier(b) << ' ' << a.IsLater(b);
      } else if (op == "guid") {
        std::string value; if (!(in >> value)) return 2;
        if (value == "EMPTY") value = "";
        Standard_GUID guid(value.c_str()); char text[37]; guid.ToCString(text);
        const auto uuid = guid.ToUUID(); out << text << ' ' << uuid.Data1 << ' ' << uuid.Data2 << ' ' << uuid.Data3;
        for (auto byte : uuid.Data4) out << ' ' << static_cast<unsigned>(byte);
      } else if (!ncollection_probe(op, in, out) && !storage_probe(op, in, out) && !message_probe(op, in, out) && !quantity_probe(op, in, out) && !resource_probe(op, in, out) && !message_progress_probe(op, in, out) && !units_probe(op, in, out)) { return 2; }
      std::cout << "OK " << out.str() << '\n';
    } catch (const Standard_Failure& error) {
      std::cout << "ERR " << error.ExceptionType() << ": " << error.what() << '\n';
    } catch (const std::exception& error) {
      std::cerr << error.what() << '\n'; return 2;
    }
  }
}
