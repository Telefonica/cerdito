# cerdito

[![Made with Rust](https://forthebadge.com/images/badges/made-with-rust.svg)](https://www.rust-lang.org)
[![Gluten Free](https://forthebadge.com/images/badges/gluten-free.svg)](https://en.wikipedia.org/wiki/Gluten-free_diet)
[![It works](https://forthebadge.com/images/badges/it-works-why.svg)](https://youtu.be/dQw4w9WgXcQ)

![Piggy Bank](cerdito.svg)

Save a few cents on your cloud infrastructure.

## What does _cerdito_ do?

_cerdito_ is configured through a _toml_ file in which you can indicate the
Atlas clusters or AKS you want to pause and the Kubernetes deployments you
want to be scaled to zero. Note that each module goes separately so it is
possible to use _cerdito_ only to pause Atlas clusters, AKS or to scale
deployments to zero.

To connect to Atlas you will need a public key and a private API access key,
for AKS you will need a Service Principal and for Kubernetes you will need
the _cubeconfig_ file.

## Installation

### From binary

Simply download latest release from [releases page][releases].

[releases]: https://github.com/Telefonica/cerdito/releases

### From source

#### Installing Rust

_cerdito_ build has been tested with current Rust stable release version.
You can install Rust from your distribution package or use
[`rustup`][rustup].
```sh
rustup default stable
```

If you prefer, you can use the stable version only for install _cerdito_.
```sh
rustup override set stable
```

[rustup]: https://rustup.rs/

#### Building _cerdito_

To build _cerdito_ simply execute the following commands.
```sh
git clone git@github.com:Telefonica/cerdito.git
cd cerdito
cargo build --release
```

Once it finishes building, you will find the binary in the `target/release/`
directory.

### With Docker

Build docker container.
```sh
docker build -t cerdito .
```

Run docker container.
```sh
docker run -t -i --rm cerdito
```

Now you can execute _cerdito_ command. Remember to create a configuration
file `cerdito.toml`. If you already have one you can run _cerdito_ directly
in this way.
```sh
docker run -t -i --rm \
  -v $(pwd)/cerdito.toml:/cerdito.toml cerdito \
  cerdito -v -c /cerdito.toml SUBCOMMAND
```

## Run

First take a look to `cerdito.toml` file to configure _cerdito_. Is self
explanatory.

You can place `cerdito.toml` in same directory where you run _cerdito_ or
where you want as long as you indicate it from the command line or with the
`CERDITO_CONFIG` environment variable.

Once you have configured _cerdito_ you can run `cerdito start` to resume your
cloud infraestructure or `cerdito stop` to pause it. _cerdito_ has some
command line options to tell it where to read the configuration.

```
Usage: cerdito [OPTIONS] <COMMAND>

Commands:
  start    Start all configured elements
  stop     Stop all configured elements
  version  Prints version information
  help     Print this message or the help of the given subcommand(s)

Options:
  -c, --config <config>          Custom configuration file path
  -k, --kubeconfig <kubeconfig>  Custom kubeconfig file path
  -v, --verbose...               Sets the level of verbosity
  -h, --help                     Print help
  -V, --version                  Print version
```

By default _cerdito_ does not show anything when it is running, if you want
to see what it is doing you can launch it with `CERDITO_LOGLEVEL=info`
environment variable or with `-v` option.

_cerdito_ supports the following environment variables.

| Variable | Description |
| --- | --- |
| `CERDITO_CONFIG` | Config file location |
| `CERDITO_LOGLEVEL` | Log level, effective values are `error`, `warn`, `info`, `debug` and `trace` |
| `MONGODB_ATLAS_PUBLIC_KEY` | Atlas public key, to avoid having to write it in the configuration file |
| `MONGODB_ATLAS_PRIVATE_KEY` | Atlas private key |
| `AZURE_TENANT_ID` | Azure tenant ID |
| `AZURE_CLIENT_ID` | Azure SP client ID |
| `AZURE_CLIENT_SECRET` | Azure SP client secret |
| `KUBECONFIG` | Location of kubeconfig file, by default `~/.kube/config` is used (if not specified in the configuration file) |
