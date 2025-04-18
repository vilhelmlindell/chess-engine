#!/bin/bash

# List of features in order
features=(
  "alpha-beta"
  "move-ordering"
  "quiescence"
  "tt"
  "pvs"
  "null-move"
  "syzygy"
  "aspiration"
  "lmr"
  "futility"
)

# Get binary name (first binary target from Cargo.toml)
BIN_NAME=$(cargo metadata --no-deps --format-version=1 | jq -r \
  '.packages[0].targets[] | select(.kind[] == "bin") | .name')

# Output base path for custom binaries
OUT_DIR="custom_builds"
mkdir -p "$OUT_DIR"

# Build with no features (base)
echo "Building base binary with no features..."
cargo build --release --no-default-features --target-dir target/base
if [ $? -ne 0 ]; then
  echo "Base build failed."
  exit 1
fi
cp "target/base/release/$BIN_NAME" "$OUT_DIR/${BIN_NAME}-base"

# Accumulator for features
accumulated_features=""

# Loop and build with increasing feature sets
for feature in "${features[@]}"; do
  accumulated_features="$accumulated_features $feature"
  safe_feature_name=$(echo "$feature" | tr '_' '-') # Just in case

  echo "Building with features: $accumulated_features"
  cargo build --release --no-default-features --features "$accumulated_features" \
    --target-dir "target/$feature"

  if [ $? -ne 0 ]; then
    echo "Build failed with features: $accumulated_features"
    exit 1
  fi

  cp "target/$feature/release/$BIN_NAME" "$OUT_DIR/${BIN_NAME}-$feature"
done
