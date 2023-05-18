use blake3::Hasher;
use base64::Engine;
use tracing::trace;

/// Hash `s` with blake3, Base64 encode (URL-safe with no padding) the hash, and truncate to ENCODED_LENGTH characters. 
pub fn encode(s: impl AsRef<str>, length: Option<usize>) -> String {
    let mut hasher = Hasher::new();
    hasher.update(s.as_ref().as_bytes());
    let hash = hasher.finalize();
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let length = length.unwrap_or(8);
    let bytes_len = 3 * (length/4); // length of the truncated length which yields a base64
    trace!("length: {}, bytes_len: {}", length, bytes_len);
    let mut out_buf = engine.encode(hash.as_bytes());
    out_buf.truncate(length);
    out_buf
}



#[cfg(test)]
mod tests {
    use test::{Bencher, black_box};
    use super::*;

    #[bench]
    fn bench_encode(b: &mut Bencher) {
        // Optionally include some setup
        let urls = vec![ "https://docs.rs/axum/latest/axum/index.html#required-dependencies", "https://github.com/mcaveniathor/youarel/stargazers", "https://localhost:3000/", "https://windows11.zip"];

        b.iter(|| {
            // Inner closure, the actual test
            let length = 10;
            for url in &urls {
                black_box(encode(&url, Some(length)));
            }
        });
    }
}

