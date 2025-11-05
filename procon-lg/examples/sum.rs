use procon_lg::lg_recur;

#[lg_recur(show_return)]
fn sum(#[show] a: &[u32]) -> u32 {
    let n = a.len();
    match n {
        0 => 0,
        1 => a[0],
        _ => {
            let (a0, a1) = a.split_at(n / 2);
            sum(a0) + sum(a1)
        }
    }
}

fn main() {
    let a = [5, 2, 6, 3, 6, 3, 9];
    println!("{}", sum(&a));
}
