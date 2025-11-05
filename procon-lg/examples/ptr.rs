use procon_lg::lg_recur;

#[lg_recur]
fn countdown(x: *const i32) {
    unsafe {
        if *x == 0 {
            println!("Bang!");
        } else {
            let x = *x - 1;
            let x = &x;
            // countdown(x); <- compile error!
            countdown(std::ptr::from_ref(x));
        }
    }
}

fn main() {
    let x = 3;
    countdown(&raw const x);
}
