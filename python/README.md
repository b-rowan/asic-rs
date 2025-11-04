# pyasic-rs ![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue) [![pyasic-rs on PyPI](https://img.shields.io/pypi/v/pyasic-rs)](https://pypi.org/project/pyasic-rs)

pyasic-rs is the official python binding library for [asic-rs](https://github.com/256-Foundation/asic-rs).

## Getting Started
The first step to controlling a miner with pyasic-rs is to get the object that represents it, with methods used for data gathering and control.

#### Getting a miner

If you know the IP address of your miner, it is fairly easy to discover it.  Use the `MinerFactory` to select the correct type.

```python
from pyasic_rs import MinerFactory
import asyncio

async def main():
    factory = MinerFactory()
    ip = "192.168.1.10"
    miner = await factory.get_miner(ip)
    # now we can do data gathering or control

if __name__ == "__main__":
    asyncio.run(main())
```

#### Miner discovery

If you don’t know the specific IP of your miner, pyasic-rs can discover it on your network.

```python
from pyasic_rs import MinerFactory
import asyncio

async def main():
    subnet = "192.168.1.0/24"
    factory = MinerFactory.from_subnet(subnet)
    miners = await factory.scan()

if __name__ == "__main__":
    asyncio.run(main())
```


There are other ways to define a discovery range to be scanned, such as:

* Octets

```python
    factory = MinerFactory.from_octets("192", "168", "1", "1-255")
```

* Range string

```python
    factory = MinerFactory.from_subnet("192.168.1.1-255")
```

#### Data gathering

Getting data is very simple with pyasic-rs, everything you need can be gathered with a single call.
Extending the "Getting a miner" example:

```python
from pyasic_rs import MinerFactory
import asyncio

async def main():
    factory = MinerFactory()
    ip = "192.168.1.10"
    miner = await factory.get_miner(ip)
    data = await miner.get_data()

if __name__ == "__main__":
    asyncio.run(main())
```

If you only want specific data, that can be done with individual function calls:

```python
    mac = await miner.get_mac();
```

Most data points from `MinerData` have a corresponding `get_...` function.

#### Miner control

Controlling a miner is very similar to getting data in pyasic-rs.
Again extending the “Getting a miner” example:

```python
from pyasic_rs import MinerFactory
import asyncio

async def main():
    factory = MinerFactory()
    ip = "192.168.1.10"
    miner = await factory.get_miner(ip)
    result = await miner.restart()
    print(result)

if __name__ == "__main__":
    asyncio.run(main())
```
