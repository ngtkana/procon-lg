use procon_lg::lg_recur;

#[lg_recur]
fn countdown(#[show] count: u32) {
    if count == 0 {
        eprintln!("Bang!");
    } else {
        eprintln!("count = {count} (eprintln)");
        countdown(count - 1);
    }
}

fn main() {
    countdown(3);
}
