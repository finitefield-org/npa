#!/usr/bin/env bash
set -euo pipefail

npa_binary_path="${NPA_BINARY_PATH:-}"
npa_version="${NPA_VERSION:-}"
npa_git_tag="${NPA_GIT_TAG:-}"
npa_git_commit="${NPA_GIT_COMMIT:-}"
rust_toolchain="${RUST_TOOLCHAIN_VERSION:-1.95.0}"

source_count=0
for value in "$npa_binary_path" "$npa_version" "$npa_git_tag" "$npa_git_commit"; do
  if [ -n "$value" ]; then
    source_count=$((source_count + 1))
  fi
done

if [ "$source_count" -ne 1 ]; then
  echo "Set exactly one pinned npa source: NPA_BINARY_PATH, NPA_VERSION, NPA_GIT_TAG, or NPA_GIT_COMMIT." >&2
  exit 2
fi

if [ -n "$npa_version" ]; then
  echo "NPA_VERSION release-download mode is not implemented yet; use NPA_BINARY_PATH, NPA_GIT_TAG, or NPA_GIT_COMMIT." >&2
  exit 2
fi

mkdir -p "$RUNNER_TEMP/npa-bin"

if [ -n "$npa_binary_path" ]; then
  case "$npa_binary_path" in
    /*) resolved_npa_path="$npa_binary_path" ;;
    *) resolved_npa_path="$GITHUB_WORKSPACE/$npa_binary_path" ;;
  esac
  if [ ! -x "$resolved_npa_path" ]; then
    echo "NPA_BINARY_PATH is not executable: $npa_binary_path" >&2
    exit 2
  fi
  ln -sf "$resolved_npa_path" "$RUNNER_TEMP/npa-bin/npa"
else
  if [ -n "$npa_git_commit" ] && [[ ! "$npa_git_commit" =~ ^[0-9a-f]{40}$ ]]; then
    echo "NPA_GIT_COMMIT must be a full lowercase 40-hex commit SHA." >&2
    exit 2
  fi
  if [ -n "$npa_git_tag" ]; then
    if [[ "$npa_git_tag" == "latest" ]] || [[ ! "$npa_git_tag" =~ ^v[0-9]+[.][0-9]+[.][0-9]+([-.][0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
      echo "NPA_GIT_TAG must be an exact immutable tag such as v0.1.0." >&2
      exit 2
    fi
  fi
  if [[ ! "$rust_toolchain" =~ ^[0-9]+[.][0-9]+[.][0-9]+$ ]]; then
    echo "RUST_TOOLCHAIN_VERSION must be an exact version such as 1.95.0." >&2
    exit 2
  fi

  npa_source="$RUNNER_TEMP/npa-source"
  rm -rf "$npa_source"
  git init "$npa_source"
  git -C "$npa_source" remote add origin https://github.com/finitefield-org/npa.git

  if [ -n "$npa_git_commit" ]; then
    git -C "$npa_source" fetch --depth 1 origin "$npa_git_commit"
    git -C "$npa_source" checkout --detach FETCH_HEAD
    actual_commit="$(git -C "$npa_source" rev-parse HEAD)"
    if [ "$actual_commit" != "$npa_git_commit" ]; then
      echo "Fetched npa commit mismatch: expected $npa_git_commit, got $actual_commit" >&2
      exit 2
    fi
  else
    git -C "$npa_source" fetch --depth 1 origin "refs/tags/$npa_git_tag:refs/tags/$npa_git_tag"
    git -C "$npa_source" checkout --detach "refs/tags/$npa_git_tag"
    actual_tag_commit="$(git -C "$npa_source" rev-list -n 1 "refs/tags/$npa_git_tag")"
    actual_commit="$(git -C "$npa_source" rev-parse HEAD)"
    if [ "$actual_commit" != "$actual_tag_commit" ]; then
      echo "Fetched npa tag mismatch: expected $actual_tag_commit, got $actual_commit" >&2
      exit 2
    fi
  fi

  rustup toolchain install "$rust_toolchain" --profile minimal
  cargo "+$rust_toolchain" build --manifest-path "$npa_source/Cargo.toml" -p npa-cli --locked --release
  ln -sf "$npa_source/target/release/npa" "$RUNNER_TEMP/npa-bin/npa"
  cargo "+$rust_toolchain" --version
  rustc "+$rust_toolchain" --version
fi

echo "$RUNNER_TEMP/npa-bin" >> "$GITHUB_PATH"
"$RUNNER_TEMP/npa-bin/npa" --version
