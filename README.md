# Vecno-cpu-miner

A Rust binary for file encryption to multiple participants.

## Installation

### From Sources

With Rust's package manager cargo, you can install vecno-cpu-miner via:

```sh
cargo build --release --all  
```

### From Binaries

The [release page](https://github.com/Vecno-Foundation/vecno-cpu-miner/releases) includes precompiled binaries for Linux, macOS and Windows.

# Usage

To start mining you need to run [vecnod](https://github.com/Vecno-Foundation/vecnod) and have an address to send the rewards to.

Help:

```
vecno-cpu-miner 0.0.1
A Vecno high performance CPU miner

USAGE:
    vecno-cpu-miner [FLAGS] [OPTIONS] --mining-address <mining-address>

FLAGS:
    -d, --debug                   Enable debug logging level
    -h, --help                    Prints help information
        --mine-when-not-synced    Mine even when vecnod says it is not synced, only useful when passing `--allow-submit-
                                  block-when-not-synced` to vecnod  [default: false]
        --testnet                 Use testnet instead of mainnet [default: false]
    -V, --version                 Prints version information

OPTIONS:
        --devfund <devfund-address>            Mine a percentage of the blocks to the Vecno devfund [default: Off]
        --devfund-percent <devfund-percent>    The percentage of blocks to send to the devfund [default: 1]
    -s, --vecnod-address <vecnod-address>      The IP of the vecnod instance [default: 127.0.0.1]
    -a, --mining-address <mining-address>      The Vecno address for the miner reward
    -t, --threads <num-threads>                Amount of miner threads to launch [default: number of logical cpus]
    -p, --port <port>                          Vecnod port [default: Mainnet = 7110, Testnet = 7210]
```

To start mining you just need to run the following:

`./vecno-cpu-miner --mining-address vecno:XXXXX`

This will run the miner on all the available CPU cores.

# Devfund

**NOTE: This feature is off by default** `<br>`
The devfund is a fund managed by the Vecno community in order to fund Vecno development `<br>`
A miner that wants to mine a percentage into the dev-fund can pass the following flags: `<br>`
`vecno-cpu-miner --mining-address= XXX --devfund=vecno:qqtsqwxa3q4aw968753rya4tazahmr7jyn5zu7vkncqlvk2aqlsdsah9ut65e` `<br>`
and can pass `--devfund-precent=XX.YY` to mine only XX.YY% of the blocks into the devfund (passing `--devfund` without specifying a percent will default to 1%)

# Donation Address

vecno:qqtsqwxa3q4aw968753rya4tazahmr7jyn5zu7vkncqlvk2aqlsdsah9ut65e
