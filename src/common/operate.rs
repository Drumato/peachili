type CheckChar = fn(ch: &char) -> bool;

pub fn take_conditional_string(s: &str, f: CheckChar) -> String {
    s.chars().take_while(f).collect::<String>()
}
