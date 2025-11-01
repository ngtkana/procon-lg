use procon_lg::lg_recur;

#[lg_recur]
fn test_multiline(n: u32) {
    if n == 0 {
        println!("Single line");
        println!("Line 1\nLine 2\nLine 3");
        println!("Another single line");
    } else {
        println!("Multi-line at level {n}:\nFirst line\nSecond line");
        test_multiline(n - 1);
    }
}

fn main() {
    test_multiline(2);
}
