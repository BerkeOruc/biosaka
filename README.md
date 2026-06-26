# BioSaka Devlog

**Creator:** Berke Oruc

**Hack Club project** | Rust + Bevy + biological data

## Why I built this project

I wanted to write something for Hack Club.. I did not want to create another to-do list or weather app. I’m interested in neuroscience. One day I found out about a worm called *C. Elegans*. It has 302 neurons. The connections between these neurons have been fully mapped since 1986. So I had data about 302 neurons and their connections. This worm can walk, eat and run away from danger. It does all this with 302 neurons. I asked myself: If I took this network data and gave it to a creature would that creature walk?

## Phase 1. Infrastructure

The first week was spent getting the project started.

**Why I chose Rust:** It is memory safe unlike C++. It is also fast unlike Python. Bevy is written in Rust so it made sense to use it.

**Petgraph:** I modeled the network as a directed graph. I used `DiGraph<Neuron, Synapse>` because there are types of connections between neurons.

**What I saw when I first ran it:**

```

neurons: 36 (s=14 i=11 m=11)

synapses: 71

interneuron, with the inputs: AVA (in=12 out=6)

```

AVA is one of *C. Elegans*s main command neurons. It helps with movement. Seeing this in my code was really satisfying.

**But there was an issue:** `cargo build` kept timing out. It was slow because it had to download a lot of data. I fixed it by changing my `.cargo/config.toml` file.

```Toml

[registries.crates-io]

protocol = "sparse"

```

## Next steps

- **Phase 2:** I will simulate neurons in parallel using Rayon. I will take a snapshot of the graph to make it faster.

- **Phase 3:** I will create an environment using Bevy. I will make a creature with 8 legs.

- **Phase 4:** I will connect the creatures senses to its neurons.

- **Phase 5:** I will make the creature evolve. The creature that walks the furthest will have a chance of survival.

## Current status

Phase 1 is done. The code loads data from a CSV file. Builds a graph correctly. The code also calculates statistics. The code compiles without warnings. The next phases will take a time.. The first step was really satisfying. Simulating a 302-neuron worm is harder than I thought.

## Resources

- White et al. 1986 – The C. Elegans* connectome paper

- OpenWorm project – A more modern dataset

- Scheffer et al. 2020 – *Drosophila* connectome (what I want to do next)
