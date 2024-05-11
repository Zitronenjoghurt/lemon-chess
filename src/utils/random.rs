use rand::{distributions::Uniform, Rng};

pub fn generate_user_friendly_code(length: u32) -> String {
    let chars = "ABCDEFGHKLMNPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, chars.len());
    (0..length)
        .map(|_| chars.as_bytes()[rng.sample(range)] as char)
        .collect()
}
