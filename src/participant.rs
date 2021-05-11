use std::collections::HashMap;
use crate::{SlotId, Stake};

pub type Id = u32;

#[derive(Debug, Clone)]
pub struct Participant {
    pub id:Id,
    pub weight:u32,
    leader_slot:HashMap<SlotId,bool>,
    signer_slots: HashMap<SlotId,bool>,
}

impl Participant {
    pub fn new(id:Id, weight:u32) -> Self {
        Self {
            id,
            weight,
            leader_slot: HashMap::new(),
            signer_slots: HashMap::new()
        }
    }

    pub fn add_signer_slot(&mut self, slot_id:SlotId) {
        self.signer_slots.insert(slot_id,false);
    }

    pub fn add_leader_slot(&mut self, slot_id:SlotId) {
        self.leader_slot.insert(slot_id, false);
    }
}


impl From<Stake> for Participant {
    fn from(stake: Stake) -> Self {
        Self {
            id: stake.id,
            weight: stake.weight,
            leader_slot: HashMap::new(),
            signer_slots: HashMap::new()
        }
    }
}