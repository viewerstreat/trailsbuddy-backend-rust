use crate::utils::{get_epoch_ts, get_random_num};

pub mod multipart;
pub mod single;

fn uniq_file_name(file_name: &str) -> String {
    let ts = get_epoch_ts();
    let random = get_random_num(101, 999);
    let (name, ext) = file_name.rsplit_once('.').unwrap_or((file_name, "unknown"));
    let name = name.split_whitespace().collect::<Vec<_>>().join("_");
    format!("{name}_{ts}_{random}.{ext}")
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_uniq_file_name() {
        let fn1 = uniq_file_name("");
        let fn2 = uniq_file_name("");
        assert!(fn1.ends_with(".unknown"));
        assert!(fn2.ends_with(".unknown"));
        assert_ne!(fn1, fn2);
        let fn1 = uniq_file_name("abcd");
        let fn2 = uniq_file_name("abcd");
        assert!(fn1.starts_with("abcd"));
        assert!(fn2.starts_with("abcd"));
        assert!(fn1.ends_with(".unknown"));
        assert!(fn2.ends_with(".unknown"));
        let fn1 = uniq_file_name("abcd.txt");
        assert!(fn1.starts_with("abcd"));
        assert!(fn1.ends_with(".txt"));
        let fn1 = uniq_file_name("abcd.txt.zip");
        let fn2 = uniq_file_name("abcd.txt.zip");
        assert!(fn1.starts_with("abcd"));
        assert!(fn1.ends_with(".zip"));
        assert!(fn2.starts_with("abcd"));
        assert!(fn2.ends_with(".zip"));
        assert_ne!(fn1, fn2);
        let fn1 = uniq_file_name("file with spaces.txt");
        assert!(fn1.starts_with("file_with_spaces"));
        assert!(fn1.ends_with(".txt"));
    }
}
