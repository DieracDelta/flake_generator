use regex::Regex;

pub fn translate_url(s: String) {
    let re = Regex::new(r"^(github|gitlab)$").unwrap();
}
