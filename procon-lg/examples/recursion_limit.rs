use procon_lg::lg_recur;

#[lg_recur(recursion_limit = 5)]
fn countdown_limited(#[show] n: u32) {
    if n > 0 {
        countdown_limited(n - 1);
    }
}

fn main() {
    countdown_limited(3);
}
