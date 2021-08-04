from typing import Union
from attr import dataclass

from query import Token, NativeToken

@dataclass
class PoolConfig:
    contract_address: str
    token: Union[NativeToken, Token]

TERRASWAP_BLUNA_BOND_CONFIG=PoolConfig(contract_address='terra1fflas6wv4snv8lsda9knvq2w0cyt493r8puh2e', token=Token('terra1u0t35drzyy0mujj8rkdyzhe264uls4ug3wdp3x'))
TERRASWAP_BLUNA_BOND_CONFIG_COL_4=PoolConfig(contract_address='terra1mtwph2juhj0rvjz7dy92gvl6xvukaxu8rfv8ts', token=Token('terra1kc87mu460fwkqte29rquh4hc20m54fxwtsx7gp'))

TERRASWAP_BLUNA_CONFIG = PoolConfig(contract_address='terra13e4jmcjnwrauvl2fnjdwex0exuzd8zrh5xk29v', token=Token('terra1u0t35drzyy0mujj8rkdyzhe264uls4ug3wdp3x'))
TERRASWAP_BLUNA_CONFIG_COL_4 = PoolConfig(contract_address='terra1jxazgm67et0ce260kvrpfv50acuushpjsz2y0p', token=Token('terra1kc87mu460fwkqte29rquh4hc20m54fxwtsx7gp'))

TERRASWAP_UST_CONFIG = PoolConfig(contract_address='terra156v8s539wtz0sjpn8y8a8lfg8fhmwa7fy22aff', token=NativeToken('uusd'))
TERRASWAP_KRT_CONFIG = PoolConfig(contract_address='terra1rfzwcdhhu502xws6r5pxw4hx8c6vms772d6vyu', token=NativeToken('ukrw'))

TERRASWAP_UST_CONFIG_COL_4 = PoolConfig(contract_address='terra1tndcaqxkpc5ce9qee5ggqf430mr2z3pefe5wj6', token=NativeToken('uusd'))
TERRASWAP_KRT_CONFIG_COL_4 = PoolConfig(contract_address='terra1zw0kfxrxgrs5l087mjm79hcmj3y8z6tljuhpmc', token=NativeToken('ukrw'))

TEQUILA_AUST = "terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl"
TEQUILA_ANCHOR_MONEY_MARKET = "terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal"
TEQUILA_SEIGNORAGE = "terra1z3sf42ywpuhxdh78rr5vyqxpaxa0dx657x5trs"
