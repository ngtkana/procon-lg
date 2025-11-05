use procon_lg::lg_recur;

#[lg_recur(show_return)]
fn fibonacci_early_return(#[show] n: u32) -> u32 {
    if n <= 1 {
        return 1; // early return
    }
    fibonacci_early_return(n - 1) + fibonacci_early_return(n - 2)
}

#[lg_recur(show_return)]
fn binary_search(
    #[show] arr: &[i32],
    #[show] target: i32,
    #[show] left: usize,
    #[show] right: usize,
) -> Option<usize> {
    if left > right {
        return None; // early return - not found
    }

    let mid = left + (right - left) / 2;

    #[allow(clippy::comparison_chain)]
    if arr[mid] == target {
        return Some(mid); // early return - found
    } else if arr[mid] < target {
        binary_search(arr, target, mid + 1, right)
    } else {
        binary_search(arr, target, left, mid - 1)
    }
}

#[lg_recur]
fn early_return_unit(#[show] condition: bool) {
    if condition {
        println!("Early exit");
        return; // early return with unit type
    }
    println!("Normal execution");
}

fn main() {
    println!("=== Early return with fibonacci ===");
    let result = fibonacci_early_return(4);
    println!("fibonacci(4) = {result}");

    println!("\n=== Early return with binary search ===");
    let arr = [1, 3, 5, 7, 9, 11, 13];
    let result1 = binary_search(&arr, 7, 0, arr.len() - 1);
    println!("Search for 7: {result1:?}");

    let result2 = binary_search(&arr, 8, 0, arr.len() - 1);
    println!("Search for 8: {result2:?}");

    println!("\n=== Early return with unit type ===");
    early_return_unit(true);
    early_return_unit(false);
}
