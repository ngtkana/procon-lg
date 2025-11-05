use procon_lg::lg_recur;

#[derive(Debug)]
struct TestStruct {
    value: i32,
}

impl TestStruct {
    #[lg_recur(show_return)]
    fn new(#[show] value: i32) -> Self {
        Self { value }
    }

    #[lg_recur]
    fn method_owned(#[show] self, #[show] multiplier: i32) -> i32 {
        self.value * multiplier
    }

    #[lg_recur(show_return)]
    fn method_ref(#[show] &self, #[show] addend: i32) -> i32 {
        self.value + addend
    }

    #[lg_recur(show_return)]
    fn method_mut_ref(#[show] &mut self, #[show] new_value: i32) -> i32 {
        let old_value = self.value;
        self.value = new_value;
        old_value
    }

    #[lg_recur]
    fn recursive_countdown(#[show] &self, #[show] count: u32) {
        if count > 0 {
            println!("count = {count}, value = {}", self.value);
            self.recursive_countdown(count - 1);
        }
    }
}

#[lg_recur(show_return)]
fn test_mutability_patterns(
    #[show] owned: i32,
    #[show] immutable_ref: &i32,
    #[show] mutable_ref: &mut i32,
    raw_const_ptr: *const i32,
    raw_mut_ptr: *mut i32,
    #[show] mut mutable_binding: i32,
) -> i32 {
    *mutable_ref += 1;
    mutable_binding *= 2;
    unsafe {
        *raw_mut_ptr = *raw_const_ptr + owned + *immutable_ref + *mutable_ref + mutable_binding;
        *raw_mut_ptr
    }
}

fn main() {
    let mut ts = TestStruct::new(42);

    println!("=== Testing method calls ===");
    let result1 = ts.method_ref(10);
    println!("method_ref result: {result1}");

    let old_value = ts.method_mut_ref(100);
    println!("method_mut_ref old_value: {old_value}");

    let result2 = TestStruct::new(5).method_owned(3);
    println!("method_owned result: {result2}");

    println!("\n=== Testing recursive method ===");
    ts.recursive_countdown(3);

    println!("\n=== Testing mutability patterns ===");
    let x = 10;
    let mut y = 20;
    let mut z = 0;
    let result = test_mutability_patterns(5, &x, &mut y, &x as *const i32, &mut z as *mut i32, 7);
    println!("Final result: {result}, x: {x}, y: {y}, z: {z}");
}
