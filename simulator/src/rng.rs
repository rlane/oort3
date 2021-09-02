pub use rand_chacha::ChaCha8Rng as SeededRng;

pub fn new_rng(seed: u32) -> SeededRng {
    rand_seeder::Seeder::from(seed).make_rng()
}
