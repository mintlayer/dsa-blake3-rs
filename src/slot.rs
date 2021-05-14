use log::debug;
use serde::{Serialize,Deserialize};
use crate::{ParticipantId, ParticipantKey, Participant, rand, total_stakes2, ParticipantIdx};
use std::collections::HashMap;

pub type Id = u32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Slot {
    pub id:Id,
    pub leader:ParticipantKey,
    committee:Vec<ParticipantId>
}

impl Slot {

    pub fn get_committee(&self) -> &Vec<ParticipantId> {
        &self.committee
    }

    pub fn new(idx:usize, bypass:&mut HashMap<ParticipantIdx,()>,participants:&mut Vec<Participant>, committee_size:usize, btc_hash:Vec<u8>) -> Self {
        let committee = random_committee(idx,bypass, participants, committee_size, btc_hash );

        Slot {
            id: idx as u32,
            leader: participants[idx].key ,
            committee
        }
    }
}


pub fn random_committee(slot_idx:usize,
                         // a flag to bypass participants that had already acted as signers
                         bypass: &mut HashMap<ParticipantIdx, ()>,
                         participants:&mut Vec<Participant>,
                         committee_size:usize,
                         btc_hash:Vec<u8>) -> Vec<ParticipantId> {
    let mut committee: Vec<ParticipantId> = vec![];
    let mut curr_bypass: HashMap<ParticipantIdx, ()> = HashMap::new();

    let slot_id = slot_idx as u32;

    // function to loop over participants as signers
    let to_iterate = |bypass_idx:&HashMap<ParticipantIdx,()>, pcpants:&Vec<Participant>| {
        pcpants.iter().enumerate().filter_map(|(idx, p)| {
            if !bypass_idx.contains_key(&idx) && idx != slot_idx {
                Some((idx, p.weight))
            } else {
                None
            }
        }
        ).collect()
    };

    let mut pcpants:Vec<(ParticipantIdx, u32)> = to_iterate(bypass,participants);

    let mut total:u32 = pcpants.iter().map(|(_,y)| *y).sum();
    let mut rnd = rand(slot_idx as u8, btc_hash.clone(), total );

    while committee.len() < committee_size {

        // all participants have been signers; refresh the bypass flag
        if pcpants.is_empty() {
            bypass.drain();
            bypass.extend( curr_bypass.iter());
            curr_bypass = HashMap::new();

            pcpants = to_iterate(bypass,participants);

            total = pcpants.iter().map(|(_,y)| *y).sum();
            rnd = rand(slot_idx as u8, btc_hash.clone(), total);
        }

        pcpants.retain(|(participant_idx, weight)| {
            if committee.len() == committee_size {
                false
            } else if weight >= &rnd {
                let p = &mut participants[*participant_idx];

                // current participant is a signer for current slot.
                p.add_signer_slot(slot_id);

                // bypass this index, to give other participants a chance to be signers.
                curr_bypass.insert(*participant_idx,());

                committee.push(p.id);

                // deduct the current participants's position
                total -= weight;

                // recalculate the random number
                rnd = rand(slot_id as u8, btc_hash.clone(), total);

                false
            } else {
                rnd -= weight;
                true
            }
        });
    }

    // concatenate the new participants to bypass
    bypass.extend(curr_bypass.iter());
    committee

}