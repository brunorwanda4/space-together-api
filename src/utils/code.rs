use rand::{rng, seq::IndexedRandom};

pub fn generate_code() -> String {
    let mut rng = rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();

    // `.choose(&mut rng)` works on slices, so borrow as slice
    (0..5)
        .map(|_| *chars.as_slice().choose(&mut rng).unwrap())
        .collect()
}
