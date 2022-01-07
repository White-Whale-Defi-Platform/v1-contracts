from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from terra_sdk.core.coins import Coin
from white_whale.contracts.governance import *

#------------------------
#   Run with: $ cd /workspaces/devcontainer/contracts ; /usr/bin/env /bin/python3 -- /workspaces/devcontainer/contracts/scripts/ust_vault.py 
#------------------------

# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
# mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
# deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

gov = Governance(deployer)
create = True

if create:
    gov.create(quorum= 0.3,threshold= 0.6,
                voting_period= 94097,
                timelock_period= 40327,
                expiration_period= 40327*2,
                proposal_deposit= 5000000000,
                snapshot_period= 13443)
    gov.set_token()

gov.update_owner(owner=gov.get("multisig"))
deployer.whale_balance()
# gov.stake(1000)
gov.get_staked_amount()
# gov.create_poll()
# gov.unstake_all()

# gov.create_poll()
   