use log::debug;
use crate::{Participant, Slot, Stake, rand, MIN_STAKE, total_stakes, btc_hash, total_stakes2};

/// used in Config
use serde::{Serialize,Deserialize};
use validator::{Validate, ValidationError};

#[derive(Serialize, Deserialize, Validate)]
#[validate(schema(function = "validate_config", skip_on_field_errors = false))]
pub struct Config {
    #[validate(length(min = 1))]
    pub btc:String,

    #[validate(range(min = "MIN_STAKE"))]
    pub stake_per_slot:u32,

    pub slots_size:usize,
    pub committee_size:usize,
    pub min_participants:usize,
    pub stakes: Vec<Stake>
}

fn validate_config(cfg:&Config) -> Result<(), ValidationError> {

    if cfg.stakes.len() < cfg.min_participants {
        return Err(ValidationError::new("stakes set is less than the no of participants."));
    }

    if cfg.min_participants < cfg.committee_size {
       return Err(
           ValidationError::new(
               "committee size must be a subset of the number of participants available."
           ));
    }


    Ok(())

}

impl Config {

    pub fn total_stakes(&self) -> u32 {
        total_stakes(&self.stakes)
    }

    fn get_participants(&self) -> Vec<Participant> {
        let min_participants = self.min_participants;
        let btc_hash = btc_hash(self.btc.as_str());

        if self.stakes.len() == min_participants {
           self.stakes.iter().map(|stake| {
                Participant::from(stake.clone())
            }).collect()
        } else {
            // choose participants between the stakes
            let mut participants:Vec<Participant> = vec![];

            // sort the stakes
            let mut sorted_stakes = self.stakes.clone();
            sorted_stakes.sort_by(|x,y| x.weight.cmp(&y.weight));

            for pcpants_idx in 0 .. min_participants {
                let mut total_stakes = total_stakes(&sorted_stakes);
                let mut rnd = rand(pcpants_idx as u8,btc_hash.clone(),total_stakes);

                sorted_stakes.retain(|stake|{
                    if participants.len() == min_participants {
                        // retain nothing, if participants list is full.
                        false
                    } else if stake.weight >= rnd {
                        // found a
                        participants.push(Participant::new(stake.id,stake.weight));
                        false
                    } else {
                        // random number is too big.
                        rnd -= stake.weight;
                        true
                    }
                });
            }

            participants
        }
    }
}


#[derive(Debug, Clone)]
pub struct Round {
    signature_threshold:usize,
    participants:Vec<Participant>,
    slots:Vec<Slot>
}

impl  Round{

    pub fn get_slots(&self) -> &Vec<Slot> {
        &self.slots
    }

    pub fn signature_threshold(&self) -> usize {
        self.signature_threshold
    }

    // dsa
    pub fn generate(cfg:Config)-> Round {
        let btc_hash = btc_hash(cfg.btc.as_str());
        let mut participants = cfg.get_participants();

        let mut pcpants:Vec<Participant> = participants.clone();

        let mut slots:Vec<Slot> = vec![];

        // fill the slots
        for slot_idx in 0 .. cfg.slots_size {
            let total_stakes:u32 = total_stakes2(&pcpants);

            let mut rnd = rand(slot_idx as u8, btc_hash.clone(), total_stakes);

            let mut drop_idx: (bool,usize) = (false, 0);

            for (participant_idx, pcpant) in pcpants.iter_mut().enumerate() {

                // found a participant to act as leader for this slot
                if pcpant.weight >= rnd {
                    if pcpant.weight >= cfg.stake_per_slot {
                        pcpant.weight -= cfg.stake_per_slot;
                    } else {
                       // This participant has no more stakes left to earn another slot.
                       drop_idx = (true, participant_idx);
                    }

                    let leader_id = {
                        let mut leader = &mut participants[participant_idx];
                        leader.add_leader_slot(slot_idx as u32);

                        leader.id
                    };

                    // add a new slot
                    slots.push(Slot::new(
                        slot_idx as u32,
                        &mut participants,
                        leader_id,
                        cfg.committee_size,
                        btc_hash.clone()
                    ));

                    break;
               } else{
                    rnd -= pcpant.weight;
                }
            }

            // remove the participant with no/few stakes left.
            if drop_idx.0 {
                pcpants.remove(drop_idx.1);
            }
        }

        Round {
            signature_threshold: cfg.committee_size/2,
            participants,
            slots
        }
    }
}