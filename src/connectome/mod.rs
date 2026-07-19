use crate::generated::{self, Sex};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Connectome {
    pub num_neurons: u16,
    pub neuron_names: Vec<String>,
    pub chemical_edges: Vec<(u16, u16, u16)>,
    pub gap_junction_edges: Vec<(u16, u16, u16)>,
    pub sex: Sex,
    name_to_id: HashMap<String, u16>,
}

impl Connectome {
    pub fn load(sex: Sex) -> Self {
        let num_neurons = generated::num_neurons(&sex);
        let neuron_names: Vec<String> = (0..num_neurons)
            .map(|i| generated::neuron_names(&sex)[i as usize].to_string())
            .collect();

        let mut chemical_edges = Vec::new();
        let mut gap_junction_edges = Vec::new();

        for &(pre, post, conn_type, weight) in generated::edges(&sex) {
            match conn_type {
                0 => chemical_edges.push((pre, post, weight)),
                1 => gap_junction_edges.push((pre, post, weight)),
                _ => {}
            }
        }

        let name_to_id: HashMap<String, u16> = neuron_names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i as u16))
            .collect();

        Connectome {
            num_neurons,
            neuron_names,
            chemical_edges,
            gap_junction_edges,
            sex,
            name_to_id,
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

    pub fn total_connections(&self) -> usize {
        self.chemical_edges.len() + self.gap_junction_edges.len()
    }

    pub fn get_chemical_edges(&self) -> &[(u16, u16, u16)] {
        &self.chemical_edges
    }

    pub fn get_gap_junction_edges(&self) -> &[(u16, u16, u16)] {
        &self.gap_junction_edges
    }

    pub fn sex_label(&self) -> &'static str {
        match self.sex {
            Sex::Hermaphrodite => "Hermaphrodite",
            Sex::Male => "Male",
        }
    }

    /// Find all neuron IDs whose name starts with the given prefix.
    /// Used for sensory stimulation keyed by neuron group name.
    pub fn find_by_prefix(&self, prefix: &str) -> Vec<u16> {
        self.neuron_names
            .iter()
            .enumerate()
            .filter(|(_, name)| name.starts_with(prefix))
            .map(|(i, _)| i as u16)
            .collect()
    }

    /// Look up a neuron ID by exact name. Returns None if not found.
    pub fn id_of(&self, name: &str) -> Option<u16> {
        self.name_to_id.get(name).copied()
    }
}
