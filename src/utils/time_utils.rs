 use regex::Regex;

pub fn is_valid_hhmm(value: &str) -> bool {
    static TIME_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

    let re = TIME_RE.get_or_init(|| {
        Regex::new(r"^([01]\d|2[0-3]):([0-5]\d)$").expect("HH:MM regex failed")
    });

    re.is_match(value)
}
