extern crate hex;

use log::{debug, info};

const MIN_STAKE: u32 = 440000;
const BTC: &'static str = "0000000000000000000d9ed0f796aeee51b200c7293a6e31c101a0e4159bf310";

type Participant = u32;
type Committee = Vec<Participant>;

#[derive(Debug, Clone)]
struct Stake {
    id: Participant,
    weight: u32,
}

#[derive(Debug, Clone)]
struct Slot {
    id: u32,
    key: Participant,
    committee: Committee,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            id: 0,
            key: 0,
            committee: vec![],
        }
    }
}

impl Slot {
    fn set_committee(&mut self, mut stakes: Vec<Stake>, max_committee: usize) {
        // we lack the number of stakes, or the max_committee set is too big
        if max_committee >= stakes.len() {
            info!("stakes not enough to fill the committee.");
            self.committee = stakes.iter().map(|stake| stake.id).collect();
        } else {
            let hash = hex::decode(BTC).expect("Decoding failed");

            let mut total_stakes = total(&stakes);
            let mut rnd = rand(self.id as u8, hash.clone(), total_stakes);

            let mut committee: Vec<Participant> = vec![];

            while committee.len() < max_committee {
                debug!(
                    " -> need {} more in the committee; participants left: {}",
                    max_committee - committee.len(),
                    stakes.len()
                );

                stakes.retain(|stake| {
                    if committee.len() == max_committee {
                        // if committee has been filled, don't bother checking for the others.
                        false
                    } else {
                        if stake.weight >= rnd {
                            committee.push(stake.id);

                            //deduct the chosen participant's stakes from the total.
                            total_stakes -= stake.weight;

                            //recalculate the random number
                            rnd = rand(self.id as u8, hash.clone(), total_stakes);

                            false
                        } else {
                            debug!(" -> stake {} loops to next round", stake.id);

                            rnd -= stake.weight;

                            true
                        }
                    }
                });
            }

            debug!("  -> committee: {:?}", committee);
            self.committee = committee;
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

fn dsa(slots: &mut Vec<Slot>, mut stakes: Vec<Stake>, max_committee: usize) {
    for (i, slot) in slots.into_iter().enumerate() {
        let hash = hex::decode(BTC).expect("Decoding failed");
        let mut rnd = rand(i as u8, hash, total(&stakes));

        for (j, stake) in stakes.iter_mut().enumerate() {
            if stake.weight >= rnd {
                if stake.weight >= MIN_STAKE {
                    stake.weight -= MIN_STAKE;
                } else {
                    stake.weight = 0;
                }

                debug!("slot:{} leader:{}", slot.id, stake.id);
                slot.id = i as u32;
                slot.key = stake.id;

                // generate a committee with the same algo, but now without the leader's stakes.
                let mut stake_without_j = stakes.clone();
                stake_without_j.remove(j);

                slot.set_committee(stake_without_j, max_committee);

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
        let max_committee = 80;
        let max_participants = 1008;

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

        let mut slots: Vec<Slot> = vec![];

        for i in 0..max_participants {
            let mut slot: Slot = Default::default();
            slot.id = i;

            slots.push(slot);
        }

        let tot = total(&stakes);
        dsa(&mut slots, stakes, max_committee);
        assert_eq!(9840002, tot);
        assert_eq!(8, slots[0].key);
        assert_eq!(5, slots[1].key);
        assert_eq!(8, slots[2].key);
        assert_eq!(3, slots[3].key);
        assert_eq!(8, slots[4].key);
        assert_eq!(7, slots[5].key);
        assert_eq!(5, slots[6].key);
        assert_eq!(5, slots[7].key);
        assert_eq!(8, slots[8].key);
        assert_eq!(4, slots[9].key);
    }

    #[test]
    fn test_committee() {
        let max_committee = 80;
        let max_participants = 1008;

        let mut slots: Vec<Slot> = vec![];
        let mut stakes: Vec<Stake> = vec![];

        for i in 0..max_participants {
            if i < max_committee {
                let mut slot: Slot = Default::default();
                slot.id = i;

                slots.push(slot);
            }

            stakes.push(Stake {
                id: i as u32,
                weight: MIN_STAKE + i as u32 * 2,
            })
        }

        dsa(&mut slots, stakes, max_committee as usize);

        {
            let slot_0_committee: Vec<u32> = vec![
                827, 647, 467, 287, 107, 936, 755, 575, 395, 215, 35, 864, 683, 503, 323, 143, 972,
                791, 611, 431, 251, 71, 900, 719, 539, 359, 179, 1007, 826, 646, 466, 286, 106, 935,
                754, 574, 394, 214, 34, 863, 682, 502, 322, 142, 971, 790, 610, 430, 250, 70, 899, 718,
                538, 358, 178, 1006, 825, 645, 465, 285, 105, 934, 753, 573, 393, 213, 33, 862, 681,
                501, 321, 141, 970, 789, 609, 429, 249, 69, 898, 717,
            ];

            let slot_0 = slots[0].clone();

            assert_eq!(828, slot_0.key);
            assert_eq!(slot_0_committee, slot_0.committee);

        }
        {
            let slot_74_committee: Vec<u32> = vec![
                50, 101, 151, 203, 251, 298, 347, 394, 445, 496, 546, 594, 646, 695, 747, 796, 848,
                895, 945, 990, 30, 84, 134, 184, 233, 280, 329, 376, 426, 475, 529, 577, 627, 677,
                725, 779, 830, 879, 929, 974, 13, 63, 117, 167, 216, 264, 311, 360, 407, 459, 510,
                560, 607, 660, 708, 760, 811, 861, 912, 958, 1005, 45, 97, 147, 199, 247, 293, 342,
                390, 440, 489, 542, 590, 641, 691, 742, 792, 844, 891, 941
            ];

            let slot_74 = slots[74].clone();

            assert_eq!(49, slot_74.key);
            assert_eq!(slot_74_committee, slot_74.committee);

        }


    }
}
