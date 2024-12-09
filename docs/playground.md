Bases: always latest Zenoh 1.x release


Starter exercise
================
* Publish a random number/value, subscribe to it and print it out.
* Compute average and expose it as a queryable.

3 possible approaches:
1. using `select!`
2. using callbacks
3. using tokio spawn


Adding crypto
=============
Depending on argument, load a security feature and do the same through secure communication.
Possible features:
* MAC
  : _Message authentication code_
  : Authentication + integrity
  : HMAC? maybe unuseful

* ECDSA
  : _Elliptic Curve Digital Signature Algorithm_
  : NIST-produced standard for digital signatures
  : Depends on random generator for signatures - even if it might be worked around with an EdDSA-like approach

* EdDSA
  : _Edwards-curve Digital Signature Algorithm_
  : Ed25519 (EdDSA over Curve25519)
  : No random generator needed for signatures (pro)
  : NIST-approved standard

* ECDH
  : _Elliptic-curve Diffie–Hellman_
  : Establish symmetric-key cipher
  : Curve25519 commonly used


