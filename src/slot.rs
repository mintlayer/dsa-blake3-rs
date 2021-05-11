use log::debug;

use crate::{ParticipantId, Participant, rand, total_stakes2};

pub type Id = u32;

#[derive(Debug, Clone)]
pub struct Slot {
    pub id:u32,
    pub leader:ParticipantId,
    committee:Vec<ParticipantId>
}

impl Slot {

    pub fn get_committee(&self) -> &Vec<ParticipantId> {
        &self.committee
    }

    pub fn new(id:u32, participants:&mut Vec<Participant>, leader:ParticipantId, committee_size:usize, btc_hash:Vec<u8>) -> Self {
        let committee = random_committee(id, participants, leader, committee_size, btc_hash );

        Slot {
            id,
            leader,
            committee
        }
    }
}


fn random_committee(id:u32,
                    participants:&mut Vec<Participant>,
                    leader:ParticipantId,
                    committee_size:usize,
                    btc_hash:Vec<u8>
) -> Vec<ParticipantId> {
    let mut committee: Vec<ParticipantId> = vec![];

    if committee_size >= participants.len() {
        // insufficient participants to reach the set committee_size

        participants.iter_mut().for_each(|p| {
            p.add_signer_slot(id);
            committee.push(p.id);

        })
    } else {

        let mut total_stakes = total_stakes2(participants);

        let mut pcpants:Vec<(usize,u32)> = participants.iter().enumerate().map(|(idx,p)| {
            (idx,p.weight)
        }).collect();

        let mut rnd = rand(id as u8, btc_hash.clone(), total_stakes);

        while committee.len() < committee_size {

            pcpants.retain(|(idx,weight)|{
                if committee.len() == committee_size {
                    // if committee has been filled, don't bother checking for the others.
                    false
                }
                else if weight >= &rnd {

                    let p = &mut participants[*idx];
                    if p.id != leader {
                        // current participant is a signer for current slot.
                        p.add_signer_slot(id);

                        committee.push(p.id);

                        // deduct the current participants's stakes
                        total_stakes -= weight;

                        // recalculate the random number
                        rnd = rand(id as u8, btc_hash.clone(), total_stakes);

                        debug!(" -> adding committee: {:?}", participants[*idx]);
                    }

                    false
                }
                else {
                    // random number is too big.
                    rnd -= weight;
                    true
                }
            });
        }
    }

    committee
}