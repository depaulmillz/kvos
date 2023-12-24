rustup override set nightly
rustup target add x86_64-unknown-none
rustup component add rust-src

sudo apt-get install -y nasm grub-common grub-pc-bin xorriso qemu qemu-utils

./create_disks.sh
