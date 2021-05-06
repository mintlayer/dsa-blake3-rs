extern crate hex;


use std::collections::BTreeMap;
use std::convert::TryInto;


const MAX_PARTICIPANTS: usize = 1008;
const MAX_COMMITTEE: usize = 80;

const MIN_STAKE: u32 = 440000;
const BTC: &'static str = "0000000000000000000d9ed0f796aeee51b200c7293a6e31c101a0e4159bf310";

type Participant = u32;
type Committee = [Participant;MAX_COMMITTEE];

#[derive(Debug, Clone)]
struct Stake {
    id: Participant,
    weight: u32,
}

#[derive(Debug, Clone)]
struct Slot {
    id: u32,
    key: Participant,
}

#[derive(Debug, Clone)]
struct Slot2 {
    id: u32,
    key: Participant,
    committee: Committee

}

impl Default for Slot2 {
    fn default() -> Self {
        Self {
            id: 0,
            key: 0,
            committee: [0;MAX_COMMITTEE]
        }
    }
}


fn total(stakes: &Vec<Stake>) -> u32 {
    // we don't care about overflow for the test
    stakes.iter().map(|slot| slot.weight).sum()
}


fn print_type_of<T>(_: &T) {
    println!("TYPE: {}", std::any::type_name::<T>())
}

fn random_committee(mut stakes: Vec<Stake>, slot_id:u8) -> Vec<Participant> {
    let hash = hex::decode(BTC).expect("Decoding failed");
    let mut total_stakes = total(&stakes);
    let mut rnd = rand(slot_id, hash.clone(), total_stakes);

    let mut committee:Vec<Participant> = vec![];

    while committee.len() < MAX_COMMITTEE {
        // println!(" -> need {} more in the committee; participants left: {}", MAX_COMMITTEE - committee.len(), stakes.len());

        stakes.retain(|stake| {
            if committee.len() == MAX_COMMITTEE {
                // if committee has been filled, don't bother checking for the others.
                false
            } else {
                if stake.weight >= rnd {

                    committee.push(stake.id);

                    //deduct the chosen participant's stakes to the total.
                    total_stakes -= stake.weight;

                    //recalculate the random number
                    rnd = rand(slot_id,hash.clone(),total_stakes);

                    false
                } else {
                    rnd -= stake.weight;
                    // println!(" -> stake {} loops to next round",stake.id);
                    true
                }
            }
        });

    }

    // println!("  -> committee: {:?}", committee);

    committee
}

fn dsa(mut stakes: Vec<Stake>, slots: Vec<Slot>) -> Vec<Slot> {
    let mut ret = slots.clone();

    for (i, _slot) in slots.iter().enumerate() {
        let hash = hex::decode(BTC).expect("Decoding failed");
        let mut rnd = rand(i as u8, hash, total(&stakes));
        println!("\nSLOT {}",i);

        let mut stakes_clone = stakes.clone();
        for (j, stake) in stakes_clone.iter_mut().enumerate() {

            if stake.weight >= rnd {
                if stake.weight >= MIN_STAKE {
                    stake.weight -= MIN_STAKE;
                } else {
                    stake.weight = 0;
                }
                println!("  ---> stake {} is leader!",stake.id);
                ret[i].key = stake.id;

                let mut stake_without_j = stakes.clone();
                stake_without_j.remove(j);

                random_committee(stake_without_j,i as u8);

                break;
            } else {
                rnd -= stake.weight;
            }
        }
    }
    ret
}


fn dsa2( slots:&mut Vec<Slot2>, mut stakes: Vec<Stake>) {
    for (i, slot) in slots.into_iter().enumerate() {
        let hash = hex::decode(BTC).expect("Decoding failed");
        let mut rnd = rand(i as u8, hash, total(&stakes));
        // println!("\nSLOT {}",i);

        for (j, stake) in stakes.iter_mut().enumerate() {

            if stake.weight >= rnd {
                if stake.weight >= MIN_STAKE {
                    stake.weight -= MIN_STAKE;
                } else {
                    stake.weight = 0;
                }
                // println!(" -> stake {} is leader!",stake.id);
                slot.id = i as u32;
                slot.key = stake.id;

                // generate a committee with the same algo, but now without the leader's stakes.
                let mut stake_without_j = stakes.clone();
                stake_without_j.remove(j);
                let committee = random_committee(stake_without_j,i as u8);
                slot.committee = committee.try_into().expect("wrong length of slice");

                break;
            } else {
                rnd -= stake.weight;
            }
        }
    }
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

        let slots = vec![
            Slot { id: 0, key: 0 },
            Slot { id: 1, key: 0 },
            Slot { id: 2, key: 0 },
            Slot { id: 3, key: 0 },
            Slot { id: 4, key: 0 },
            Slot { id: 5, key: 0 },
            Slot { id: 6, key: 0 },
            Slot { id: 7, key: 0 },
            Slot { id: 8, key: 0 },
            Slot { id: 9, key: 0 },
        ];

        let tot = total(&stakes);
        let result = dsa(stakes, slots);
        assert_eq!(9840002, tot);
        assert_eq!(8, result[0].key);
        assert_eq!(5, result[1].key);
        assert_eq!(8, result[2].key);
        assert_eq!(3, result[3].key);
        assert_eq!(8, result[4].key);
        assert_eq!(7, result[5].key);
        assert_eq!(5, result[6].key);
        assert_eq!(5, result[7].key);
        assert_eq!(8, result[8].key);
        assert_eq!(4, result[9].key);
    }


    #[test]
    fn test_committee() {
        let mut slots:Vec<Slot2> = vec![];
        let mut stakes:Vec<Stake> = vec![];

        for i in 0 .. MAX_PARTICIPANTS {
            slots.push(Default::default());

            stakes.push( Stake {
                id: i as u32,
                weight: MIN_STAKE + i as u32 *2
            })
        }

        let tot = total(&stakes);
        dsa2(&mut slots,stakes);

        assert!(true);

        let slot_0_committee = [
            827, 647, 467, 287, 107, 936, 755, 575, 395, 215, 35, 864, 683,
            503, 323, 143, 972, 791, 611, 431, 251, 71, 900, 719, 539, 359,
            179, 1007, 826, 646, 466, 286, 106, 935, 754, 574, 394, 214, 34,
            863, 682, 502, 322, 142, 971, 790, 610, 430, 250, 70, 899, 718,
            538, 358, 178, 1006, 825, 645, 465, 285, 105, 934, 753, 573, 393,
            213, 33, 862, 681, 501, 321, 141, 970, 789, 609, 429, 249, 69, 898, 717];

        let slot_0 = slots[0].clone();


        assert_eq!(828,slot_0.key);
        assert_eq!(slot_0_committee,slot_0.committee)
    }
}
