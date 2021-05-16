use std::collections::HashMap;
use crate::{SlotId, Stake};

pub type Id = u32;
pub type Key = u32;
pub type Idx = usize;

#[derive(Debug, Clone)]
pub struct Participant {
    pub id:Id,
    pub key:Key,
    pub weight:u32,
    signer_slots: HashMap<SlotId,bool>,
}

impl Participant {
    pub fn new(id:Id, key:u32, weight:u32) -> Self {
        Self {
            id,
            key,
            weight,
            signer_slots: HashMap::new()
        }
    }

    pub fn add_signer_slot(&mut self, slot_id:SlotId) {
        self.signer_slots.insert(slot_id,false);
    }

    pub fn num_of_signer_slots(&self) -> usize {
        self.signer_slots.len()
    }
}

