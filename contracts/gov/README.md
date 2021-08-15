# White Whale Governance Contracts

The Gov contract contains logic for creating and managing polls and allows the White Whale Protocol to be governed by its users in a decentralized manner. After the initial bootstrapping of White Whale contracts, the Gov contract is assigned to eb the owner of itself and other contracts.

New proposals for change are submitted as polls, and are voted on by WHALE stakers through the voting procedure. Polls can contain messages that can be executed directly without changing the White Whale Protocol code.

The Gov Contract keeps a balance of WHALE tokens, which it uses to reward stakers with funds it receives from user deposits from creating new governance polls. This balance is separate from the Community Pool, which is held by the Community contract (owned by the Gov contract).

## Aims and References

This repository is based both on the examples set out by the [](), []() and their respective Governance contracts.

By starting from this common ground the hope is that with time, the Anchor, Mirror, White Whale and other communities can work together to establish a common standard for Governance contracts. 

A small attempt at this is made below based on what we have found when build our governance contract.

## Specification of a Governance Contract

Governance contracts across cosmwasm and perhaps even outside follow a typical pattern that is to say they loosely define an interface of what a governance contract does. Noted here is a abstraction of the minimum viable set of functionalities needed for a governance contract.
 
# CosmWasm Starter Pack

This is a template to build smart contracts in Rust to run inside a
[Cosmos SDK](https://github.com/cosmos/cosmos-sdk) module on all chains that enable it.
To understand the framework better, please read the overview in the
[cosmwasm repo](https://github.com/CosmWasm/cosmwasm/blob/master/README.md),
and dig into the [cosmwasm docs](https://www.cosmwasm.com).
This assumes you understand the theory and just want to get coding.

## Creating a new repo from template

Assuming you have a recent version of rust and cargo (v1.51.0+) installed
(via [rustup](https://rustup.rs/)),
then the following should get you a new repo to start a contract:

First, install
[cargo-generate](https://github.com/ashleygwilliams/cargo-generate).
Unless you did that before, run this line now:

```sh
cargo install cargo-generate --features vendored-openssl
```

Now, use it to create your new contract.
Go to the folder in which you want to place it and run:


**Latest: 0.16**

```sh
cargo generate --git https://github.com/CosmWasm/cosmwasm-template.git --name PROJECT_NAME
````

**Older Version**

Pass version as branch flag:

```sh
cargo generate --git https://github.com/CosmWasm/cosmwasm-template.git --branch <version> --name PROJECT_NAME
````

Example:

```sh
cargo generate --git https://github.com/CosmWasm/cosmwasm-template.git --branch 0.14 --name PROJECT_NAME
```

You will now have a new folder called `PROJECT_NAME` (I hope you changed that to something else)
containing a simple working contract and build system that you can customize.

## Create a Repo

After generating, you have a initialized local git repo, but no commits, and no remote.
Go to a server (eg. github) and create a new upstream repo (called `YOUR-GIT-URL` below).
Then run the following:

```sh
# this is needed to create a valid Cargo.lock file (see below)
cargo check
git branch -M main
git add .
git commit -m 'Initial Commit'
git remote add origin YOUR-GIT-URL
git push -u origin master
```

## CI Support

We have template configurations for both [GitHub Actions](.github/workflows/Basic.yml)
and [Circle CI](.circleci/config.yml) in the generated project, so you can
get up and running with CI right away.

One note is that the CI runs all `cargo` commands
with `--locked` to ensure it uses the exact same versions as you have locally. This also means
you must have an up-to-date `Cargo.lock` file, which is not auto-generated.
The first time you set up the project (or after adding any dep), you should ensure the
`Cargo.lock` file is updated, so the CI will test properly. This can be done simply by
running `cargo check` or `cargo unit-test`.

## Using your project

Once you have your custom repo, you should check out [Developing](./Developing.md) to explain
more on how to run tests and develop code. Or go through the
[online tutorial](https://docs.cosmwasm.com/) to get a better feel
of how to develop.

[Publishing](./Publishing.md) contains useful information on how to publish your contract
to the world, once you are ready to deploy it on a running blockchain. And
[Importing](./Importing.md) contains information about pulling in other contracts or crates
that have been published.

Please replace this README file with information about your specific project. You can keep
the `Developing.md` and `Publishing.md` files as useful referenced, but please set some
proper description in the README.
