use procon_lg::lg_recur;

#[lg_recur]
fn test_multiline(#[show] n: u32) {
    if n == 0 {
        eprintln!("Single line");
        eprintln!("Line 1\nLine 2\nLine 3");
        eprintln!("Another single line");
    } else {
        eprintln!("Multi-line at level {n}:\nFirst line\nSecond line");
        test_multiline(n - 1);
    }
}

fn main() {
    test_multiline(2);
}
