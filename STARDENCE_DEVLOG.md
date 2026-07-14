# Stardence Devlog — BioSaka v0.1

**"Stand ready for my arrival, worm."**

So I built a worm brain. In Rust. In a terminal.

307 neurons, ~2800 synapses, all from the actual C. elegans connectome. White et al. 1986 spent years slicing a worm into 8000 serial sections under an electron microscope. I spent a weekend turning that into a terminal app. I think I got the better deal.

The simulation runs LIF neurons — leaky integrate-and-fire. Every tick, potentials decay, synapses fire, gap junctions couple, noise jitters. The worm body wiggles. The graph lights up. It's a tiny universe in a terminal window.

ratatui handles the UI. crossterm handles the keys. rand handles the noise. the connectome data gets compiled into the binary at build time — no file I/O at runtime, just pure computation.

The hardest part was getting the worm to look like it's actually crawling and not just vibrating. Turns out you need a proper sinusoidal wave with phase propagation. Who knew.

Stand ready for my arrival, worm.

— berke, 2026
