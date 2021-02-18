use std::hash::Hash;

use seahash::SeaHasher;

//a struct to provide a seeded hasher. I doesn't expose the underlying hasher intentionnally to make sure nothing breaks it.
pub struct SeededHasher {
    hasher: SeaHasher,
}

impl SeededHasher {
    pub fn new(seed: &str) -> SeededHasher {
        let mut hasher = SeaHasher::new();
        seed.hash(&mut hasher);
        SeededHasher { hasher }
    }

    pub fn get_hasher(&self) -> SeaHasher {
        self.hasher.clone()
    }
}
