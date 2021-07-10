#!/bin/bash

# download local terra testnet (start using 'docker-compose up' in 'localterra')
# see https://docs.terra.money/contracts/tutorials/setup.html#install-terra-core-locally
if [ ! -d "localterra" ]; then
    git clone https://github.com/terra-project/localterra
fi

# download contract template
# see also https://github.com/CosmWasm/cosmwasm-plus
if [ ! -d "contract-template" ]; then
    cargo generate --git https://github.com/CosmWasm/cosmwasm-template.git --branch 0.10 --name contract-template
fi

# install terra core
if [ ! -d "core" ]; then
    git clone https://github.com/terra-project/core && cd core && git checkout feature/mempool-oracle-priority && make install
fi

# configure terracli for tequila-0004
terracli config chain-id tequila-0004
terracli config node http://15.164.0.235:26657

# configure terracli for columbus-4
# terracli config chain-id columbus-4
# terracli config node http://public-node.terra.dev:26657
