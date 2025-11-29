use std::io;
use primitive_types::U256;
use std::time::Instant;

fn main() {
    println!("Generate the nth fibonacci number");
    let n: i32 = loop {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        
        match input.trim().parse() {
            Ok(num) => if num < 1 {println!("Number too small!")} else if num > 369 {println!("Number too large!")} else { break num },
            Err(_) => {
                println!("Please type a number!");
            }
        };
    };
    let n = n as u16;

    println!("generating the {n}th fibonacci number");
    let (nth_fibonacci, time) = fibonacci(n);
    println!("The {n}th fibonacci number is {}\nCalculation took {} ns", nth_fibonacci, time);
}

fn fibonacci(n: u16) -> (U256, u128) {
    let mut nums: [U256; 2] = [U256::one(); 2];
    let start = Instant::now();
    for _i in 0..(n-1) {
        nums = [nums[1], nums[0]+nums[1]];
    }
    let duration = start.elapsed();

    (nums[0], duration.as_nanos())
}