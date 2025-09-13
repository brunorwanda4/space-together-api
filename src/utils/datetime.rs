use regex::Regex;

pub fn validate_datetime(input: &str) -> Result<(), String> {
    let datetime_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,3})?Z$").unwrap();
    if datetime_regex.is_match(input) {
        Ok(())
    } else {
        let mut error_message = String::new();
        if !Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap().is_match(input) {
            error_message.push_str("Invalid date format (YYYY-MM-DD).\n");
        }
        if !Regex::new(r"T\d{2}:\d{2}:\d{2}").unwrap().is_match(input) {
            error_message.push_str("Invalid time format (HH:MM:SS).\n");
        }
        if !input.ends_with('Z') {
            error_message.push_str("Must end with 'Z' (UTC timezone).\n");
        }
        error_message.push_str("Correct format: YYYY-MM-DDTHH:MM:SS.sssZ\n");
        Err(error_message)
    }
}

pub fn is_date_string(date: &str) -> bool {
    let datetime_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$").unwrap();
    datetime_regex.is_match(date)
}
