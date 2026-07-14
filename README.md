# BioSaka

**The worm meets bare metal.**

<video src="biosaka.mp4" controls width="100%"></video>

*307 neurons, ~2800 synapses, all running in your terminal.*

## what

BioSaka loads the actual C. elegans hermaphrodite connectome from White et al. 1986 and Cook et al. 2019, runs a leaky integrate-and-fire neural simulation, and draws it in your terminal. Watch 307 neurons spike in real time. Watch the worm crawl. Watch the brain light up.

no GUI. no bloat. just a worm in a box.

## features

- **real data** — 307 neurons, 2847 edges from published EM reconstructions. every synapse is real.
- **spiking simulation** — LIF neurons with synaptic transmission and gap junction coupling
- **5-tab TUI** — neural graph, worm view, statistics, credits, technical info
- **live graph** — all 307 neurons in a circular layout, color coded by activity
- **worm body** — 20-segment body with sinusoidal movement driven by motor neurons
- **real-time stats** — network activity, spike counts, per-neuron firing rates
- **interactive** — pause, zoom, pan, switch views. no mouse required.

## controls

| key | what it does |
|---|---|
| `1` `2` `3` | graph / worm / stats |
| `c` | credits |
| `i` | technical info (scroll with up/down) |
| `space` | pause / resume |
| `+` / `-` | zoom in / out (graph tab) |
| arrows | pan (graph tab) or scroll (info tab) |
| `q` | quit |

## quick start

### cargo install (recommended)

```bash
cargo install biosaka
biosaka worm
```

### from source

```bash
git clone https://github.com/berkeoruc/biosaka
cd biosaka
cargo run --release
```

works on linux, macos, windows. needs a terminal that likes crossterm.

## how it works

```
                         ┌──────────────────────┐
  data/connectome.csv ──>│      build.rs        │──> static edge list
    (White 1986)         │  (compile time)      │    (307 neurons)
                         └──────────────────────┘
                                    │
                         ┌─────────▼──────────┐
                         │  LIF neural engine │
                         │  (simulation loop) │
                         └─────────┬──────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
             ┌──────────┐   ┌──────────┐   ┌──────────────┐
             │  neuron  │   │  worm    │   │    TUI       │
             │  model   │   │  body    │   │  (ratatui)   │
             └──────────┘   └──────────┘   └──────────────┘
```

neurons are leaky integrate-and-fire units:
- membrane potential leaks (τ = 0.95 per step)
- chemical synapses: pre fires → post gets weight * 0.15
- gap junctions: direct electrical coupling
- gaussian noise keeps things interesting

the worm body is 20 segments. motor neurons (VB, DB, VA, DA) drive a sinusoidal wave. left-right asymmetry makes it turn.

## data sources

- White, J.G. et al. (1986). *The Structure of the Nervous System of the Nematode Caenorhabditis elegans.* Phil. Trans. R. Soc. Lond. B
- Cook, S.J. et al. (2019). *Whole-animal connectomes of both Caenorhabditis elegans sexes.* Nature 571, 63-71
- OpenWorm project — [c302](https://github.com/openworm/c302)

## license

**BioSaka Research License** — All rights reserved.

This project is a learning and research project. You may use,
modify, and study the code for **non-commercial research and
educational purposes only**. Commercial use, redistribution,
and incorporation into commercial products are prohibited.

See [LICENSE](LICENSE) for full terms.

---

*handwritten. berke oruc, 2026.*
