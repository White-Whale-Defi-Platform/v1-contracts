from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgExecuteContract

client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004")
mnemonic = "main jar girl opinion train type cycle blood marble kitchen april champion amount engine crumble tunnel model vicious system student hood fee curious traffic"
deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "1000000uusd")


def send_msg(msg):
    tx = deployer.create_and_sign_tx(
        msgs=[msg], fee=std_fee
    )
    return client.tx.broadcast(tx)

def execute_contract(contract_addr: str, execute_msg):
    msg = MsgExecuteContract(
        sender=deployer.key.acc_address, 
        contract=contract_addr, 
        execute_msg=execute_msg
    )
    return send_msg(msg)

contract_address = "terra1kdphzxjcfcej2a98gz4fhwx26c3qe7calwcvj4"
result = execute_contract(contract_addr=contract_address, execute_msg={
    "above_peg": {
        "amount": {
            "denom": "uusd",
            "amount": "50000"
        },
        "luna_price": {
            "denom": "uusd",
            "amount": "6836720"
        }
    }
})

print("ExecuteMsg response")
print(result)
