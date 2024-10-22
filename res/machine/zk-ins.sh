#!/bin/bash
# if not root user, exit
if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit
fi

VER=$1
ADDR=$2
WORKER=$(hostname -I | awk '{print $1}')

echo "This script will install nvidia driver/CUDA and ZKWORK prover in your ubuntu system, and auto configure it to run on boot"

# if no VER or ADDR quit
if [ -z "$VER" ] || [ -z "$ADDR" ] ; then
  echo "Usage: $0 <zkwork-version> <receive-address>"
  echo "Example: $0 0.2.3 aleo1spkkxewxj2dl2lgdps9xr28093p5nxsvjv55g2unmqfu0hmwyuysmf4qp3"
  exit 1
fi

echo "deb http://cz.archive.ubuntu.com/ubuntu jammy main" >> /etc/apt/sources.list && apt update && DEBIAN_FRONTEND=noninteractive apt install libc6 -y && apt install -y g++-11

ubuntu-drivers install

wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.1-1_all.deb

dpkg -i cuda-keyring_1.1-1_all.deb
apt-get update
apt-get -y install cuda-toolkit-12-6
apt-get -y install jq

wget https://gh-proxy.com/https://github.com/6block/zkwork_aleo_gpu_worker/releases/download/v${VER}/aleo_prover-v${VER}_full.tar.gz

tar -xvf aleo_prover-v${VER}_full.tar.gz -C /opt

# geneate run/stop scripts
echo "#!/bin/bash
cd /opt/aleo_prover
./aleo_prover --address $ADDR --pool aleo.asia1.zk.work:10003 --pool aleo.hk.zk.work:10003 --pool aleo.jp.zk.work:10003 --custom_name $WORKER >> prover.log 2>&1
echo \$! > aleo_prover.pid
" > /opt/aleo_prover/start.sh
chmod +x /opt/aleo_prover/start.sh

echo "#!/bin/bash
kill -9 \$(cat /opt/aleo_prover/aleo_prover.pid)
" > /opt/aleo_prover/stop.sh
chmod +x /opt/aleo_prover/stop.sh

# add systemd service to run PWD/aleo_prover/run_prover.sh
echo "[Unit]
Description=Aleo Prover
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/aleo_prover
ExecStart=/opt/aleo_prover/start.sh
ExecStop=/opt/aleo_prover/stop.sh
Restart=always

[Install]
WantedBy=multi-user.target
" > /etc/systemd/system/aleo.service

systemctl daemon-reload
systemctl enable aleo.service
reboot


