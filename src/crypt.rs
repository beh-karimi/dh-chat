pub mod dh {
    use std::collections::HashSet;
    use sha2::{Sha256, Digest};
    use rand_chacha::ChaCha20Rng;
    use rand::prelude::*;

    #[derive(Debug)]
    pub struct KeyPair {
        pub private: u64,
        pub public: u64,
        pub modulus : u64,
        pub gen: u64,
    }

    pub fn mod_exp(base: u64, exp: u64, modulus: u64) -> u64 {
        let mut result = 1;
        for _ in 0..exp {
            result = result * base % modulus;
        }
        result
    }

    pub fn get_prime_factors(n: u64) -> HashSet<u64> {
        let mut factors = HashSet::new();

        let upperbound = (n as f64).sqrt() as u64 + 1;
        let mut primes = vec![];

        for i in 2..upperbound {
            let mut is_prime = true;

            for p in &primes {
                if i%p == 0 {
                    is_prime = false;
                    break;
                }
            }

            if is_prime { primes.push(i); }
        }

        for p in &primes {
            if n%p==0 {
                factors.insert(*p);
                
                let mut x = n / *p;
                for p2 in &primes {
                    while x%p2 == 0 {
                        x /= p2;
                    }
                    if x==1 { break; }
                }

                if x>1 { factors.insert(x); }
            }
        }

        factors
    }

    pub fn check_gen(n: u64, modulus: u64, exps: &Vec<u64>) -> bool {
        let mut current_exp = 0;
        let mut acc = 1;
        for exp in exps {
            for _ in 0..exp-current_exp {
                acc = acc*n % modulus;
            }
            if acc==1 { return false; }
            current_exp = *exp;
        }
        true
    }

    pub fn get_gen(n: u64) -> Vec<u64>{
        let mut res = vec![];

        let phi_factors = get_prime_factors(n-1);
        let mut exps: Vec<u64> = phi_factors.iter().map(|x| (n-1)/x).collect();
        exps.sort();

        for i in n/3..n*2/3 {
            if check_gen(i, n, &exps) {
                res.push(i);
                if res.len() > 80 {
                    break;
                }
            }
        }

        res
    }

    #[allow(dead_code)]
    fn gen_prime(min: u64) -> u64 {
        let mut rng = ChaCha20Rng::from_entropy();
        let min = rng.gen_range(0..min);
        
        let mut i = 2;
        let mut primes = vec![2];
        loop {
            let mut is_prime = true;
            for p in &primes {
                if i%p==0 {
                    is_prime = false;
                    break;
                }
            }
            if is_prime {
                if i >= min { return i;}
                primes.push(i);
            }
            i += 1;
        }
    }

    pub fn gen_key_pair(modulus: u64, generator: u64) -> KeyPair {
        let mut rng = ChaCha20Rng::from_entropy();
        let k_prv = rng.gen_range(0..modulus);
        let k_pub = mod_exp(generator, k_prv, modulus);

        KeyPair { private: k_prv, public: k_pub, modulus, gen: generator }
    }

    pub fn gen_common_key(modulus: u64, self_key: u64, other_key: u64) -> Vec<u8> {
        let shared_secret = mod_exp(other_key, self_key, modulus);
        Sha256::digest(shared_secret.to_be_bytes()).to_vec()
    }
}

pub fn encrypt(key: &[u8], msg: &str) -> Vec<u8> {
    let mut cipher = vec![0; msg.len()];
    let key_len = key.len();
    for (i, b) in msg.bytes().enumerate() {
        cipher[i] = b ^ key[i%key_len];
    }
    cipher
}

pub fn decrypt(key: &[u8], cipher: &[u8]) -> String {
    let mut msg = String::new();
    msg.reserve(cipher.len());
    let key_len = key.len();
    for (i, b) in cipher.iter().enumerate() {
        msg.push((b ^ key[i%key_len]) as char);
    }
    msg
}
