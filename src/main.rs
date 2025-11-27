use std::io;

fn main() {
    println!("Generate the nth fibonacci number");
    let n: u8 = loop {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        
        match input.trim().parse() {
            Ok(num) => if num > 0 && num < 186 { break num } else {println!("Number out of range")},
            Err(_) => {
                println!("Please type a number!");
            }
        };
    };

    println!("generating the {n}th fibonacci number");
    println!("The {n}th fibonacci number is {}", fibonacci(n));
}

fn fibonacci(n: u8) -> u128{
    let mut nums: [u128; 2] = [1, 1];

    for _i in 0..(n-1) {
        nums = [nums[1], nums[0]+nums[1]];
    };

    nums[0]
}