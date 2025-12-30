#!/bin/bash
# Build and install the Voxel plugin to Bitwig's CLAP folder

set -e

echo "Building plugin..."
cargo xtask bundle vibewig-plugin --release

echo "Installing to ~/Library/Audio/Plug-Ins/CLAP/..."
rm -rf ~/Library/Audio/Plug-Ins/CLAP/Voxel.clap
cp -r target/bundled/vibewig-plugin.clap ~/Library/Audio/Plug-Ins/CLAP/Voxel.clap

echo "Done. In Bitwig: Settings → Locations → Rescan Plug-ins, then search 'Voxel'"
