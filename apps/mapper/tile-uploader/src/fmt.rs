pub fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        return format!("{:.1}M", n as f64 / 1_000_000.0);
    }
    if n >= 1_000 {
        return format!("{:.1}K", n as f64 / 1_000.0);
    }
    return format!("{n}");
}

pub fn format_bytes(n: u64) -> String {
    if n >= 1_000_000_000 {
        return format!("{:.2}GB", n as f64 / 1_000_000_000.0);
    }
    if n >= 1_000_000 {
        return format!("{:.1}MB", n as f64 / 1_000_000.0);
    }
    return format!("{:.1}KB", n as f64 / 1_000.0);
}

pub fn format_duration(secs: f64) -> String {
    if secs >= 3600.0 {
        return format!("{:.1}h", secs / 3600.0);
    }
    if secs >= 60.0 {
        return format!("{:.1}m", secs / 60.0);
    }
    return format!("{secs:.0}s");
}
