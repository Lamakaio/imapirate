use std::hash::Hasher;

use seahash::SeaHasher;

//a struct to provide a seeded hasher. I doesn't expose the underlying hasher intentionnally to make sure nothing breaks it.
pub struct SeededHasher {
    hasher: SeaHasher,
}

impl SeededHasher {
    pub fn new(seed: u64) -> SeededHasher {
        let mut hasher = SeaHasher::new();
        hasher.write_u64(seed);
        SeededHasher { hasher }
    }

    pub fn get_hasher(&self) -> SeaHasher {
        self.hasher.clone()
    }
}
