from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.util.contract import read_file_as_b64, get_code_id, get_contract_address
from terra_sdk.core.auth import StdFee
from terra_sdk.core.bank import MsgSend
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract

client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004")
mnemonic = "main jar girl opinion train type cycle blood marble kitchen april champion amount engine crumble tunnel model vicious system student hood fee curious traffic"
deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "10000000uusd")


def send_msg(msg):
    tx = deployer.create_and_sign_tx(
        msgs=[msg], fee=std_fee
    )
    return client.tx.broadcast(tx)

def store_contract(contract_name: str) -> str:
    bytes = read_file_as_b64(f"artifacts/{contract_name}.wasm")
    msg = MsgStoreCode(deployer.key.acc_address, bytes)
    result = send_msg(msg)
    return get_code_id(result)

def instantiate_contract(code_id: str, init_msg) -> str:
    msg = MsgInstantiateContract(
        owner=deployer.key.acc_address,
        code_id=code_id,
        init_msg=init_msg
    )
    result = send_msg(msg)
    return get_contract_address(result)

def send_to_contract(contract_addr: str):
    msg = MsgSend(
        from_address=deployer.key.acc_address,
        to_address=contract_addr,
        amount="1000000uusd"
    )
    return send_msg(msg)

def execute_contract(contract_addr: str, execute_msg):
    msg = MsgExecuteContract(
        sender=deployer.key.acc_address, 
        contract=contract_addr, 
        execute_msg=execute_msg
    )
    return send_msg(msg)


code_id = store_contract(contract_name="test_contract")
contract_address = instantiate_contract(code_id=code_id, init_msg={})
print(f'instantiated {contract_address}')
send_to_contract(contract_addr=contract_address)
print(f'sent funds')
result = execute_contract(contract_addr=contract_address, execute_msg={
    "below_peg": {
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

