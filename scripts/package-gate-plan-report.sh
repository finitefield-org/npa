#!/usr/bin/env bash

# Shared PAS-16 gate-plan reporting for local gate scripts.
# Source this file from scripts after changing to the repository root.

npa_package_gate_plan_base() {
  printf '%s' "${NPA_PACKAGE_GATE_PLAN_BASE:-origin/main}"
}

npa_package_gate_plan_report_enabled() {
  case "${NPA_PACKAGE_GATE_PLAN:-report}" in
    off|OFF|0|false|FALSE|no|NO)
      return 1
      ;;
    *)
      return 0
      ;;
  esac
}

npa_package_gate_plan_selection_enabled() {
  case "${NPA_PACKAGE_GATE_PLAN_SELECT:-0}" in
    1|true|TRUE|yes|YES)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

npa_package_gate_plan_changed_path_count() {
  local base="$1"
  local changed_paths

  if ! changed_paths="$(git diff --name-only "${base}...HEAD" 2>/dev/null)"; then
    printf '%s' "unknown"
    return 0
  fi

  printf '%s\n' "${changed_paths}" | sed '/^$/d' | wc -l | tr -d '[:space:]'
}

npa_package_gate_plan_report() {
  local current_gate="$1"
  local base
  local changed_path_count
  local plan_output

  if ! npa_package_gate_plan_report_enabled; then
    return 0
  fi

  base="$(npa_package_gate_plan_base)"
  changed_path_count="$(npa_package_gate_plan_changed_path_count "${base}")"

  echo "[gate-plan] current_gate=${current_gate}"
  echo "[gate-plan] base=${base}"
  echo "[gate-plan] changed_path_count=${changed_path_count}"

  if plan_output="$(cargo run -q -p npa-cli -- package gate-plan --base "${base}" --root proofs 2>&1)"; then
    printf '%s\n' "${plan_output}"
  else
    printf '%s\n' "${plan_output}"
    echo "[gate-plan] report unavailable; continuing existing ${current_gate} commands"
  fi
}

npa_package_gate_plan_apply_selection() {
  local current_gate="$1"
  local base
  local plan_json

  if ! npa_package_gate_plan_selection_enabled; then
    return 0
  fi

  base="$(npa_package_gate_plan_base)"
  if ! plan_json="$(cargo run -q -p npa-cli -- package gate-plan --base "${base}" --root proofs --json 2>&1)"; then
    printf '%s\n' "${plan_json}"
    echo "[gate-plan] selection unavailable; running existing ${current_gate} commands"
    return 0
  fi

  if printf '%s' "${plan_json}" | grep -F "\"status\":\"passed\"" >/dev/null \
    && printf '%s' "${plan_json}" | grep -F "${current_gate}" >/dev/null; then
    echo "[gate-plan] selection=opt-in current_gate=${current_gate} selected=true"
    return 0
  fi

  echo "[gate-plan] selection=opt-in current_gate=${current_gate} selected=false"
  echo "[gate-plan] exiting without running ${current_gate}; gate-plan is not proof evidence"
  exit 0
}
