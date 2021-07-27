import aiohttp
import asyncio
from datetime import timedelta

import terra_sdk


async def loop(op, sleep_time: timedelta, recover_time: timedelta = timedelta(seconds=10)):
    while True:
        try:
            op()
        except (aiohttp.client_exceptions.ClientConnectorError, terra_sdk.exceptions.LCDResponseError):
            await asyncio.sleep(recover_time.total_seconds())
        await asyncio.sleep(sleep_time.total_seconds())


def execute_loop(op, sleep_time: timedelta):
    asyncio.get_event_loop().run_until_complete(loop(op=op, sleep_time=sleep_time))
