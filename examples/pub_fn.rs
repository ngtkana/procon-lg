use procon_lg::lg_recur;

mod test_module {
    use super::*;

    #[lg_recur]
    pub fn public_factorial(n: u32) -> u32 {
        if n <= 1 {
            1
        } else {
            n * public_factorial(n - 1)
        }
    }
}

fn main() {
    println!("{}", test_module::public_factorial(5));
}
