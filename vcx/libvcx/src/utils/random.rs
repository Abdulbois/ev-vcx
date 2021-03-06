use rand::Rng;

pub fn random_string (size: usize) -> String {
    rand::thread_rng().gen_ascii_chars().take(size).collect::<String>()
}

pub fn random_number() -> u32 {
    rand::thread_rng().gen::<u32>()
}