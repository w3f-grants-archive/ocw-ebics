![Unit tests](https://github.com/element36-io/ocw-ebics/actions/workflows/unit-tests.yml/badge.svg)
# Ebics Off-chain Worker Substrate pallet

The repository contains a substrate solo chain that is connected to 
an arbitrary bank account which supports
the EBICS banking interface [(ISO20022)](https://www.iso20022.org/).
It synchronizes balances and transaction of a bank account with the solo chain
using an off-chain-worker and zero-knowledge proofs. This worker also contains extrinsincs to
trigger wire-transfers on the connected bank account.

Zero-Knowledge proofs are used to enable trustless atomic swap between 
FIAT and any ledger technology. The system is called Hyperfridge - the whitepaper
is [here](docs/hyperfridge-draft.pdf).

### Getting Started

Our runtime includes a pallet called `fiat-ramps` that is responsible for synchronization of bank account state with the on-chain state

`Fiat-ramps` is located inside `/pallets` folder. It is an offchain-worker pallet that primarily does two activities:

- Poll EBICS service to get the latest bank statements, puts them in the queue
- Polls EBICS service for the ZK receipt of the queued statements until it is available
- Reads the receipt and verifies it
- Process *burn requests* registered in the local pallet storage and send `unpeg` request to EBICS API.

*Burn request* is a single request to *burn*, *transfer* funds from EBICS supporting bank account using *extrinsics*. Account submits a *request* to a chain and it is registered in the local storage. Offchain worker picks up the burn request and sends it to the EBICS service. If everything goes well, EBICS service confirms the transaction and includes it in the statement, thus *finalizing* the burn request. This is done because transactions in traditional banks are not instant and sometimes it takes days to finalize them.

### Rust Setup

First, complete the [basic Rust setup instructions](./docs/rust-setup.md).

### Build

Use the following command to build the node without launching it:

```sh
cargo build --release
```

or use Makefile:

```sh
make build
```

Note: the above code might take long to compile depending on your machine specs

## Run

This chain needs EBICS Java service to properly work. This service is responsible for connecting to the bank account and providing an API for our offchain worker to interact with. You can find instructions for running the service [here](https://github.com/element36-io/ebics-java-service/blob/hyperfridge/docs/TEST.md#run-and-test-with-docker):

Or manually, make sure you cloned [`ebics-java-service`](https://github.com/element36-io/ebics-java-service) and switch to `hyperfridge` branch:

```
docker compose pull
docker compose up -d
# optional
docker compose logs -f
```

Now start the OCW. The provided `cargo run` command will launch a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

```sh
cargo run --release --dev --tmp

# or, simply:
make launch-chain
```

Or you can run the node with consistent state without `--tmp` flag.

## Tests

To run unit tests for offchain-worker, execute the following command:

```sh
cargo test -p fiat-ramps

# or, simply:
make run-tests
```

## Linting

To run `clippy` linter, execute the following command:

```sh
cargo clippy
```

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/node-template --dev
```

Purge the development chain's state:

```bash
./target/release/node-template purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/node-template -lruntime=debug --dev
```

### Test bank accounts

Following test accounts are for Alice, Bob and Charlie, respectively.

```json
{
  "accounts" : [ { 
    "ownerName" : "Alice",
    "iban" : "CH2108307000289537320",
    "accountId": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "bic" : "HYPLCH22570",
    "offeredAccountId" : "accountname1",
    "nexusBankAccountId" : "CH2108307000289537320"
  }, {
    "ownerName" : "Bob",
    "iban" : "CH1230116000289537312",
    "accountId": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    "bic" : "HYPLCH22572",
    "offeredAccountId" : "accountname2",
    "nexusBankAccountId" : "CH1230116000289537312"
  }, {
    "ownerName" : "Charlie",
    "iban" : "CH2108307000289537313",
    "accountId": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
    "bic" : "HYPLCH22573",
    "offeredAccountId" : "accountname3",
    "nexusBankAccountId" : "CH2108307000289537313"
  } ]
}
```

### Connect with Polkadot-JS Apps Front-end

Once the node template is running locally, you can connect it with **Polkadot-JS Apps** front-end
to interact with your chain. [Click here](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) connecting the Apps to your local node template.

Now you will be able to open the block explorer and see transactions, events that have occured in the blockchain.

For example, you can take a look at chain storage of `fiat-ramps` [here](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/chainstate)

### Inject Signer Account

For the offchain worker to sign transactions, we need to insert authority keys into the chain. This can be done using RPC call `author_insert_key` which is located in PolkadotJs Apps interface RPC calls section.

For testing puroposes, we can use this account:

```js
key_type: ramp
suri: cup swing hill dinner pioneer mom stick steel sad raven oak practice
public_key: 5C555czPfaHgYhKhsRg2KNCLGCJ82jVsvweTHAnfvT83uy5T
```

### New API url

You can set the new url for the `ebics-service` via PolkadotJS interface. Follow this link to the `Sudo` [tab](https://polkadot.js.org/apps/#/sudo) and choose `FiatRamps.setApiUrl` extrinsic. Paste the new url for the API and click `Submit transaction`. If everything is good, i.e you are the Sudo account and you have the necessary rights, you should see the transaction included in the block and offchain worker starts querying the new API.

### Run with Docker


First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

Run with image from DockerHub:

```bash
docker run -it -p 9944:9944 e36io/ebics-ocw:hyperfridge --dev --tmp --unsafe-rpc-external --rpc-cors=all --rpc-methods=unsafe -loffchain-worker
```

#### Docker (Linux)

Build:
```
docker build -t ebics-ocw .
```

Run:
```
docker run -it -p 9944:9944 ebics-ocw:latest --dev --tmp --unsafe-rpc-external --rpc-cors=all --rpc-methods=unsafe -loffchain-worker
```

#### Docker (MacOS)

Build:
```
docker build --platform linux/x86_64 -t ebics-ocw .
```

Run:
```
docker run --platform=linux/x86_64 -it -p 9944:9944 ebics-ocw:latest --dev --tmp --unsafe-rpc-external --rpc-cors=all --rpc-methods=unsafe -loffchain-worker
```
