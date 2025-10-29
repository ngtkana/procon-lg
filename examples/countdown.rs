use lg_recur::lg_recur;

#[lg_recur]
fn countdown(count: u32) {
    if count == 0 {
        println!("Bang!");
    } else {
        println!("count = {count} (println)");
        eprintln!("count = {count} (debug)");
        print!("count = {count} (print)\n");
        eprint!("count = {count} (eprint)\n");
        dbg!(&count);
        countdown(count - 1);
    }
}

fn main() {
    countdown(3);
}
