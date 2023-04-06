mod random;

use random::UniformRng;

fn main() {
    let mut rng = UniformRng::from_seed(0);
    for _ in 0..100 {
        println!("{}", rng.gen_range(..=20));
    }
}
