use rand::Rng;

pub fn generate_base32_string() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    let base32_string = base32::encode(
        base32::Alphabet::RFC4648 { padding: false },
        &bytes,
    );

    base32_string
}
