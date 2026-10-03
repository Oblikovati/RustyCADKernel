#pragma once

#include "Schema.h"
#include "clang/Lex/PPCallbacks.h"
#include <set>
#include <string>
#include <utility>
#include <vector>

namespace clang {
class MacroInfo;
}

namespace api_inventory {
class PreprocessorEvents final : public clang::PPCallbacks {
public:
  PreprocessorEvents(const Config &config, clang::SourceManager &sources,
                     const clang::LangOptions &language);
  void MacroDefined(const clang::Token &name,
                    const clang::MacroDirective *directive) override;
  void SourceRangeSkipped(clang::SourceRange range,
                         clang::SourceLocation endifLocation) override;
  void InclusionDirective(clang::SourceLocation hashLocation,
                          const clang::Token &includeToken,
                          llvm::StringRef fileName,
                          bool isAngled,
                          clang::CharSourceRange filenameRange,
                          clang::OptionalFileEntryRef file,
                          llvm::StringRef searchPath,
                          llvm::StringRef relativePath,
                          const clang::Module *imported,
                          bool isModuleImport,
                          clang::SrcMgr::CharacteristicKind fileType) override;
  void emitHeaders() const;

private:
  const Config &config_;
  clang::SourceManager &sources_;
  const clang::LangOptions &language_;
  std::set<std::filesystem::path> seenHeaders_;
  std::vector<std::pair<std::string, const clang::MacroInfo *>> macros_;
};
} // namespace api_inventory
