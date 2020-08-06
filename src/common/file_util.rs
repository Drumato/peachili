use std::fs;
use std::io::Write;

/// pathから内容を読み込み，Stringを返す
/// ファイルが存在しなかったとき，None
pub fn read_program_from_file(path: &str) -> Option<String> {
    let result_contents = fs::read_to_string(path);

    if result_contents.is_err() {
        return None;
    }

    Some(result_contents.unwrap())
}

/// path で新規にファイルを作成し，programを書き込む
pub fn write_program_into(path: &str, program: String) {
    let mut file = fs::File::create(path).unwrap();
    file.write_all(program.as_bytes()).unwrap();
    file.flush().unwrap();
}