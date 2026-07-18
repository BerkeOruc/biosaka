#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/connectome_data.rs"));

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sex {
    Hermaphrodite,
    Male,
}

pub fn neuron_names(sex: &Sex) -> &'static [&'static str] {
    match sex {
        Sex::Hermaphrodite => HERM_NAMES,
        Sex::Male => MALE_NAMES,
    }
}

pub fn num_neurons(sex: &Sex) -> u16 {
    match sex {
        Sex::Hermaphrodite => HERM_NEURONS,
        Sex::Male => MALE_NEURONS,
    }
}

pub fn num_edges(sex: &Sex) -> usize {
    match sex {
        Sex::Hermaphrodite => HERM_EDGES,
        Sex::Male => MALE_EDGES,
    }
}

pub fn edges(sex: &Sex) -> &'static [(u16, u16, u8, u16)] {
    match sex {
        Sex::Hermaphrodite => HERM_SYNAPSES,
        Sex::Male => MALE_SYNAPSES,
    }
}
