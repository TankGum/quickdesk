//! `cargo run -p qd-ai-usage --example usage` — print detected AI tools and their limits.
fn main() {
    let all = qd_ai_usage::collect(true);
    for p in &all {
        println!(
            "{} ({}) source={} plan={:?} as_of={:?} error={:?}",
            p.name, p.provider, p.source, p.plan, p.as_of, p.error
        );
        for w in &p.windows {
            println!(
                "  {:<12} {:>6} resets_at={:?} {}",
                w.id,
                w.used_percent.map(|x| format!("{x:.0}%")).unwrap_or("-".into()),
                w.resets_at,
                w.detail.as_deref().unwrap_or("")
            );
        }
    }
    println!("headline 5h: {:?}", qd_ai_usage::headline_percent(&all));
}
