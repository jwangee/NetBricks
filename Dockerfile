FROM ch8728847/nf:base
MAINTAINER "Tamas Levai <levait@tmit.bme.hu>"

RUN apt-get -yq update \
    && apt-get -yq --no-install-recommends install \
       iputils-ping \
       bash \
       sudo \
       libpcap0.8 \
       libnuma1 \
       libsctp1 \
    && apt-get -yq clean && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /app/target/release
COPY ./target/release/acl-distribnat /app/target/release
COPY ./target/release/acl-urlfilter-chacha /app/target/release
COPY ./target/release/vlanpop-acl /app/target/release

COPY ./examples.sh /app/
COPY ./build.sh /app
RUN mkdir /app/native
COPY ./native/libzcsi.so /app/native

RUN mkdir /app/3rdparty
COPY ./3rdparty/tools /app/3rdparty
RUN mkdir -p /app/3rdparty/dpdk/build/
COPY ./3rdparty/dpdk/build/lib /app/3rdparty/dpdk/build/
RUN mkdir -p /app/3rdparty/dpdk-confs
COPY ./3rdparty/dpdk-confs /app/3rdparty/dpdk-confs

RUN rm /app/main && ln -s /app/build.sh /app/main