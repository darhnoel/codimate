//! Python's `random.Random`, bit for bit, for what markers use: seeding from a
//! string, `random()`, `uniform()` and `choice()` between two things. A
//! marker's tilt and ragged ends are drawn from it, seeded by the marker's
//! name, so they must be the numbers Python drew or every film's markers move.
//!
//! The generator is MT19937 (Matsumoto and Nishimura), seeded as CPython's
//! `random.seed(str)` does: the string's bytes followed by their SHA-512,
//! read as one big-endian integer, fed to `init_by_array` as 32-bit words,
//! least significant first.

use sha2::{Digest, Sha512};

const N: usize = 624;
const M: usize = 397;

pub struct Random {
    state: [u32; N],
    index: usize,
}

impl Random {
    /// `random.Random(seed)` for a string seed.
    pub fn new(seed: &str) -> Random {
        let mut bytes = seed.as_bytes().to_vec();
        bytes.extend_from_slice(&Sha512::digest(seed.as_bytes()));
        // The integer's 32-bit words, least significant first; no leading zeros.
        let start = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
        let digits = &bytes[start..];
        let mut key: Vec<u32> = digits
            .rchunks(4)
            .map(|chunk| chunk.iter().fold(0u32, |w, b| (w << 8) | *b as u32))
            .collect();
        if key.is_empty() {
            key.push(0);
        }
        let mut rng = Random {
            state: [0; N],
            index: N,
        };
        rng.init_by_array(&key);
        rng
    }

    fn init_genrand(&mut self, s: u32) {
        self.state[0] = s;
        for i in 1..N {
            let prev = self.state[i - 1];
            self.state[i] = 1812433253u32
                .wrapping_mul(prev ^ (prev >> 30))
                .wrapping_add(i as u32);
        }
        self.index = N;
    }

    fn init_by_array(&mut self, key: &[u32]) {
        self.init_genrand(19650218);
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..N.max(key.len()) {
            let prev = self.state[i - 1];
            self.state[i] = (self.state[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1664525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                self.state[0] = self.state[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            let prev = self.state[i - 1];
            self.state[i] = (self.state[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1566083941))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                self.state[0] = self.state[N - 1];
                i = 1;
            }
        }
        self.state[0] = 0x8000_0000;
    }

    fn next_u32(&mut self) -> u32 {
        if self.index >= N {
            for k in 0..N {
                let y = (self.state[k] & 0x8000_0000) | (self.state[(k + 1) % N] & 0x7fff_ffff);
                let mut next = self.state[(k + M) % N] ^ (y >> 1);
                if y & 1 != 0 {
                    next ^= 0x9908_b0df;
                }
                self.state[k] = next;
            }
            self.index = 0;
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    /// `random()`: a float in [0, 1) from 53 random bits.
    pub fn random(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as f64;
        let b = (self.next_u32() >> 6) as f64;
        (a * 67108864.0 + b) * (1.0 / 9007199254740992.0)
    }

    /// `uniform(a, b)`.
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }

    /// `choice((first, second))`'s index: `_randbelow(2)` by rejection.
    pub fn choose_of_two(&mut self) -> usize {
        loop {
            let r = (self.next_u32() >> 30) as usize; // getrandbits(2)
            if r < 2 {
                return r;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Random;

    #[test]
    fn draws_the_numbers_python_draws() {
        let cases: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../tests/python_random.json")).unwrap();
        for case in cases {
            let seed = case["seed"].as_str().unwrap();
            let mut rng = Random::new(seed);
            assert_eq!(
                rng.uniform(0.5, 1.5),
                case["uniform"].as_f64().unwrap(),
                "{seed:?}"
            );
            assert_eq!(
                rng.choose_of_two(),
                case["choice_index"].as_u64().unwrap() as usize
            );
            for bite in case["bites"].as_array().unwrap() {
                assert_eq!(rng.uniform(0.3, 1.0), bite.as_f64().unwrap());
            }
            let mut fresh = Random::new(seed);
            for value in case["random"].as_array().unwrap() {
                assert_eq!(fresh.random(), value.as_f64().unwrap(), "{seed:?}");
            }
        }
    }
}
