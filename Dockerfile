FROM ch8728847/nf:base
MAINTAINER "Jianfeng Wang <pkueewjf@gmail.com>"

RUN apt-get -yq update && apt-get -yq install \
    iputils-ping \
    bash \
    sudo \
    libnuma-dev \
    libsctp-dev \
    && apt-get -yq clean

RUN mkdir /app/NetBricks
COPY ./build.sh /app/NetBricks
COPY ./target /app/NetBricks/target

RUN mkdir /app/NetBricks/3rdparty
COPY ./3rdparty/tools /app/NetBricks/3rdparty

RUN mkdir -p /app/NetBricks/3rdparty/dpdk/build/
COPY ./3rdparty/dpdk/build/lib /app/NetBricks/3rdparty/dpdk/build/