use procon_lg::lg_recur;

#[lg_recur(no_return)]
fn test_basic(#[no_debug] no_debug: NoDebug, #[no_debug] count: u32, print_me: u32) -> NoDebug {
    if count == 0 {
        println!("Bang!");
        NoDebug(42)
    } else {
        let NoDebug(x) = test_basic(no_debug, count - 1, print_me + 10);
        NoDebug(x + 1)
    }
}

fn main() {
    let NoDebug(ans) = test_basic(NoDebug(0), 3, 42);
    println!("{ans}");
}

struct NoDebug(u32);
