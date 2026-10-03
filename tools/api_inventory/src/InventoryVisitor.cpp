#include "InventoryVisitor.h"
#include "clang/AST/ASTContext.h"

namespace api_inventory {
InventoryVisitor::InventoryVisitor(clang::ASTContext &context,
                                   const Config &config)
    : context_(context), config_(config) {}

bool InventoryVisitor::VisitDecl(clang::Decl *declaration) {
  const auto &sources = context_.getSourceManager();
  const auto location = sources.getExpansionLoc(declaration->getLocation());
  if (location.isInvalid()) return true;
  const auto path = canonicalPath(sources.getFilename(location).str());
  if (path != config_.mainHeader && !isUnder(path, config_.sourceRoot)) {
    ++externalDecls_;
    return true;
  }
  if (!shouldEmit(path, config_)) {
    ++occtUnemittedDecls_;
    return true;
  }
  const std::string kind = declaration->getDeclKindName();
  const auto event = declarationEvent(*declaration, context_, config_);
  const auto visible = event.getBoolean("api_visible").value_or(false);
  const auto reason = event.getString("exclusion_reason").value_or("unhandled_decl_kind");
  ++kinds_[kind];
  ++dispositions_[visible ? "candidate" : reason.str()];
  if (!visible && reason == "unhandled_decl_kind") unhandledKinds_.insert(kind);
  ++emitted_;
  writeEvent(event);
  return true;
}

void InventoryVisitor::emitSummary() {
  llvm::json::Object kinds;
  for (const auto &[kind, count] : kinds_) kinds[kind] = static_cast<int64_t>(count);
  llvm::json::Object dispositions;
  for (const auto &[reason, count] : dispositions_)
    dispositions[reason] = static_cast<int64_t>(count);
  llvm::json::Array unhandled;
  for (const auto &kind : unhandledKinds_) unhandled.push_back(kind);
  writeEvent(llvm::json::Object{
      {"event", "summary"},
      {"emitted_declarations", static_cast<int64_t>(emitted_)},
      {"external_declarations_excluded", static_cast<int64_t>(externalDecls_)},
      {"occt_declarations_outside_emit_scope", static_cast<int64_t>(occtUnemittedDecls_)},
      {"declarations_by_kind", std::move(kinds)},
      {"declarations_by_disposition", std::move(dispositions)},
      {"unhandled_decl_kinds", std::move(unhandled)},
      {"complete_for_emit_scope", unhandledKinds_.empty()}});
}

bool InventoryVisitor::hasUnhandledKinds() const { return !unhandledKinds_.empty(); }
} // namespace api_inventory
