use log::debug;
use crate::{Participant, Slot, Stake, rand, MIN_STAKE};

#[derive(Debug, Clone)]
pub struct Round {
    committee_size:usize,
    signature_threshold:usize,
    participants:Vec<Participant>,
    slots:Vec<Slot>
}

impl  Round{
    fn empty(committee_size:usize) -> Self {
        Self {
            committee_size,
            signature_threshold: committee_size/2,
            participants: vec![],
            slots: vec![]
        }
    }

    pub fn get_slots(&self) -> &Vec<Slot> {
        &self.slots
    }

    pub fn signature_threshold(&self) -> usize {
        self.signature_threshold
    }

    // dsa
    pub fn new(stakes:&Vec<Stake>, slots_size:usize, committee_size:usize, btc_hash:Vec<u8>) -> Round {
        // prepare the structure
        let mut round = Self::empty(committee_size);

        // create a temporary vector for looping
        let mut participants:Vec<Participant> = stakes.iter().map(|stake| {
            Participant::from(stake.clone())
        }).collect();

        let mut pcpants:Vec<(usize, Participant)> = participants.iter().enumerate().map(|(idx,p)| {
            (idx, p.clone())
        }).collect();

        let mut slots:Vec<Slot> = vec![];

        for slot_idx in 0 .. slots_size {
            debug!("SLOT:{}", slot_idx);

            let total_stakes:u32 =  pcpants.iter().map(|(_,stake)| stake.weight).sum();

            let mut rnd = rand(slot_idx as u8, btc_hash.clone(), total_stakes);

            for (pcpant_idx, pcpant) in pcpants.iter_mut() {

                // found a participant to act as leader for this slot
                if pcpant.weight >= rnd {
                    if pcpant.weight >= MIN_STAKE {
                        pcpant.weight -= MIN_STAKE;
                    } else {
                        pcpant.weight = 0;
                    }

                    let mut p = &mut participants[*pcpant_idx];
                    p.add_leader_slot(slot_idx as u32);

                    debug!(" -> leader: {:?}", participants[*pcpant_idx]);

                    // add a new slot
                    slots.push(Slot::new(
                        slot_idx as u32,
                        &mut participants,
                        pcpant.id,
                        committee_size,
                        btc_hash.clone()
                    ));

                    break;
                } else {
                    rnd -= pcpant.weight;
                }
            }

        }

        round.participants = participants;
        round.slots = slots;

        round

    }
}