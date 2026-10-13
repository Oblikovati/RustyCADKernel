// SPDX-License-Identifier: GPL-2.0-only
#pragma once
#include <cstdint>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>
std::vector<std::string> split(const std::string& text, char delimiter) {
  std::vector<std::string> fields;
  std::stringstream stream(text);
  std::string field;
  while (std::getline(stream, field, delimiter)) fields.push_back(field);
  return fields;
}
std::vector<double> decode(const std::string& text) {
  std::vector<double> values;
  for (const auto& field : split(text, ',')) {
    const uint64_t bits = std::stoull(field, nullptr, 16);
    double value;
    std::memcpy(&value, &bits, sizeof(value));
    values.push_back(value);
  }
  return values;
}
