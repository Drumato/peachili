type CheckChar = fn(ch: &char) -> bool;

pub fn count_length(s: &str, f: CheckChar) -> usize {
    take_conditional_string(s, f).len()
}

pub fn take_conditional_string(s: &str, f: CheckChar) -> String {
    s.chars().take_while(f).collect::<String>()
}
