import base64
import json

import pathlib
import sys
import datetime


from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from terra_sdk.core.coins import Coin
from white_whale.contracts.emissions import *

#------------------------
#   Run with: $ cd /workspaces/devcontainer/contracts ; /usr/bin/env /bin/python3 -- /workspaces/devcontainer/contracts/scripts/ust_vault.py 
#------------------------

#####################
#   DEPLOY PARAMETERS
#####################

MILLION = 1_000_000

YEAR = 2022
MONTH = 1
DAY = 12
# 0 AM UTC
HOUR = 14
START_TIME = datetime.datetime(year=YEAR, month=MONTH, day=DAY, hour=HOUR)
# START_TIME = datetime.datetime.utcnow() + datetime.timedelta(minutes=2);

# Start time in linux language
BLOCK_START_TIME = (START_TIME - datetime.datetime(1970, 1, 1)).total_seconds()

DURATION = datetime.timedelta(days=90)

END_TIME = START_TIME + DURATION

NOW = datetime.datetime.utcnow()
print(f'Emmissions start {START_TIME - NOW} from now.')

print(f'Starts on {START_TIME}')
print(f'Ends on {END_TIME}')

BLOCK_END_TIME = BLOCK_START_TIME + DURATION.total_seconds()

print(BLOCK_END_TIME - BLOCK_START_TIME)
print((END_TIME - START_TIME).total_seconds())
# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
# mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"

claimer = "terra10lm49e6ufm8cfpwcmcltvxkv3s6cqeunyjhaj5"
deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None, return_msg=False)
# deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

emissions = Emissions(deployer)
create = False
print(f'gov address: {emissions.get("governance")}')
print(f'multisig address: {emissions.get("multisig")}')
print(deployer.wallet.key.acc_address)
input("Confirm")

if create:
    # TODO: make multisig owner 
    emissions.create(start=int(BLOCK_START_TIME), duration=int(DURATION.total_seconds()))

# deployer.whale_balance()

emissions.create_vesting(amount=int(3*600_000*MILLION), start=int(BLOCK_START_TIME), duration=int(DURATION.total_seconds()), claimer=claimer)

# emissions.claim()
# emissions.reclaim(deployer.wallet.key.acc_address)