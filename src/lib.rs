
mod slot;
mod participant;
mod round;

extern crate hex;

use log::{debug, info};
use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationError};

pub use participant::{ Participant, Id as ParticipantId, Key as ParticipantKey, Idx as ParticipantIdx };
pub use slot::{ Slot, Id as SlotId };
pub use round::{ Config as RoundConfig, *};

const MIN_STAKE: u32 = 440000;
const BTC: &'static str = "0000000000000000000d9ed0f796aeee51b200c7293a6e31c101a0e4159bf310";


#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct Stake{
    pub id:u32,
    #[validate(range(min = "MIN_STAKE"))]
    pub weight:u32
}

pub fn total_stakes(stakes:&Vec<Stake>) -> u32 {
    // we don't care about overflow for the test
    stakes.iter().map(|s| s.weight).sum()
}

pub(crate) fn total_stakes2(pcpants:&Vec<Participant>) -> u32 {
    pcpants.iter().map(|p| p.weight).sum()
}

pub fn btc_hash(btc:&str) -> Vec<u8> {
    hex::decode(BTC).expect("Decoding failed")
}

fn print_type_of<T>(_: &T) {
    println!("TYPE: {}", std::any::type_name::<T>())
}

fn rand(i: u8, hash: Vec<u8>, top: u32) -> u32 {
    let wrap_i: [u8; 1] = [i];
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"Mintlayer");
    hasher.update(hash.as_slice());
    hasher.update(&wrap_i);
    let mut out = [0; 4];
    let mut output_reader = hasher.finalize_xof();
    output_reader.fill(&mut out);
    let ret: u32 = u32::from_be_bytes(out);
    // we don't care about float for this POC
    ((top as f64) / (u32::MAX as f64) * ret as f64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random() {
        let hash = hex::decode(BTC).expect("Decoding failed");
        assert_eq!(32, hash.len());

        let res = rand(1, hash, 100);
        assert_eq!(50, res);
    }

    #[test]
    fn validate_stake() {
        let stake = Stake {
            id: 0,
            weight: MIN_STAKE - 1
        };

        assert!(stake.validate().is_err());

        let stake = Stake {
            id: 0,
            weight: MIN_STAKE + 1
        };

        assert!(stake.validate().is_ok());
    }

    #[test]
    fn test_stakes() {
        let path = std::path::Path::new("tests/assets/test_stakes.json");
        let cfg = std::fs::read_to_string(path).unwrap();
        let cfg:RoundConfig = serde_json::from_str(&cfg).unwrap();

        assert!(cfg.validate().is_ok());

        assert_eq!(9840002,cfg.total_stakes());

        let round = Round::generate(cfg);

        let slots = round.get_slots();

        // prints out the slots in json format
        //
        // let slot_json = serde_json::to_string(&slots).unwrap();
        // println!("SLOTS: {}", slot_json);


        assert_eq!(2,round.signature_threshold());
        assert_eq!(8, slots[0].leader);
        assert_eq!(5, slots[1].leader);
        assert_eq!(8, slots[2].leader);
        assert_eq!(3, slots[3].leader);
        assert_eq!(8, slots[4].leader);
        assert_eq!(7, slots[5].leader);
        assert_eq!(5, slots[6].leader);
        assert_eq!(5, slots[7].leader);
        assert_eq!(8, slots[8].leader);
        //assert_eq!(4, slots[9].leader); removed, because the number of stakes is only 9.
    }

    #[test]
    fn test_committee() {
        let committee_size = 80;
        let min_participants = 1008;

        let mut stakes: Vec<Stake> = vec![];

        for i in 0..min_participants {
            stakes.push(Stake {
                id: i as u32,
                weight: MIN_STAKE + i as u32 * 2,
            })
        }

        let cfg = RoundConfig {
            btc: BTC.to_string(),
            stake_per_slot: MIN_STAKE,
            committee_size,
            min_participants,
            stakes
        };

        assert!(cfg.validate().is_ok());

        let round = Round::generate(cfg);

        let slots = round.get_slots();

        assert_eq!(40,round.signature_threshold());

        let slots = round.get_slots();
        {
            let slot_0_committee: Vec<u32> = vec![
                820, 642, 463, 285, 108, 929, 750, 571, 393, 215, 38, 858, 680,
                501, 323, 146, 968, 788, 610, 431, 253, 76, 897, 718, 539, 361,
                183, 6, 826, 648, 469, 291, 114, 935, 756, 577, 399, 221, 44,
                864, 686, 507, 329, 151, 974, 793, 615, 436, 258, 81, 902, 723,
                544, 366, 188, 11, 831, 653, 474, 296, 119, 940, 761, 582, 404,
                226, 49, 869, 691, 512, 334, 156, 981, 798, 620, 441, 263, 86,
                907, 728
            ];

            let slot_0 = slots[0].clone();
            let committee = slot_0.get_committee();

            assert_eq!(828, slot_0.leader);
            assert_eq!(80, committee.len());
            assert_eq!(&slot_0_committee, committee);
        }
        {
            let slot_74_committee: Vec<u32> = vec![
                49, 98, 147, 195, 244, 294, 341, 394, 443, 492, 540, 585, 637,
                682, 734, 786, 836, 888, 936, 989, 32, 80, 130, 179, 227, 277,
                327, 379, 429, 477, 525, 571, 620, 667, 718, 768, 820, 871,
                921, 972, 17, 65, 115, 163, 212, 261, 312, 361, 411, 461, 509,
                557, 606, 653, 702, 750, 804, 855, 904, 954, 1, 51, 100, 149,
                197, 246, 296, 345, 396, 445, 494, 542, 588, 639, 684, 736,
                788, 841, 890, 938
            ];

            let slot_74 = slots[74].clone();
            let committee = slot_74.get_committee();

            assert_eq!(49, slot_74.leader);
            assert_eq!(&slot_74_committee, committee);
        }
    }

    #[test]
    fn test_100_participants() {
        let path = std::path::Path::new("tests/assets/participants_100.json");
        let cfg = std::fs::read_to_string(path).unwrap();
        let cfg:RoundConfig = serde_json::from_str(&cfg).unwrap();

        assert!(cfg.validate().is_ok());

        assert_eq!(44699702,cfg.total_stakes());

        let round = Round::generate(cfg);

        let slots = round.get_slots();

        let mut participants = round.get_participants().clone();

        let p0 = &participants[0];
        assert_eq!(28,p0.num_of_signer_slots());

        let p1 = &participants[1];
        assert_eq!(30, p1.num_of_signer_slots());

        let p70 = &participants[70];
        assert_eq!(99,p70.key);
        assert_eq!(24, p70.num_of_signer_slots());

        let p24 = &participants[24];
        assert_eq!(99, p24.key);
        assert_eq!(30, p24.num_of_signer_slots());

        assert_eq!(15,round.signature_threshold());
    }

    #[test]
    fn test_200_stakes() {
        let committee_size = 30;
        let min_participants = 100;

        let mut stakes: Vec<Stake> = vec![];

        for i in 0..150 {
            stakes.push(Stake {
                id: i as u32,
                weight: MIN_STAKE + i as u32 * 2,
            })
        }

        let cfg = RoundConfig {
            btc: BTC.to_string(),
            stake_per_slot: MIN_STAKE,
            committee_size,
            min_participants,
            stakes
        };

        assert!(cfg.validate().is_ok());

        let round = Round::generate(cfg);
        let slots = round.get_slots();

        let mut participants = round.get_participants().clone();

        let p92 = &participants[92];
        assert_eq!(55, p92.key);
        assert_eq!(31, p92.num_of_signer_slots());

        let p71 = &participants[71];
        assert_eq!(36, p71.key);
        assert_eq!(34, p71.num_of_signer_slots());

        let p16 = &participants[16];
        assert_eq!(76, p16.key);
        assert_eq!(31, p16.num_of_signer_slots());

        let p5 = &participants[5];
        assert_eq!(108,p5.key);
        assert_eq!(29,p5.num_of_signer_slots());

        let s5 = &slots[5];
        let s5_committee = s5.get_committee();
        assert_eq!(p5.key, s5.leader);
        assert!(s5_committee.contains(&p71.id));
        assert!(s5_committee.contains(&p92.id));
        assert!(s5_committee.contains(&p16.id));

        // prints out the order of the  participant id, with the corresponding weight and the
        // number of slots that participant is a signer of.
        //
        // participants.sort_by(|p1, p2| {
        //     p1.key.cmp(&p2.key)
        // });
        //
        // participants.iter().for_each(|p| {
        //     println!("PId:{} key:{} wt:{} signer_slots:{}", p.id, p.key, p.weight, p.num_of_signer_slots());
        // });
    }
}
