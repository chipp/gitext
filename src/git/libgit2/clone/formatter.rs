pub fn format_bytes_count(bytes: usize) -> String {
    if bytes > 1 << 30 {
        format!(
            "{}.{:.>2} GiB",
            bytes >> 30,
            (bytes & ((1 << 30) - 1)) / 10737419
        )
    } else if bytes > 1 << 20 {
        let x = bytes + 5243;
        format!(
            "{}.{:.>2} MiB",
            x >> 20,
            ((x & ((1 << 20) - 1)) * 100) >> 20
        )
    } else if bytes > 1 << 10 {
        let x = bytes + 5;
        format!(
            "{}.{:.>2} KiB",
            x >> 10,
            ((x & ((1 << 10) - 1)) * 100) >> 10
        )
    } else {
        format!("{} bytes", bytes)
    }
}

pub fn format_bytes_rate_count(bytes: usize) -> String {
    if bytes > 1 << 30 {
        format!(
            "{}.{:.>2} GiB/s",
            bytes >> 30,
            (bytes & ((1 << 30) - 1)) / 10737419
        )
    } else if bytes > 1 << 20 {
        let x = bytes + 5243;
        format!(
            "{}.{:.>2} MiB/s",
            x >> 20,
            ((x & ((1 << 20) - 1)) * 100) >> 20
        )
    } else if bytes > 1 << 10 {
        let x = bytes + 5;
        format!(
            "{}.{:.>2} KiB/s",
            x >> 10,
            ((x & ((1 << 10) - 1)) * 100) >> 10
        )
    } else {
        format!("{} bytes/s", bytes)
    }
}
