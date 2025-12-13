mod primes;

const SEED: u64 = 958745;

fn main() {
    let bits = 512;

    println!("Generating {} bit primes...", bits);

    let mut stream1 = primes::SeededStream::new(bits, SEED);
    // let mut stream2 = primes::SeededStream::new(bits, SEED);

    let mut i = 1;
    loop {
        let prime1 = stream1.next().unwrap();
        // let prime2 = stream2.next().unwrap();
        // assert_eq!(prime1, prime2);
        println!("matching prime #{}: {}", i, prime1);
        i += 1;
    }
}
