/// pathから内容を読み込み，Stringを返す
/// ファイルが存在しなかったとき，None
pub fn read_program_from_file(path: &str) -> Option<String> {
    use std::fs;

    let result_contents = fs::read_to_string(path);

    if result_contents.is_err() {
        return None;
    }

    Some(result_contents.unwrap())
}