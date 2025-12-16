use regex::Regex;

pub fn is_valid_email(email: &str) -> Result<String, String> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if email_regex.is_match(email) {
        Ok(email.to_string())
    } else {
        let invalid_chars: String = email
            .chars()
            .filter(|&c| !email_regex.is_match(&c.to_string()))
            .collect();
        Err(format!(
            "Invalid email: [{}]. Please provide a valid email.",
            invalid_chars
        ))
    }
}
