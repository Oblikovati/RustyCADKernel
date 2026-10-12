#pragma once
namespace retirement_fixture {
template <template <class...> class Container>
struct ActiveTemplateParameter {};
[[deprecated("Use activeFunction")]] inline void retiredFunction() {}
inline void activeFunction() {}
struct [[deprecated("Use ActiveRecord")]] RetiredRecord {
  void inheritedMember() {}
};
template <class T> struct [[deprecated("Use ActiveTemplate")]] RetiredTemplate {
  void inheritedMember() {}
};
//! Obsolete class retained for compatibility.
struct ObsoleteRecord {
  void inheritedMember() {}
};
struct ActiveRecord {
  //! This is obsolete method for backward compatibility.
  void obsoleteMethod() {}
  //! @param ignored obsolete argument that must be ignored.
  void activeWithOldParameter(int ignored) { (void)ignored; }
  //! Marks cached bounds as obsolete.
  void invalidateBounds() {}
};
enum Mode {
  ActiveMode,
  RetiredMode [[deprecated("Use ActiveMode")]],
  ObsoleteMode //!< Obsolete alias retained for compatibility.
};
} // namespace retirement_fixture
