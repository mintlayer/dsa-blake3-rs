extern crate hex;

use std::collections::BTreeMap;
type Participant = u32;

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

const MAX_PARTICIPANTS: usize = 1008;
const MAX_COMMITTEE: usize = 80;

#[derive(Debug, Clone)]
struct FinalSlot {
    slot:Slot,
    committee: [Participant;MAX_COMMITTEE]
}


fn total(stakes: &Vec<Stake>) -> u32 {
    // we don't care about overflow for the test
    stakes.iter().map(|slot| slot.weight).sum()
}

const MIN_STAKE: u32 = 440000;
const BTC: &'static str = "0000000000000000000d9ed0f796aeee51b200c7293a6e31c101a0e4159bf310";

fn print_type_of<T>(_: &T) {
    println!("TYPE: {}", std::any::type_name::<T>())
}

fn dsa(mut stakes: Vec<Stake>, slots: Vec<Slot>) -> Vec<Slot> {
    let mut ret = slots.clone();

    for (i, _slot) in slots.iter().enumerate() {
        let hash = hex::decode(BTC).expect("Decoding failed");
        let mut rnd = rand(i as u8, hash, total(&stakes));

        for (_, stake) in stakes.iter_mut().enumerate() {
            if stake.weight >= rnd {
                if stake.weight >= MIN_STAKE {
                    stake.weight -= MIN_STAKE;
                } else {
                    stake.weight = 0;
                }
                ret[i].key = stake.id;
                break;
            } else {
                rnd -= stake.weight;
            }
        }
    }
    ret
}

fn dsa_two(mut stakes: Vec<Stake>, slots: Vec<Slot>) -> Vec<FinalSlot> {
    use std::convert::TryInto;

    let mut ret:Vec<FinalSlot> =vec![];

    // avoiding participant to be a blocksigner on adjacent slots,
    // giving chance to the other participants who are not yet chosen.
    let mut is_in_committee: BTreeMap<Participant,()> = BTreeMap::new();

    for (i, slot) in slots.iter().enumerate() {

        // clear up the list, when most participants had been a blocksigner.
        if is_in_committee.len() > stakes.len()-MAX_COMMITTEE {
            println!("\nemptying the is_in_committee checker...");
            is_in_committee = BTreeMap::new();
        }

        let hash = hex::decode(BTC).expect("Decoding failed");
        let mut rnd = rand(i as u8, hash, total(&stakes));

        // help in choosing the committee; based on the rnd variable.
        let mut committee_diff:i32 = 0;

        // choosing a committee based on this list.
        // BTreeMap used, ensuring that the participant of the least calculated committee_diff
        // will be chosen.
        let mut pot_committee:BTreeMap<i32,Participant> = BTreeMap::new();

        let mut leader:Option<Participant> = None;

        for (_, stake) in stakes.iter_mut().enumerate() {

            if leader.is_none() && stake.weight >= rnd {
                if stake.weight >= MIN_STAKE {
                    stake.weight -= MIN_STAKE;
                } else {
                    stake.weight = 0;
                }
                leader = Some(stake.id.clone());

                println!("\nslot:{} leader:{}", slot.id, stake.id);
            } else {

                // continued computation to get the committee.
                if let Some(updated_rnd) =  rnd.checked_sub(stake.weight) {
                    rnd = updated_rnd;
                    committee_diff = rnd as i32;
                } else {
                    committee_diff = rnd as i32 - stake.weight as i32;
                }

                // add participant to the potential committee ONLY if it hasn't been a blocksigner
                // on the previous slots.
                if ! is_in_committee.contains_key(&stake.id) {
                    pot_committee.insert(committee_diff, stake.id);
                }
            }
        }

        let mut committee:Vec<Participant> = vec![];

        // iterate over potential committee
        let mut pot_iter = pot_committee.into_iter();

        // keep iterating until the maximum number of participants in the committee
        // has been fulfilled.
        while committee.len() < MAX_COMMITTEE {
            if let Some((_,participant)) = pot_iter.next() {
                committee.push(participant);
                is_in_committee.insert(participant,());
            } else {
                println!("  warning: no more participants.");
                break;
            }
        }

        println!("  committee: {:?}", committee);

        // update the slot with the leader and the committee
        let mut updated_slot = slot.clone();
        updated_slot.key = leader.unwrap();

        ret.push(FinalSlot {
            slot: updated_slot,
            committee: committee.try_into().expect("slice with incorrect length")
        });
    }

    ret
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
        let mut slots:Vec<Slot> = vec![];
        let mut stakes:Vec<Stake> = vec![];

        for i in 0 .. MAX_PARTICIPANTS {
            slots.push(Slot {
                id: i as u32,
                key: 0
            });

            stakes.push( Stake {
                id: i as u32,
                weight: MIN_STAKE + i as u32 *2
            })
        }

        let tot = total(&stakes);
        let result = dsa_two(stakes, slots);
        assert!(true);

        //TODO: add more asserts
    }
}
