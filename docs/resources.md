Collections
============

* https://cryptography.rs/
  * Manually curated list of Rust cryptographic libraries
* https://lib.rs/cryptography
  * Cryptography libraries tagged with `cryptography` and related tags


Libraries
=========
* https://docs.rs/snow/latest/snow/
  * Noise Protocol implementation
  * https://noiseprotocol.org/noise.html
* https://github.com/zama-ai/tfhe-rs
  * Fully Homomorphic Encryption over the Torus
    * bool/integer arithmetic over encrypted data
  * https://tfhe.github.io/tfhe/
* https://github.com/parallaxsecond/rust-tss-esapi
  * TPM 2.0 support
* https://docs.rs/x509-parser/latest/x509_parser/
  * x509 certificate parser
* https://github.com/rustpq/pqcrypto
  * post-quantum cryptography
* https://github.com/hacl-star/hacl-star
  * formally verified
  * non-rust
* 25519
  * https://lib.rs/crates/ed25519-dalek
  * https://lib.rs/crates/curve25519-dalek
  * https://lib.rs/crates/x25519-dalek

Projects
========
* https://github.com/RustCrypto
* https://github.com/rusticata

Ristretto
---------
https://ristretto.group/

Ristretto is a technique for constructing prime order elliptic curve groups with non-malleable encodings. It extends Mike Hamburg's Decaf approach to cofactor elimination to support cofactor-\(8\) curves such as Curve25519.

In particular, this allows an existing Curve25519 library to implement a prime-order group with only a thin abstraction layer, and makes it possible for systems using Ed25519 signatures to be safely extended with zero-knowledge protocols, with no additional cryptographic assumptions and minimal code changes.

Ristretto can be used in conjunction with Edwards curves with cofactor \(4\) or \(8\), and provides the following specific parameter choices:

* `ristretto255`, built on top of Curve25519.
  * For Rust: [curve25519-dalek](https://doc.dalek.rs/curve25519_dalek/) already implements it
* [Why Ristretto?](https://ristretto.group/why_ristretto.html) describes the pitfalls of the cofactor abstraction mismatch.
* [What is Ristretto?](https://ristretto.group/what_is_ristretto.html) describes what Ristretto provides to protocol implementors.
* [Ristretto in Detail](https://ristretto.group/details/index.html) contains mathematical justification for why Ristretto works.
* [Explicit Formulas](https://ristretto.group/formulas/index.html) describes how to implement Ristretto.
* [Test Vectors](https://ristretto.group/test_vectors/ristretto255.html) contains test vectors for the Ristretto functions.
* [Ristretto Implementations](https://ristretto.group/implementations.html) contains a list of implementations of Ristretto.
