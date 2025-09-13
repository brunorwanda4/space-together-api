use itertools::Itertools;
use rand::{rng, seq::SliceRandom};
use regex::Regex;

pub fn is_valid_name(name: &str) -> Result<String, String> {
    let re = Regex::new(r"^[a-zA-Z\s\-'\.,]*$").unwrap();
    if re.is_match(name) {
        let word_count = name.split_whitespace().count();
        if word_count >= 2 {
            Ok(name.to_string())
        } else {
            Err(
                "Name is valid but not a full name. Please provide both first and last names."
                    .to_string(),
            )
        }
    } else {
        let invalid_chars: String = name
            .chars()
            .filter(|&c| {
                !Regex::new(r"[a-zA-Z\s\-'\.,]")
                    .unwrap()
                    .is_match(&c.to_string())
            })
            .collect();

        let corrected_name: String = name
            .chars()
            .filter(|&c| {
                Regex::new(r"[a-zA-Z\s\-'\.,]")
                    .unwrap()
                    .is_match(&c.to_string())
            })
            .collect();

        if !corrected_name.is_empty() {
            Err(format!(
                "contains disallowed characters [{}]. Suggested name: '{}'.",
                invalid_chars, corrected_name
            ))
        } else {
            Err(format!(
                "contains disallowed characters [{}]. Please try another name.",
                invalid_chars
            ))
        }
    }
}

pub fn generate_username(name: &str) -> String {
    let mut rng = rng();

    let binding = name.to_lowercase();
    let words: Vec<&str> = binding.split_whitespace().collect();
    let mut possible_usernames = Vec::new();

    for permutation in words.iter().permutations(words.len()) {
        let joined = permutation.into_iter().join("_");
        possible_usernames.push(joined.clone());
        possible_usernames.push(format!("_{}", joined));
    }

    possible_usernames.shuffle(&mut rng);

    let username = possible_usernames
        .first()
        .unwrap_or(&words.join("_"))
        .to_string();

    format!("{}_{}", username, rand::random::<u8>())
}

pub fn is_valid_username(username: &str) -> Result<String, String> {
    let re = Regex::new(r"^[a-zA-Z0-9_-]*$").unwrap();
    if re.is_match(username) {
        Ok(username.to_string())
    } else {
        let invalid_chars: String = username
            .chars()
            .filter(|&c| !re.is_match(&c.to_string()))
            .collect();
        let corrected_username: String = username
            .chars()
            .filter(|&c| re.is_match(&c.to_string()))
            .collect();

        if !corrected_username.is_empty() {
            Err(format!(
                "Invalid username: [{}]. Suggested: '{}'.",
                invalid_chars, corrected_username
            ))
        } else {
            Err(format!(
                "Invalid username: [{}]. Please try another name.",
                invalid_chars
            ))
        }
    }
}
