# Development Log — BioSaka v0.1

## 2026-07-14

**"Stand ready for my arrival, worm."**

I built a worm brain in a terminal. Not a metaphor. An actual C. elegans connectome — 307 neurons, ~2800 synapses from the White 1986 EM reconstruction — running as a spiking neural network in a ratatui TUI.

The worm moves. The graph lights up. You can watch every neuron fire in real time.

Why? Because I wanted to see what happens when you take a real biological connectome, stuff it into a Rust binary, and let it run. The answer: it looks like a disco ball made of synapses. Some neurons fire constantly. Some never fire. Some oscillate. It's alive in the way a campfire is alive.

The hardest part was making the worm look like it's actually crawling and not just having a seizure. The answer was a sinusoidal wave with phase propagation along the body segments. Motor neuron activity modulates frequency. Left-right asymmetry modulates direction. It's not perfect but it's a worm and it moves.

Stand ready for my arrival, worm.

— berke, 2026
