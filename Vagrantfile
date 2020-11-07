# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "bento/ubuntu-18.04"

  config.vm.synced_folder ".", "/NetBricks"

  config.vm.provider "virtualbox" do |vb|
    vb.gui = false
    vb.memory = "4096"
    vb.cpus = "1"
  end

  config.vm.provision "deps", type: "shell", privileged: false, inline: <<-SHELL
    sudo apt update
    sudo apt install -y \
         libgnutls30 \
         libgnutls-openssl-dev \
         libcurl4-gnutls-dev \
         libnuma-dev \
         libpcap-dev \
         libsctp-dev \
         linux-headers-generic \
         build-essential \
         clang
  SHELL

  config.vm.provision "docker", type: "shell", privileged: false, inline: <<-SHELL
    sudo apt remove -y docker docker-engine docker.io containerd runc
    sudo apt-get install -y \
         apt-transport-https \
         ca-certificates \
         curl \
         gnupg-agent \
         software-properties-common
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
    sudo add-apt-repository \
         "deb [arch=amd64] https://download.docker.com/linux/ubuntu \
         $(lsb_release -cs) \
         stable"
    sudo apt update
    sudo apt install -y docker-ce docker-ce-cli containerd.io
  SHELL

  config.vm.provision "rust", type: "shell", privileged: false, inline: <<-SHELL
    curl https://sh.rustup.rs -sSf > /tmp/rustup.sh
    sh /tmp/rustup.sh -y --default-toolchain nightly-2019-01-19
    rm /tmp/rustup.sh
    echo "source $HOME/.cargo/env" > $HOME/.bashrc
  SHELL

  config.vm.provision "hugepages", type: "shell", privileged: false, inline: <<-SHELL
    echo 'vm.nr_hugepages=1024' | sudo tee /etc/sysctl.d/hugepages.conf
    sudo mount -t hugetlbfs none /dev/hugepages
    sudo sysctl -w vm.nr_hugepages=1024
  SHELL

  config.vm.provision "clean", type: "shell", privileged: false, inline: <<-SHELL
    sudo apt -y clean
    sudo apt -y update
  SHELL

end
