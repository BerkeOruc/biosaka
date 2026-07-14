#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

echo "=== BioSaka — C. elegans Neural Simulator ==="
echo ""

# 1 — check Rust toolchain
if ! command -v cargo &>/dev/null; then
    echo "[FAIL] Rust/Cargo not found. Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 2 — check connectome data
if [ ! -f data/connectome.csv ]; then
    echo "[FAIL] Missing data/connectome.csv — the connectome edge list is required."
    exit 1
fi
echo "[OK]  Connectome data: $(wc -l < data/connectome.csv) lines"

# 3 — build
echo "[...] Building (release mode)..."
BUILD_OUT=$(cargo build --release 2>&1) || {
    echo "[FAIL] Build failed:"
    echo "$BUILD_OUT"
    exit 1
}
echo "[OK]  Build successful"

# 4 — show key stats from binary
echo ""
echo "--- Project Stats ---"
echo "  Rust source: $(find src -name '*.rs' | wc -l) modules, $(wc -l src/**/*.rs src/*.rs 2>/dev/null | tail -1) lines"
echo "  Embedded: $(grep -c 'NEURON_NAMES' target/release/build/biosaka-*/out/connectome_data.rs 2>/dev/null || echo "?") neurons, ??? edges"
echo "  Terminal UI: ratatui + crossterm"
echo "  Simulation: LIF spiking neurons + gap junctions"
echo ""

# 5 — check if running in a real terminal
if [ ! -t 0 ]; then
    echo "[WARN] Not a real TTY — the TUI needs an interactive terminal."
    echo "       Run './run.sh' directly in your terminal emulator."
    echo ""
fi

# 6 — launch
echo "=== Launching BioSaka ==="
echo "  Controls: [1/2/3] tabs  [Space] pause  [+/-] zoom  [q] quit"
echo ""
exec cargo run --release
