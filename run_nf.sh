sudo docker run -it --pid=host --rm --privileged --cpuset-cpus="0-1" \
    -v /sys/bus/pci/drivers:/sys/bus/pci/drivers \
    -v /sys/kernel/mm/hugepages:/sys/kernel/mm/hugepages \
    -v /mnt/huge:/mnt/huge -v /dev:/dev \
    -v /sys/devices/system/node:/sys/devices/system/node \
    -v /var/run:/var/run -v /tmp/sn_vports:/tmp/sn_vports \
    ch8728847/netbricks:test bash
