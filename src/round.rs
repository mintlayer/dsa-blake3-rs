use log::debug;
use crate::{Participant, Slot, Stake, rand, MIN_STAKE, total_stakes, btc_hash, total_stakes2, ParticipantId, ParticipantIdx};

/// used in Config
use serde::{Serialize,Deserialize};
use validator::{Validate, ValidationError};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Validate)]
#[validate(schema(function = "validate_config", skip_on_field_errors = false))]
pub struct Config {
    #[validate(length(min = 1))]
    pub btc:String,

    #[validate(range(min = "MIN_STAKE"))]
    pub stake_per_slot:u32,

    pub committee_size:usize,
    pub min_participants:usize,
    pub stakes: Vec<Stake>
}

fn validate_config(cfg:&Config) -> Result<(), ValidationError> {

    if cfg.stakes.len() < cfg.min_participants {
        return Err(ValidationError::new("stakes set is less than the no. of participants."));
    }

    if cfg.min_participants < cfg.committee_size {
       return Err(
           ValidationError::new(
               "committee size must be a subset of the no. of participants available."
           ));
    }

    Ok(())
}

impl Config {

    pub fn total_stakes(&self) -> u32 {
        total_stakes(&self.stakes)
    }

    // dsa
    fn get_participants(&self) -> Vec<Participant> {
        let btc_hash = btc_hash(self.btc.as_str());

        let mut participants: Vec<Participant> = vec![];

        let mut stakes = self.stakes.clone();

        for idx in  0 .. self.min_participants {
            let mut drop_idx: (bool,usize) = (false, 0);

            let total_stakes:u32 = total_stakes(&stakes);
            let mut rnd = rand(idx as u8, btc_hash.clone(), total_stakes);

            for (stake_idx, stake) in stakes.iter_mut().enumerate() {
                // found a participant
                if stake.weight >= rnd {

                   let weight =  if stake.weight >= self.stake_per_slot {
                        stake.weight -= self.stake_per_slot;

                        self.stake_per_slot
                    } else {
                        drop_idx = (true, stake_idx);

                        stake.weight
                    };

                    let p = Participant::new(idx as u32, stake.id,weight);
                    participants.push(p);

                    break;
                } else {
                    rnd -= stake.weight;
                }
            }

            // remove the stake with no/few weight left.
            if drop_idx.0 {
                stakes.remove(drop_idx.1);
            }
        }

        participants
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

    pub fn get_participants(&self) -> &Vec<Participant> {
        &self.participants
    }

    pub fn signature_threshold(&self) -> usize {
        self.signature_threshold
    }

    pub fn generate(cfg:Config)-> Round {
        let btc_hash = btc_hash(cfg.btc.as_str());
        let mut participants = cfg.get_participants();

        let mut slots:Vec<Slot> = vec![];

        let mut bypass_idx:HashMap<ParticipantIdx, ()> = HashMap::new();

        // create slots
        for idx in 0 .. participants.len() {
            slots.push(Slot::new(
                idx,
                &mut bypass_idx,
                &mut participants,
                cfg.committee_size,btc_hash.clone()
            ))
        }

        Round {
            signature_threshold: cfg.committee_size/2,
            participants,
            slots
        }
    }
}