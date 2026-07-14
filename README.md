# BioSaka

**The worm meets bare metal.**

<p align="center">
  <video src="biosaka.mp4" controls width="640"></video>
  <br>
  <em>307 neurons, ~2800 synapses. all running in your terminal.</em>
</p>

## what

BioSaka loads the actual C. elegans hermaphrodite connectome from White et al. 1986 and Cook et al. 2019, runs a leaky integrate-and-fire neural simulation, and draws it in your terminal. Watch 307 neurons spike in real time. Watch the worm crawl. Watch the brain light up.

no GUI. no bloat. just a worm in a box.

no windows support either. this worm doesnt do microslop. tested on arch linux. works on macos and other linux distros if youre not a coward.

## quick start

### arch linux (AUR)

```bash
yay -S biosaka
biosaka worm
```

### cargo install

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

needs a terminal that likes crossterm. if your terminal cant handle escape codes, get a better terminal.

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

## features

- **real data** — 307 neurons, 2847 edges from published EM reconstructions. every synapse is real.
- **spiking simulation** — LIF neurons with synaptic transmission and gap junction coupling
- **5-tab TUI** — neural graph, worm view, statistics, credits, technical info
- **live graph** — all 307 neurons in a circular layout, color coded by activity
- **worm body** — 20-segment body with sinusoidal movement driven by motor neurons
- **real-time stats** — network activity, spike counts, per-neuron firing rates
- **interactive** — pause, zoom, pan, switch views. no mouse required.

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

neurons are leaky integrate-and-fire units. heres the math without getting boring:

```
V(t+1) = V(t) x 0.95 + I_syn + noise
```

- membrane potential leaks 5% every tick
- chemical synapse: pre fires -> post gets a jolt
- gap junctions: direct electrical coupling between neurons
- gaussian noise keeps things from being boring

the worm body is 20 segments. motor neurons (VB, DB, VA, DA) drive a sinusoidal wave. left-right asymmetry makes it turn. no two runs look the same.

## data sources

- White, J.G. et al. (1986). *The Structure of the Nervous System of the Nematode Caenorhabditis elegans.* Phil. Trans. R. Soc. Lond. B
- Cook, S.J. et al. (2019). *Whole-animal connectomes of both Caenorhabditis elegans sexes.* Nature 571, 63-71
- OpenWorm project — [c302](https://github.com/openworm/c302)

## docs

- `logo.txt` — ASCII art. opens the worm's DNA with a text editor.
- `LICENSE` — research use only. dont sell the worm.
- [aur/biosaka](aur/PKGBUILD) — arch linux package.
- `src/` — 4 modules. 500 lines of rust. all of it handwritten.

## license

**BioSaka Research License** — All rights reserved.

This project is a learning and research project. You may use,
modify, and study the code for **non-commercial research and
educational purposes only**. Commercial use, redistribution,
and incorporation into commercial products are prohibited.

See [LICENSE](LICENSE) for full terms.

---

*handwritten. berke oruc, 2026.*
