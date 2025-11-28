// todo: trait

/// x,xxx.xx
pub fn fmt_n(n: i32) -> String {
    n.to_string().chars().rev().enumerate().fold(
        String::new(),
        |mut acc, (i, ch)| { if i != 0 && i % 3 == 0 { acc.push(','); } acc.push(ch); acc }
    ).chars().rev().collect()
}

/// xx,xxx,xxx
pub fn fmt_f(f: f32) -> String {
    format!("{:.2}", f)
        .split_once('.')
        .map(|(int, frac)| int.parse::<i32>()
            .map(|v| format!("{}.{}", fmt_n(v), frac))
            .unwrap_or_else(|_| format!("{:.2}", f))
        )
        .unwrap_or_else(|| format!("{:.2}", f))
}
