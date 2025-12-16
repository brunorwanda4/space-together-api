use chrono::{Datelike, Utc};
use rand::{rng, seq::IndexedRandom};

use crate::domain::school::School;

pub fn generate_code() -> String {
    let mut rng = rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();

    // `.choose(&mut rng)` works on slices, so borrow as slice
    (0..5)
        .map(|_| *chars.as_slice().choose(&mut rng).unwrap())
        .collect()
}

pub async fn generate_school_registration_number(school: &School) -> Option<String> {
    let year = Utc::now().year();
    let random = rand::random::<u16>() % 10000;
    Some(format!("{}-{}-{:04}", school.username, year, random))
}
