use chrono::{Datelike, Utc};
use rand::{seq::SliceRandom, thread_rng};

use crate::domain::school::School;

pub fn generate_code() -> String {
    let mut rng = thread_rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();

    (0..5)
        .map(|_| *chars.choose(&mut rng).unwrap())
        .collect()
}

pub fn generate_school_registration_number(school: &School) -> Option<String> {
    let year = Utc::now().year();
    let random = rand::random::<u16>() % 10000;
    Some(format!("{}-{}-{:04}", school.username, year, random))
}
