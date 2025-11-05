use procon_lg::lg_recur;

#[lg_recur]
fn a(x: i32) -> i32 {
    eprintln!("Hello from a");
    if x <= 1 {
        1
    } else {
        b(x + 2)
    }
}

#[lg_recur]
fn b(x: i32) -> i32 {
    eprintln!("Hello from b");
    a(x - 3) + 4
}

fn main() {
    println!("{}", b(5));
}
