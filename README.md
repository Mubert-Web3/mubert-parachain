<div align="center">

# Mubert Parachain

**An Ethereum compatible [Parachain](https://polkadot.com/parachains/) with native IP-Onchain support built with the [Polkadot-SDK](https://github.com/paritytech/polkadot-sdk).**

</div>

## Build the Mubert Parachain Node

If you're new to working with Substrate-based blockchains, see [Polkadots's Getting Started Guide](https://docs.polkadot.com/develop/#parachain-developers).

```bash
# Fetch the code
git clone https://github.com/Mubert-Web3/mubert-parachain
cd mubert-parachain

# Build the node
cargo build --release
```
## Starting a Development Chain

### Relay chain prerequisites

Download the `polkadot` (and the accompanying `polkadot-prepare-worker` and `polkadot-execute-worker`) binaries from
[Polkadot SDK releases](https://github.com/paritytech/polkadot-sdk/releases). Then expose them on `PATH` like so:

```sh
export PATH="$PATH:<path/to/binaries>"
```

### Run the Mubert Parachain Node

For setup, please consider the instructions for `zombienet` installation [here](https://paritytech.github.io/zombienet/install.html#installation)

We're left just with starting the network:

```sh
zombienet --provider native spawn zombienet.toml
```

## Connect with the Polkadot-JS Apps Front-End

- 🌐 You can interact with your local node using the
hosted version of the Polkadot/Substrate Portal:
[relay chain](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944)
and [parachain](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9988).

- 🪐 A hosted version is also
available on [IPFS](https://dotapps.io/).

- 🧑‍🔧 You can also find the source code and instructions for hosting your own instance in the
[`polkadot-js/apps`](https://github.com/polkadot-js/apps) repository.
