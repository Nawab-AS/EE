use std::io;
use rand::Rng;
use std::cmp::Ordering;


fn main() {
    print!("\nGuess the number!");

    let mut secret_number: u16 = 0;
    loop {
        println!("\nWhat should the maximum range be?");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        
        secret_number = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                if input.trim().to_lowercase() == "quit" {
                    println!("Thank you for playing");
                    break;
                }
                println!("Please type a number!");
                continue;
            }
        };

        if secret_number >= 2 {
            break;
        }
    }

    let secret_number = rand::thread_rng().gen_range(1..=secret_number);

    loop {
        println!("\nPlease input your guess");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u16 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                if guess.trim().to_lowercase() == "quit" {
                    println!("Thank you for playing");
                    break;
                }
                println!("Please type a number!");
                continue;
            }
        };

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        };
    }
}