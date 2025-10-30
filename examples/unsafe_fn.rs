use procon_lg::lg_recur;

#[lg_recur]
unsafe fn unsafe_sum(#[no_debug] ptr: *const i32, len: usize) -> i32 {
    if len == 0 {
        0
    } else {
        *ptr + unsafe_sum(ptr.add(1), len - 1)
    }
}

fn main() {
    let data = [1, 2, 3, 4, 5];
    let result = unsafe { unsafe_sum(data.as_ptr(), data.len()) };
    println!("{result}");
}
