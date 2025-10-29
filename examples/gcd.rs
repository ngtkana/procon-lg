use lg_recur::lg_recur;

#[lg_recur]
fn gcd(mut x: u32, mut y: u32) -> u32 {
    if x < y {
        std::mem::swap(&mut x, &mut y);
    }

    if y == 0 { x } else { gcd(y, x % y) }
}

fn main() {
    println!("{}", gcd(15, 42));
}
