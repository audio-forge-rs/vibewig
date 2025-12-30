#!/bin/bash
# Build the Voxel plugin

set -e

echo "Building plugin..."
cargo xtask bundle vibewig-plugin --release

echo ""
echo "Plugin ready at: target/bundled/vibewig-plugin.clap"
echo "Add this folder to Bitwig: Settings → Locations → Plug-in Locations"
echo "  $(pwd)/target/bundled"
