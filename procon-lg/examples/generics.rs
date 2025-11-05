use procon_lg::lg_recur;

#[lg_recur(show_return)]
fn generic_gcd<T>(#[show] a: T, #[show] b: T) -> T
where
    T: std::cmp::PartialEq + std::ops::Rem<Output = T> + Copy + Default + std::fmt::Debug,
{
    if b == T::default() {
        a
    } else {
        generic_gcd(b, a % b)
    }
}

fn main() {
    println!("{}", generic_gcd(48u32, 18u32));
    println!("{}", generic_gcd(48i32, 18i32));
}
