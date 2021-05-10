
mod slot;
mod participant;
mod round;

extern crate hex;

use log::{debug, info};

pub use participant::{ Participant, Id as ParticipantId };
pub use slot::{Slot, Id as SlotId };
pub use round::*;

const MIN_STAKE: u32 = 440000;
const BTC: &'static str = "0000000000000000000d9ed0f796aeee51b200c7293a6e31c101a0e4159bf310";


#[derive(Debug, Clone)]
pub struct Stake{
    pub id:u32,
    pub weight:u32
}

pub fn total_stakes(stakes:&Vec<Stake>) -> u32 {
    // we don't care about overflow for the test
    stakes.iter().map(|s| s.weight).sum()
}

pub(crate) fn total_stakes2(pcpants:&Vec<Participant>) -> u32 {
    pcpants.iter().map(|p| p.weight).sum()
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
    fn test_stakes() {
        let committee_size = 4;
        let slot_size = 10;

        let stakes = vec![
           Stake {
                id: 0,
                weight: MIN_STAKE,
            },
           Stake {
                id: 1,
                weight: MIN_STAKE,
            },
           Stake {
                id: 2,
                weight: MIN_STAKE + 1,
            },
           Stake {
                id: 3,
                weight: MIN_STAKE * 2,
            },
           Stake {
                id: 4,
                weight: MIN_STAKE * 2,
            },
           Stake {
                id: 5,
                weight: MIN_STAKE * 4,
            },
           Stake {
                id: 6,
                weight: 1_000_000,
            },
           Stake {
                id: 7,
                weight: 1_000_001,
            },
           Stake {
                id: 8,
                weight: 3_000_000,
            },
        ];

        let hash = hex::decode(BTC).expect("Decoding failed");

        let round = Round::new(&stakes,slot_size,committee_size,hash);
        let slots = round.get_slots();

        assert_eq!(9840002,total_stakes(&stakes));
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
        assert_eq!(4, slots[9].leader);
    }

    #[test]
    fn test_committee() {
        let committee_size = 80;
        let slot_size = 1008;

        let mut stakes: Vec<Stake> = vec![];

        for i in 0..slot_size {
            stakes.push(Stake {
                id: i as u32,
                weight: MIN_STAKE + i as u32 * 2,
            })
        }

        let hash = hex::decode(BTC).expect("Decoding failed");

        let round = Round::new(&stakes,slot_size,committee_size,hash);
        assert_eq!(40,round.signature_threshold());

        let slots = round.get_slots();
        {
            let slot_0_committee: Vec<u32> = vec![
                829, 649, 470, 291, 112, 941, 761, 582, 403, 224, 45, 875, 695, 516, 337, 158,
                987, 807, 628, 449, 270, 91, 921, 741, 562, 383, 204, 25, 855, 675, 496, 317,
                138, 967, 787, 608, 429, 250, 71, 901, 721, 542, 363, 184, 5, 835, 655, 476,
                297, 118, 948, 768, 589, 410, 231, 52, 882, 702, 523, 344, 165, 994, 814, 635,
                456, 277, 98, 928, 748, 569, 390, 211, 32, 862, 682, 503, 324, 145, 974, 794
            ];

            let slot_0 = slots[0].clone();

            let committee = slot_0.get_committee();

            assert_eq!(828, slot_0.leader);
            assert_eq!(80, committee.len());
            assert_eq!(&slot_0_committee, committee);

        }
        {
            let slot_74_committee: Vec<u32> = vec![
                50, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 649, 698, 747, 796,
                845, 894, 943, 992, 33, 84, 134, 184, 234, 284, 334, 384, 434, 484, 534, 584,
                633, 682, 731, 780, 829, 878, 927, 976, 17, 68, 118, 168, 218, 268, 318, 368,
                418, 468, 518, 568, 618, 667, 716, 765, 814, 863, 912, 961, 2, 53, 103, 153,
                203, 253, 303, 353, 403, 453, 503, 553, 603, 652, 701, 750, 799, 848, 897, 946
            ];

            let slot_74 = slots[74].clone();
            let committee = slot_74.get_committee();

            assert_eq!(49, slot_74.leader);
            assert_eq!(&slot_74_committee, committee);

        }
    }
}
