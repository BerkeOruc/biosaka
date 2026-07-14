pub struct Connectome {
    pub num_neurons: u16,
    pub neuron_names: Vec<String>,
    pub chemical_edges: Vec<(u16, u16, u16)>,
    pub gap_junction_edges: Vec<(u16, u16, u16)>,
}

impl Connectome {
    pub fn load() -> Self {
        let num_neurons = crate::generated::NUM_NEURONS;
        let neuron_names: Vec<String> = (0..num_neurons)
            .map(|i| crate::generated::NEURON_NAMES[i as usize].to_string())
            .collect();

        let mut chemical_edges = Vec::new();
        let mut gap_junction_edges = Vec::new();

        for &(pre, post, conn_type, weight) in crate::generated::EDGES {
            match conn_type {
                0 => chemical_edges.push((pre, post, weight)),
                1 => gap_junction_edges.push((pre, post, weight)),
                _ => {}
            }
        }

        Connectome {
            num_neurons: crate::generated::NUM_NEURONS,
            neuron_names,
            chemical_edges,
            gap_junction_edges,
        }
    }

    pub fn num_neurons(&self) -> u16 {
        self.num_neurons
    }

    pub fn neuron_name(&self, id: u16) -> &str {
        &self.neuron_names[id as usize]
    }

    pub fn total_chemical_synapses(&self) -> usize {
        self.chemical_edges.len()
    }

    pub fn total_gap_junctions(&self) -> usize {
        self.gap_junction_edges.len()
    }

    #[allow(dead_code)]
    pub fn total_connections(&self) -> usize {
        self.chemical_edges.len() + self.gap_junction_edges.len()
    }

    pub fn get_chemical_edges(&self) -> &[(u16, u16, u16)] {
        &self.chemical_edges
    }

    pub fn get_gap_junction_edges(&self) -> &[(u16, u16, u16)] {
        &self.gap_junction_edges
    }
}
