rustup override set nightly
rustup component add rust-src
rustup target add x86_64-unknown-none

sudo apt-get install -y nasm grub-common grub-pc-bin xorriso

./create_disks.sh
