Our aim is to provide a zero-touch approach to managing the devices' capabilities.

Single system
=============

A single system could be thought of as a company's internal network, where all devices are managed by the same
administrator, and a single root CA is issued that shall be trusted by everyone.

Nodes setup
-----------
Each device and router is pre-configured with:

- the root CA it trusts
- their own keypair
- a system-unique name (_ID or mnemonic_)
- a shared storage with read-only access
  > its synchronisation impact shall be minimised

Policy definition
-----------------

1. Developers of client applications define their outgoing and incoming requirements
   > In future this could be done with static analysis of the application's code
2. The administrator collects the requirements of applications
   > The administrator knows the topology of the system and the applications that will run on each device
3. The administrator **minimally defines** the capabilities for each device to be deployed
   > The administrator can customise the capabilities for each device, reducing the application(s) functionality
4. Each device capability is defined as a set of other device's permissions to interact with it
   > They are based on CA, source name, source public key, action, path
   > - the path allows the same syntax as Zenoh Key Expressions

Policy distribution
-------------------

1. The administrator encrypts the capabilities for each device with the device's public key
2. ~~The administrator signs the (cipher/clear-text) capabilities with the root CA~~
3. The administrator hashes the cleartext capabilities and signs the hash with the root CA
4. The administrator puts in the shared storage the mapping `device_id -> (hash,signature)`
5. The administrator gives the routers the mapping `device_id -> (hash,capabilities_ciphertext)`
   > the `hash` is only needed if the request made by devices includes it, but it is redundant
6. The routers propagate the ciphertexts mappings to each other
7. a. **Pull-version:**
    1. each device reads from the shared storage the mapping `device_id -> (hash,signature)`
    2. the device verifies the `hash`'s `signature` with the root `CA` - it must pass, otherwise
       `[(bad ?)end]: storage was compromised`
    3. if the hash matches the one it knows, the device uses the capabilities it has `[good end]`
    4. if the hash doesn't match, the device asks the routers for the capabilities ciphertext matching the new hash
    5. the nearest router provides the matching ciphertext
       > should be the latest capabilities ciphertext known to them for the device
    6. the device decrypts the capabilities
       > WARN: unauthenticated decryption issues?
    7. the device checks the hash matches the new one it found - it must pass, otherwise
       `[(bad ?)end]: router was compromised`
    8. the device uses the new capabilities it got `[good end]`

   b. **Push-version:**
    1. the routers push the new capabilities ciphertext to each device
    2. 7.a.i + 7.a.ii + 7.a.iii + 7.a.vi + 7.a.vii + 7.a.viii `[good end]` - or, skip 7.a.iv and 7.a.v
   > both versions should be available, as a device might be offline for a long time and upon waking up it should be
   able to update its capabilities

Events
------

General policy update
: policy change, new devices added, devices gently removed (not gone rogue/compromised)

1. The administrator redefines the capabilities for each device as in [Policy definition](#policy-definition)
    * many devices might have the same final capabilities
2. The [Policy distribution](#policy-distribution) steps are repeated
    * many devices won't need to update their capabilities, and will find out by the matching hash

Device needs to be isolated
: it or its keypair were compromised

1. the administrator issues new capabilities for the communicating devices, removing the allowed interaction
2. a Certificate Revocation List might be maintained in the shared storage for the devices that are not allowed to
   communicate anymore

Device re-keying
: device certificate is expiring soon

1. the device generates a new keypair and sends the public key to the administrator, signed with the old key and
   encrypted with the root CA public key
    * the device must be able to sign with the old key
2. the administrator issues new capabilities for the communicating devices, **changing** the new key for the re-keying
   device
    * when the other devices will receive the new capabilities, they will start ignoring the re-keying device until it
      gets its own new capabilities, as it will be still using the old key
    * we are changing the key, not adding a new one and removing the old one in a second step, to reduce both the
      network and devices load
3. the re-keying device receives the new capabilities (probably the same), encrypted with the new public key
    * now it can assume all the other devices know about the new key
4. the re-keying device starts using the new key and drops the old one

Router compromise
: a router is found to be malicious or is compromised

1. it cannot issue any different capabilities than the ones it has received from the administrator
   > if unauthenticated ciphertext decryption creates issues, they arise here
2. it cannot provide old capabilities to the devices, as they verify the signed hash on the shared storage
3. it is isolated as any other device by the administrator

Two systems collaboration
=========================

When two systems need to collaborate, they basically need to trust each other's root CA, and update the single devices'
capabilities with the _other_ system's devices' permissions.
This is achieved by the administrator of each system defining the capabilities for the other system's devices, defining
them as rooted on the _other_ system's root CA.

Thus, only the [Policy definition](#policy-definition) step 4 partially changes, as the _other_ system's capabilities on
_our_ system's devices are defined with the _other_ system's root CA in the tuple.
Duplicated names between the systems are supported, as the root CA is different.

Example: two companies, A and B, collaborate.

Sub-system segmentation
=======================

When a system is too big to be managed by a single administrator, it can be split into sub-systems, each with its own
CA that is signed by the root/_super_ system's CA, like in any CA hierarchy.

The capabilities are defined as in the [Policy definition](#policy-definition) section, with the sub-system's root CA in
the tuple.
Also when two sub-systems collaborate, as in the [Two systems collaboration](#two-systems-collaboration) section, the
capabilities are defined with the other sub-system's root CA in the tuple.

A device deployed under a sub-system, will know the very-root CA to trust, and the path-like name of the sub-system it
belongs to, so:

- a device can trust the sub-system's CA when signed by a chain of CAs up to the very-root CA
- a sub-system's CA can be rotated by its super-system's CA, and the devices will identify the new CA as the same one to
  trust as before
- the shared storage accessed by each device will contain only a single sub-system's hashes, and the routers will
  propagate only the sub-system's capabilities ciphertexts
  > we achieve minimisation of the shared storage synchronisation impact, and the routers' bandwidth usage

Example:
- Europe
  - Italia
    - Veneto
      - Venezia (Città Metropolitana)
        - Venezia (Comune)
          - Marghera
          - Mestre
          - Venezia (Centro Storico)
      - Verona (Provincia)
    - Lombardia
  - France
    - Île-de-France
      - Paris
        - 1st arrondissement
        - ...
        - 18th arrondissement
      - Versailles
    - Provence-Alpes-Côte d'Azur
    - Occitanie

// TODO - how to handle the case when a sub-system _root_ is compromised
(sub-CA revocation list)

// TODO - more on Certificate Revocation List: single system, two systems, sub-systems
