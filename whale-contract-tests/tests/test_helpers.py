from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.core import Coins
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgExecuteContract

WHALE_TOKEN= "terra1al4gd6wudfalazsvrjzz4fs8srasqcn9vyvqp9"


# SEND GGY
# msg = MsgExecuteContract(
#             sender=wallet.key.acc_address,
#             contract=WHALE_TOKEN,
#             execute_msg={
#         "transfer": {
#             "recipient": "terra1f6nthhyvtjalucnzdwwajp7mnhm5tpn5l46sed",
#             "amount": "100000000",
#         }},
#         )
# tx = wallet.create_and_sign_tx(
#             msgs=[msg], fee=std_fee
#         )
# result = client.tx.broadcast(tx)

def send_gov_token(amount: "100000000", recipient="terra1f6nthhyvtjalucnzdwwajp7mnhm5tpn5l46sed"):
    msg = MsgExecuteContract(
            sender=wallet.key.acc_address,
            contract=WHALE_TOKEN,
            execute_msg={
        "transfer": {
            "recipient": recipient,
            "amount": amount,
        }},
        )
    tx = wallet.create_and_sign_tx(
                msgs=[msg], fee=std_fee
            )
    result = client.tx.broadcast(tx)