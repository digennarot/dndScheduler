use bcrypt::{hash, DEFAULT_COST};

fn main() {
    let password = "Admin123!Secure";
    let hashed = hash(password, DEFAULT_COST).expect("Failed to hash password");
    println!("{}", hashed);
}
