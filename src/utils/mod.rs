pub mod dir;

pub fn human_readable_size(bytes: u64) -> String {
    let kilobyte = 1024u64;
    let megabyte = kilobyte * 1024;
    let gigabyte = megabyte * 1024;
    let terabyte = gigabyte * 1024;

    match bytes {
        b if b >= terabyte => format!("{:.2} TB", b as f64 / terabyte as f64),
        b if b >= gigabyte => format!("{:.2} GB", b as f64 / gigabyte as f64),
        b if b >= megabyte => format!("{:.2} MB", b as f64 / megabyte as f64),
        b if b >= kilobyte => format!("{:.0} kB", b as f64 / kilobyte as f64),
        _ => format!("{} b", bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes() {
        assert_eq!(human_readable_size(500), "500 b");
    }

    #[test]
    fn test_kilobytes() {
        assert_eq!(human_readable_size(2048), "2 kB");
    }

    #[test]
    fn test_megabytes() {
        assert_eq!(human_readable_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_gigabytes() {
        assert_eq!(human_readable_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_terabytes() {
        assert_eq!(human_readable_size(1099511627776), "1.00 TB");
    }

    #[test]
    fn test_edge_case() {
        assert_eq!(human_readable_size(1023), "1023 b");
        assert_eq!(human_readable_size(1024), "1 kB");
    }
}
