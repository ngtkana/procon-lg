use procon_lg::lg_recur;

#[lg_recur]
fn fib(#[show] n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        let x = fib(n - 1);
        let y = fib(n - 2);
        x + y
    }
}

fn main() {
    println!("{}", fib(4));
}
