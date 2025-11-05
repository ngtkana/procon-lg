use procon_lg::lg_recur;

#[lg_recur]
fn count_down(hidden_arg: HiddenType, #[show] count: u32, #[show] print_me: u32) -> HiddenType {
    if count == 0 {
        println!("Bang!");
        HiddenType(42)
    } else {
        let HiddenType(x) = count_down(hidden_arg, count - 1, print_me + 10);
        HiddenType(x + 1)
    }
}

fn main() {
    let HiddenType(ans) = count_down(HiddenType(0), 3, 42);
    println!("{ans}");
}

struct HiddenType(u32);
