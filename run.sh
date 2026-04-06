#!/bin/bash
set -e

cd ~/JVault/src-tauri || { echo "Directory not found"; exit 1; }

if [[ "$1" == "--dev" ]]; then
    cargo tauri dev         # hot reload, faster iteration
else
    cargo tauri build && ~/JVault/src-tauri/target/release/jvault "${@:2}"
fi
