<h1 style="text-align: center;"><a href="https://casperlabs.io/"><img src="https://pbs.twimg.com/profile_images/1627476525997228032/rciFKQnM_400x400.jpg" width="120" style="position: relative; top:5px" alt="Casper"></a> <b>dePoll -- A Decentralized Voting Contract</b>

[![Build Status](https://app.travis-ci.com/cryofracture/grocery-inventory.svg?branch=main)](https://app.travis-ci.com/cryofracture/grocery-inventory)  [![Build Status](https://app.travis-ci.com/cryofracture/grocery-inventory.svg?branch=feature)](https://app.travis-ci.com/cryofracture/grocery-inventory)

# **[CasperLabs](https://casperlabs.io/)** Blockchain-based voting contract

# 📔 **Project**

#### This projects' purpose is to serve as a baseline proof-of-concept of a decentralized voting platform on the <a href="https://casperlabs.io/"><img  style="position: relative; top:3px" alt="Casper" src="https://pbs.twimg.com/profile_images/1627476525997228032/rciFKQnM_400x400.jpg" height="20" width="20" alt="Casper"/> Casper Blockchain</a> with pre-composed smart contract and a small exampls of this use case in action, including queries done with the [Casper Client CLI](https://github.com/casper-ecosystem/casper-client-rs).

🏭 **Tech Stack**

- **Blockchain ⛓️**

  <img alt="Casper" width="100" src="https://pbs.twimg.com/profile_images/1627476525997228032/rciFKQnM_400x400.jpg"/>

- **Backend**

  <img alt="Rust" src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white"/>
  <img alt="WEBASSEMBLY" src="https://camo.githubusercontent.com/9734598c5ee062706c931512b7572b5675274ee5a728b9afddfe0d4cdd1ba82d/68747470733a2f2f696d672e736869656c64732e696f2f7374617469632f76313f7374796c653d666f722d7468652d6261646765266d6573736167653d576562417373656d626c7926636f6c6f723d363534464630266c6f676f3d576562417373656d626c79266c6f676f436f6c6f723d464646464646266c6162656c3d"/>

[//]: # (## Contract deployment walkthrough)

[//]: # (Go [here]&#40;./guide/README.md&#41; for a walkthrough on how to deploy the inventory management system and interact with the NCTL network to browse items.)

## Smart contracts

Smart contracts are implemented in [Rust](https://www.rust-lang.org/) using the [Casper smart contract crate](https://docs.rs/casper-contract/latest/casper_contract/).

# 🛣️ Roadmap / Todo / Tofix
- MVP
    - [✓] Develop contract 🏬
    - [✓] Develop contract function that initializes new Poll 🚚
    - [✓] Develop contract function that adds new Poll option 🚚
    - [✓] Develop contract entrypoint that extends poll duration 🚚
- Back-end
    - [✓] Fine-tune contract functions to optimize/lower cost of contract calls.
- Housekeeping
    - [] Document contract code (In Progress)
    - [] Build out unit tests (In Progress)
    - [✓] Simplify code

### ❓Have questions?

Go to the `#dev-discussion` channel [on Discord](https://discord.gg/casperblockchain)

### 🪦 Errors ?

If you see any typos or errors you can edit the code directly on GitHub and raise a Pull Request on the `feature` branch, thank you in advance!