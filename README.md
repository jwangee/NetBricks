Container Building Process
--------

## 1) Create a build environment
You can either setup a builder VM using Vagrant and Virtualbox (1a) or setup your host (1b) to build NetBricks.

### 1a) Use builder VM
This requires Vagrant and Virtualbox to be installed. You can build Netbricks with the following commands:
```sh
vagrant up
vagrant ssh
$ cd /NetBricks
$ make
```
### 1b) Install dependencies

#### Install Package Dependencies
```sh
sudo apt-get install -y libgnutls30 libgnutls-openssl-dev libcurl4-gnutls-dev libnuma-dev \
libpcap-dev libsctp-dev linux-headers-generic build-essential clang
```

#### Install Rust
``` sh
curl https://sh.rustup.rs -sSf > /tmp/rustup.sh
sh /tmp/rustup.sh -y --default-toolchain nightly-2019-01-19
rm /tmp/rustup.sh
echo "source $HOME/.cargo/env" >> $HOME/.bashrc
```

## 2) Get container
You can either pull the docker image from DockerHub (2a) or create it on your own (2b).

### 2a) Download from DockerHub

A recent build is available on DockerHub at [levaitamas/netbricks](https://hub.docker.com/r/levaitamas/netbricks).

### 2b) Build container

Requires Docker to be installed.
```sh
make docker
```

## 3) Run NFs in container

Start container with `./run_nf.sh`.

### 3a) NF Chains
In the container you can start NF chains by issuing:
```sh
/app/main run <chain> -c 0 -p <nic_pcie_addr>
```

Supported NF chains are:

* [macswap](faas-nfv/macswap)
* [vlanpop-acl](faas-nfv/vlanpop-acl)
* [acl-urlfilter-chacha](faas-nfv/acl-urlfilter-chacha)
* [acl-distribnat](faas-nfv/acl-distribnat)

### 3b) Benchmarks

In the container you can start benchmarks by issuing:
```sh
/app/main run <benchmark> -n <nnumber_of_measurements>
```

Supported benchmarks are:
* [scheduler_benchmark](faas-nfv/scheduler_benchmark): Measures the StandaloneSceduler's scheduling overhead

--------
# UPSTREAM README:

[NetBricks](http://netbricks.io/) is a Rust based framework for NFV development. Please refer to the
[paper](https://people.eecs.berkeley.edu/~apanda/assets/papers/osdi16.pdf) for information
about the architecture and design. Currently NetBricks requires a relatively modern Linux version.

Building
--------
NetBricks can be built either using a Rust nightly build or using Rust built from the current Git head. In the later
case we also build [`musl`](https://www.musl-libc.org/) and statically link to things. Below we provide basic instructions for both.

Finally, in addition to the above options, NetBricks can also be built within a Docker container. In this case, you do
not need to install any of the dependencies, and the final product can be run the same. However to run NetBricks you
still need to be on a machine that is correctly configured to run DPDK (see
[here](http://dpdk.org/doc/guides-16.07/linux_gsg/quick_start.html) for more details), and you still need to install
Rust nightly (for libraries). Please see the [container build instructions](#container-build) for more information.

Using Rust Nightly
------------------
First obtain Rust nightly. I use [rustup](https://rustup.rs), in which case the following works

```
curl https://sh.rustup.rs -sSf | sh  # Install rustup
source $HOME/.cargo/env
rustup install nightly
rustup default nightly
```

This repository was tested using rust 2019-01-18, to use this version do:
```
rustup default nightly-2019-01-19
```

Then clone this repository and run `build.sh`

```
./build.sh
```

This should download DPDK, and build all of NetBricks.

Using Rust from Git
-------------------
The instructions for doing so are simple, however building takes significantly longer in this case (and consumes tons of
memory), so do this only if you have lots of time and memory. Building is as simple as

```
export RUST_STATIC=1
./build.sh
```

Dependencies
------------
Building NetBricks requires the following dependency packages (on Debian):

```
apt-get install libgnutls30 libgnutls-openssl-dev libcurl4-gnutls-dev libnuma-dev libpcap-dev
```

NetBricks also supports using SCTP as a control protocol. SCTP support requires the use of `libsctp` (this is an
optional dependency) which can be installed on Debian using:

```
apt-get install libsctp-dev
```

Tuning
------
Changing some Linux parameters, including disabling C-State, and P-State; and isolating CPUs can greatly benefit NF
performance. In addition to these boot-time settings, runtime settings (e.g., disabling uncore frequency scaling and
setting the appropriate flags for Linux power management QoS) can greatly improve performance. The
[energy.sh](scripts/tuning/energy.sh) in [scripts/tuning](scripts/tuning) will set these parameter appropriately, and
it is recommended you run this before running the system.

Container Build
---------------
You must have [Docker](https://www.docker.com/) installed. Once this is done, just run

```
./build.sh build_container
```

This will build and copy the binaries over to the `target` subdirectory. As noted above, you can run it if you have a
DPDK compatible machine.

Example NFs
-----------
This repository includes a set of example NFs under the `test` directory. A complete list of example can be found by
running
```
./build.sh run
```

The build script can be used to run these examples as

```
./build.sh run <example name> <options>
```

Passing `-h` will provide a list of options. All of these examples accept one or more ports as input. Ports can be
specified as one of:

-   PCI ID of a NIC
-   `dpdk:<PMD spec>` where PMD spec can be something like
    `dpdk:eth_pcap0,rx_pcap=$HOME/tcpflow/tests/udp.pcap,tx_pcap=out.pcap` which specifies a PCAP file should be used.
    See DPDK source for other PMD drivers that are available.
-   `ovs:<integer>` to connect to an OpenVSwitch DPDK ring port (`dpdkr`).
-   `bess:<port name>` to connect to a BESS `ZeroCopyVPort`

Future Work
-----------
Support for [`futures`](https://github.com/alexcrichton/futures-rs) for control plane functionality.
