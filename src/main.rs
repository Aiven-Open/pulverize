// Copyright (c) 2026 Aiven, Helsinki, Finland. https://aiven.io/
use argh::FromArgs;
use fastcdc::v2020::{Normalization, StreamCDC};
use miniserde::{Serialize, json};
use std::fs::File;
use std::io::Read;
use xxhash_rust::xxh3::{Xxh3, xxh3_128};

const CHUNK_AVERAGE_SIZE: u64 = 4 * 1024 * 1024;
const CHUNK_MIN_SIZE: u64 = CHUNK_AVERAGE_SIZE / 4;
const CHUNK_MAX_SIZE: u64 = CHUNK_AVERAGE_SIZE * 4;

#[derive(FromArgs)]
/// Break a large file in smaller chunks.
struct Arguments {
    /// path to input file
    #[argh(option)]
    input: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct Chunk {
    digest: String,
    length: u64,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct Generator {
    name: String,
    version: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct Manifest {
    /// Version of the manifest
    version: u64,
    /// A reader of this manifest must know at least this manifest version
    min_version: u64,
    /// Metadata about the software that generated this manifest
    generator: Generator,
    /// Algorithm used to compute the hash of all chunks hashes
    root_digest_algorithm: String,
    // Hash of all chunk hashes
    root_digest: String,
    /// Algorithm used to compute the hash of each chunk
    chunk_digest_algorithm: String,
    /// List of all chunks, in the order they are in the complete file
    chunks: Vec<Chunk>,
}

fn chunk_readable<R: Read>(readable: R) -> Manifest {
    let mut root_hasher = Xxh3::new();
    let chunks = StreamCDC::with_level(
        readable,
        CHUNK_MIN_SIZE as usize,
        CHUNK_AVERAGE_SIZE as usize,
        CHUNK_MAX_SIZE as usize,
        Normalization::Level3,
    )
    .map(|maybe_chunk_data| {
        let chunk_data = maybe_chunk_data.expect("failed to produce chunk");
        let digest = format!("{:032x}", xxh3_128(chunk_data.data.as_ref()));
        root_hasher.update(digest.as_bytes());
        Chunk {
            digest,
            length: chunk_data.length as u64,
        }
    })
    .collect::<Vec<_>>();
    build_manifest(format!("{:032x}", root_hasher.digest128()), chunks)
}

fn build_manifest(root_digest: String, chunks: Vec<Chunk>) -> Manifest {
    Manifest {
        version: 1,
        min_version: 1,
        generator: Generator {
            name: env!("CARGO_PKG_NAME").into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
        root_digest_algorithm: String::from("xxh3_128"),
        chunk_digest_algorithm: String::from("xxh3_128"),
        root_digest,
        chunks,
    }
}

fn main() {
    let args: Arguments = argh::from_env();
    let file = File::open(args.input).expect("cannot open file");
    let manifest = chunk_readable(file);
    println!("{}", json::to_string(&manifest));
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngReader;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use std::io::repeat;

    #[test]
    fn test_chunk_simple() {
        let expected_manifest = build_manifest(
            String::from("89ad58f8df56ad0ca5386f7b62011523"),
            vec![
                Chunk {
                    digest: String::from("bb739cb0762fbd3d9bfa83c732225bc7"),
                    length: 1900635,
                },
                Chunk {
                    digest: String::from("a858ef6b0de52ac7dc0133abca426db3"),
                    length: 4610295,
                },
                Chunk {
                    digest: String::from("4d460a03af338183521d229da87463ea"),
                    length: 4353895,
                },
                Chunk {
                    digest: String::from("ebdf287fddc28efa4c666144ed378938"),
                    length: 1106442,
                },
                Chunk {
                    digest: String::from("e24bde00de95f3a9cdbe50853d4fc93d"),
                    length: 4508672,
                },
                Chunk {
                    digest: String::from("31408ee0cdf7a524225f10eb7faf9c12"),
                    length: 297277,
                },
            ],
        );
        // It's random content but with a fixed seed to make sure the test is repeatable
        let content = RngReader(SmallRng::seed_from_u64(0u64)).take(16 * 1024 * 1024);
        let manifest = chunk_readable(content);
        assert_eq!(manifest, expected_manifest);
    }

    #[test]
    fn test_chunk_empty() {
        let expected_manifest =
            build_manifest(String::from("99aa06d3014798d86001c324468d497f"), vec![]);
        let manifest = chunk_readable("".as_bytes());
        assert_eq!(manifest, expected_manifest);
    }

    #[test]
    fn test_chunk_max_size() {
        let expected_manifest = build_manifest(
            String::from("fa77e84a755108e704c6818a6db57734"),
            vec![
                Chunk {
                    digest: String::from("df96ebbb325830a31baeeaef3241a5a2"),
                    length: CHUNK_MAX_SIZE,
                },
                Chunk {
                    digest: String::from("30b4907c06653f2c8d09b82b03f6b7f2"),
                    length: 1,
                },
            ],
        );
        // An infinite string of 0b10101010 never causes CDC to find a natural split.
        // But since chunks have a max length, we still can't have an arbitrarily large chunk.
        let content = repeat(0b10101010).take(CHUNK_MAX_SIZE + 1);
        let manifest = chunk_readable(content);
        assert_eq!(manifest, expected_manifest);
    }

    #[test]
    fn test_digest_length() {
        // We can have a bug where we don't properly serialize the leading 0 of the hash.
        // There is ~1/16 chances for the first digit of the hash to be 0.
        // So by testing all combinations 256 chars * 10 length, we have a very high probability of catching that bug.
        for char in 0u8..=255u8 {
            for count in 1..=10 {
                let content = repeat(char).take(count);
                let manifest = chunk_readable(content);
                let first_chunk_digest = &manifest.chunks[0].digest;
                let digest_length = first_chunk_digest.len();
                assert!(
                    digest_length == 32,
                    "Failed with byte 0x{char:x} repeated {count} times: {first_chunk_digest} has length {digest_length}"
                );
            }
        }
    }
}
