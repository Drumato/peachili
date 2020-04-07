type CheckChar = fn(ch: &char) -> bool;

pub fn take_conditional_string(s: &str, f: CheckChar) -> String {
    s.chars().take_while(f).collect::<String>()
}

pub fn read_program_from_file(path: &str) -> String {
    use std::fs;

    let result_contents = fs::read_to_string(path);

    if result_contents.is_err() {
        panic!("read {} failed.", path);
    }

    result_contents.unwrap()
}
