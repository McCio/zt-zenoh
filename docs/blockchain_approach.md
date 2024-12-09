Single CA
=========

We are currently only considering a single CA setup.

The CA is the only party allowed to sign blocks in the chain.

Each block contains the list of HMACs of capabilities (~ACL) per-node.
Each HMAC identifies the CA and is paired with an incremental number identifying the version of the node's capabilities.
The HMAC is generated over the node's public key-encrypted ciphertext.

Thus, nodes can validate both that each block is coming from the CA, and each HMAC, too.
To validate a single HMAC only the ciphertext is needed, maintaining secrecy.
Only the final node can decrypt the ciphertext and read the capabilities.

Capabilities are defined on the receiving side:
* node X can ask me this
* node X can tell me this
* ...

Routers exchange blocks & capabilities ciphertexts, peers only request their own capabilities.


Node configuration
------------------

Any node may start in bootstrap mode on first wakeup: it is a _dev mode_ to install the CA certificate and maybe get its public key for capabilities configuration.

Alternatively, the configuration is deployed before first wakeup: at least the CA certificate is needed, for routers also the 0th block.

Device beahviour
----------------

QoS defined in configuration for how to behave in "insecure" mode:
* apply last known capabilities
* isolate from network
* ...

The device wakes up and starts in bootstrap mode.
1. It searches for a router on the network to get its capabilities known.
   1. It already knows the CA public key, so it can verify the router's info.
2. The router sends the capabilities ciphertext with the HMAC.
3. The device validates the ciphertext and its source, then decrypts it and applies the capabilities.
4. Periodically, the device asks the router for an updated version, validates it, and applies the new capabilities.

If the device can talk to multiple routers, it asks both checking the version. They'll usually match, and that's a healthy sign for both routers.
If they differ, and both are valid, it will take the one with the higher version number. (maybe flag the lower number-provider somehow?)

If, asking for updates, a router sends a lower version number, the device will ignore it. (maybe flag it somehow?)

If a device sees a new version number passing by (maybe routing the message to another device), it will ask for the new version from the router it knows.
Until it gets the new version, it will apply the predefined QoS, as it **knows** the capabilities were changed, not **how** yet.

When a device wakes up after a long time it is sleeping, it will receive a new version of the capabilities (version n+m), and it's ok as-is.
A possible improvement might be a small Blockchain for its own capabilities, so it can verify the new version is a future one, with respect to the last one it knows, rather than just "a version signed by the same CA with a greater version number".


Router behaviour
----------------

Routers behave similarly to nodes, but they have a few more responsibilities.
They receive the CA-signed blocks and capabilities ciphertexts, and they distribute them to the nodes.
They exchange them with other routers, usually a waterfall behaviour happen when the CA broadcasts a new block:
* the CA broadcasts the new block is available
* the CA sends the block to the routers it knows directly
  * or, the routers ask for the block
* the CA sends the ciphertexts to the routers it knows directly
  * or, the routers ask for the ciphertexts
* in the meantime, the routers propagate the new version availability to other routers
* and then, the routers also send the block and ciphertexts to the other routers
  * or, the other routers ask for the block and ciphertexts

Each router also parses their own capabilities, like any other node.

The CA might be connected only during the broadcasting phase, or it might be connected all the time.

When a router wakes up after a long time it is sleeping, given the Blockchain properties, it will be able to verify the new block it receives is a future one, with respect to the last one it knows.

Rogue device isolation
----------------------

If a device is found to be rogue, the CA will revoke its capabilities and broadcast the new block.
How? TBD.


Rogue router isolation
----------------------

If a router is found to be rogue by a device, the device will stop asking for updates from it.
- It is recognised by the version of the capabilities' ciphertext.

If a router is found to be rogue by another router, the other router will stop asking and sending updates to it.
- It is recognised by the version of the block and/or the capabilities' ciphertext.


Subnet tree
-----------

If a subnet only relies on a single router, the router will be the only one to ask for updates.


Node re-keying
--------------

The CA might prompt a device to be re-keyed.
1. It will ask the device to generate a new key pair and send the public key back.
2. Then it will broadcast a new block with its capabilities' on the new public key.
3. The device will ask for the capabilities on the new public key to its known routers, setting the Zenoh ID to the new public key.
4. When it will receive them, it will stop using the old one and start using the new one.
5. After confirmation of reception, the CA will revoke the old capabilities and broadcast a new block.


CA re-keying
------------
TBD.
