import asyncio
from datetime import timedelta


async def loop(op, sleep_time: timedelta):
    while True:
        op()
        await asyncio.sleep(sleep_time.total_seconds())


def execute_loop(op, sleep_time: timedelta):
    asyncio.get_event_loop().run_until_complete(loop(op=op, sleep_time=sleep_time))
