use std::{sync::Arc};
use sha2::{Sha256, Digest};

/*
Current minimum
(paoloose/dot/site/aABdDefFhHiIjKlLMnNpqrRsTUvVwW): 000000000040558f0e351ca906f13d46a7436b391e4fd0f174bc91a82fb037ea
*/

// Available combinations
const CHARS: &[u8; 64] = b"aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ0123456789+/";

// I designed this miner to use a range of values so i can "resume" its execution, and thanks
// to this we can guarantee we never calculate the same sha two times
const MIN_NUM: usize = 50000000000000;
const SKIP_ITER: usize = 100000000000;
const MAX_NUM: usize = 99999999999999;
const SHA_PREFIX: &str = "paoloose/dot/site/";

fn main() {
    let n_threads = std::thread::available_parallelism().unwrap().into();

    // Global variable to store the minimum sha that is currently known
    let global_min_sha_arc = Arc::new(std::sync::RwLock::new(u128::MAX));

    let mut handles = Vec::with_capacity(n_threads);
    let mut random_str_container = [0_u8; 64];

    for t in 0..n_threads {
        let global_min_sha = global_min_sha_arc.clone();

        // We define the slices that each thread will handle
        let total_tries = MAX_NUM - MIN_NUM;
        let start_loop = MIN_NUM + (t * (total_tries / n_threads)) + SKIP_ITER;
        let start_loop = usize::max(start_loop, 1); // avoid zero
        let end_loop = MIN_NUM + ((t + 1) * (total_tries / n_threads));

        let handle = std::thread::spawn(move || {
            let mut hasher = Sha256::new();
            let mut local_min_sha = u128::MAX;

            // For each number, we translate that number to a set of random characters
            // to create the input that will be hashed
            for c in start_loop..end_loop {
                let mut current_char = 0;
                // Each bit decided whether to include the char of CHARS (1 means include)
                for (b, ch) in CHARS.iter().enumerate() {
                    if (((c >> b) & 1) as u8) == 1 {
                        random_str_container[current_char] = *ch;
                        current_char += 1;
                    }
                }

                // And we hash the string prepended with the prefix
                let random_str = unsafe {
                    str::from_utf8_unchecked(&random_str_container[0..current_char])
                };

                let to_hash = format!("{SHA_PREFIX}{random_str}");
                hasher.update(to_hash.as_bytes());
                let sha256_result = hasher.finalize_reset();

                // We only take the first 128 bits as it is more than enough to get the podium :p
                let halfsha = unsafe { *(sha256_result.as_ptr() as *const u128) };
                let halfsha = u128::from_be(halfsha.to_le());

                // And we store the minimums
                if halfsha < local_min_sha {
                    local_min_sha = halfsha;

                    let global_min_sha_val = global_min_sha.read().unwrap();

                    if local_min_sha < *global_min_sha_val {
                        drop(global_min_sha_val);
                        let mut global_min_sha_set = global_min_sha.write().unwrap();
                        *global_min_sha_set = local_min_sha;
                        println!("[t:{t}] New min ({c}) ({to_hash}): {:x}", sha256_result);
                    }
                }
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
}
