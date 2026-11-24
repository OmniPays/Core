use soroban_sdk::{unwrap::UnwrapOptimized, Env, EnvBase, String, Val};

const MAX_PREFIX_BYTES: usize = 100;
const MAX_CONCAT_BYTES: usize = 35;

fn utf8_prefix_len(bytes: &[u8]) -> usize {
    match core::str::from_utf8(bytes) {
        Ok(_) => bytes.len(),
        Err(error) => error.valid_up_to(),
    }
}

fn copy_prefix(e: &Env, value: &String, max: usize, out: &mut [u8]) -> usize {
    let copy_len = (value.len() as usize).min(max).min(out.len());
    e.string_copy_to_slice(value.to_object(), Val::U32_ZERO, &mut out[..copy_len])
        .unwrap_optimized();
    utf8_prefix_len(&out[..copy_len])
}

pub trait TakeFirstNCharsAndConcat {
    fn take_first_n_chars(&self, e: &Env, n: usize) -> String;
    fn concat(&self, e: &Env, other: String) -> String;
}

impl TakeFirstNCharsAndConcat for String {
    fn take_first_n_chars(&self, e: &Env, n: usize) -> String {
        let mut slice: [u8; MAX_PREFIX_BYTES] = [0; MAX_PREFIX_BYTES];
        let prefix_len = copy_prefix(e, self, n, &mut slice);
        String::from_bytes(e, &slice[..prefix_len])
    }

    fn concat(&self, e: &Env, other: String) -> String {
        let mut slice: [u8; MAX_CONCAT_BYTES] = [0; MAX_CONCAT_BYTES];
        let first_len = copy_prefix(e, self, self.len() as usize, &mut slice);
        let combined_len =
            first_len + copy_prefix(e, &other, other.len() as usize, &mut slice[first_len..]);
        String::from_bytes(e, &slice[..combined_len])
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn take_first_n_chars_ascii() {
        let e = Env::default();
        let value = String::from_str(&e, "ABCDEFGHIJ");
        assert_eq!(
            value.take_first_n_chars(&e, 6),
            String::from_str(&e, "ABCDEF")
        );
        assert_eq!(
            value.take_first_n_chars(&e, 20),
            String::from_str(&e, "ABCDEFGHIJ")
        );
    }

    #[test]
    fn take_first_n_chars_avoids_splitting_utf8() {
        let e = Env::default();
        let value = String::from_str(&e, "éééé");
        assert_eq!(value.take_first_n_chars(&e, 1), String::from_str(&e, ""));
        assert_eq!(value.take_first_n_chars(&e, 2), String::from_str(&e, "é"));
        assert_eq!(value.take_first_n_chars(&e, 6), String::from_str(&e, "ééé"));
    }

    #[test]
    fn take_first_n_chars_handles_longer_strings_than_the_buffer() {
        let e = Env::default();
        let value = String::from_str(&e, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        let result = value.take_first_n_chars(&e, 200);
        assert_eq!(result.len(), MAX_PREFIX_BYTES as u32);
    }

    #[test]
    fn concat_preserves_ascii_metadata() {
        let e = Env::default();
        let left = String::from_str(&e, "TOK0-ABCDEF");
        let right = String::from_str(&e, "-SOROSWAP-LP");
        assert_eq!(
            left.concat(&e, right),
            String::from_str(&e, "TOK0-ABCDEF-SOROSWAP-LP")
        );
    }

    #[test]
    fn concat_truncates_long_inputs_without_panicking() {
        let e = Env::default();
        let left = String::from_str(&e, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        let right = String::from_str(&e, "ééééééééééééééééééééé");
        assert!(left.concat(&e, right).len() <= MAX_CONCAT_BYTES as u32);
    }
}
