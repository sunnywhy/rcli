use rand::seq::SliceRandom;

const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
// no I, O, avoid confusion
const LOWER: &[u8] = b"abcdefghijkmnopqrstuvwxyz";
// no l, avoid confusion
const NUMBER: &[u8] = b"123456789";
// no 0, avoid confusion
const SYMBOL: &[u8] = b"!@#$%^&*_";

pub fn process_genpass(
    length: u8,
    upper: bool,
    lower: bool,
    number: bool,
    symbol: bool,
) -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    let mut password: Vec<u8> = Vec::with_capacity(length as usize);
    let mut chars = Vec::new();

    if upper {
        chars.extend_from_slice(UPPER);
        password.push(
            *UPPER
                .choose(&mut rng)
                .expect("UPPER won't be empty in this context"),
        );
    }
    if lower {
        chars.extend_from_slice(LOWER);
        password.push(
            *LOWER
                .choose(&mut rng)
                .expect("LOWER won't be empty in this context"),
        );
    }
    if number {
        chars.extend_from_slice(NUMBER);
        password.push(
            *NUMBER
                .choose(&mut rng)
                .expect("NUMBER won't be empty in this context"),
        );
    }
    if symbol {
        chars.extend_from_slice(SYMBOL);
        password.push(
            *SYMBOL
                .choose(&mut rng)
                .expect("SYMBOL won't be empty in this context"),
        );
    }

    for _ in 0..length - password.len() as u8 {
        let c = chars
            .choose(&mut rng)
            .expect("chars won't be empty in this context");
        password.push(*c); // *c will make a copy here since the type is u8
    }

    password.shuffle(&mut rng);

    println!("{}", String::from_utf8(password)?);

    Ok(())
}
