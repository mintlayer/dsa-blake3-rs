extern crate hex;

#[derive(Debug, Clone)]
struct Stake {
    id: u32,
    weight: u32,
}

#[derive(Debug, Clone)]
struct Slot {
    id: u32,
    key: u32,
}

fn total(stakes: &Vec<Stake>) -> u32 {
    // we don't care about overflow for the test
    stakes.iter().map(|slot| slot.weight).sum()
}

const MIN_STAKE: u32 = 440000;
const MAX_VAL: u32 = 4294967295;
const BTC: &'static str = "0000000000000000000d9ed0f796aeee51b200c7293a6e31c101a0e4159bf310";

fn print_type_of<T>(_: &T) {
    println!("TYPE: {}", std::any::type_name::<T>())
}

fn dsa(stakes: Vec<Stake>, slots: Vec<Slot>) -> Vec<Slot> {
    let mut stakes_copy = stakes.clone();
    let mut ret = slots.clone();

    for (i, _slot) in slots.iter().enumerate() {
        let hash = hex::decode(BTC).expect("Decoding failed");
        let mut rnd = rand(i as u8, hash, total(&stakes));

        for (el_id, stake) in stakes.iter().enumerate() {
            if stake.weight >= rnd {
                if stake.weight >= MIN_STAKE {
                    stakes_copy[el_id].weight -= MIN_STAKE;
                } else {
                    stakes_copy.remove(el_id);
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

fn rand(i: u8, hash: Vec<u8>, max: u32) -> u32 {
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
    ((max as f64) / (MAX_VAL as f64) * ret as f64) as u32
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
        assert_eq!(6, result[1].key);
        assert_eq!(8, result[2].key);
        assert_eq!(3, result[3].key);
        assert_eq!(8, result[4].key);
        assert_eq!(8, result[5].key);
        assert_eq!(5, result[6].key);
        assert_eq!(6, result[7].key);
        assert_eq!(8, result[8].key);
        assert_eq!(5, result[9].key);
    }
}
