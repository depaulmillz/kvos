#![doc = include_str!("../README.md")]

use clap::Parser;

#[doc(hidden)]
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long, action)]
    debug : bool,
    #[arg(short, long, action)]
    show_int: bool,
    #[arg(short, long, action)]
    gtk: bool,
    #[arg(short, long, action)]
    buzzer: bool,
}

#[test]
fn test_all() {
    let iso_path = env!("TEST_ISO_PATH");

    let mut cmd1 = std::process::Command::new("ls");
    cmd1.arg(format!("{}", iso_path));
    let output = cmd1.output().unwrap();
    panic!("{} {}", String::from_utf8(output.stderr).unwrap(), String::from_utf8(output.stdout).unwrap());

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("-device");
    cmd.arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    cmd.arg("-serial");
    cmd.arg("stdio");
    cmd.arg("-display");
    cmd.arg("none");
    //ACHI
    // cmd.arg("-machine");
    // cmd.arg("q35");
    // cmd.arg("-drive");
    // cmd.arg(format!("file={}\\..\\build\\harddrive.raw,index=0,if=ide,format=raw", iso_path));
    //
    //
    cmd.arg("-drive");
    cmd.arg("file=disk.img,format=raw");
    cmd.arg("-drive");
    cmd.arg("file=unit_test_disk.img,format=raw");
    cmd.arg("-cdrom");
    cmd.arg(format!("{}", iso_path));
    cmd.args(&["-smp", "cpus=2"]);
    let mut child = cmd.spawn().unwrap();
    if let Some(code) = child.wait().expect("unable to run qemu").code() {
        if code != 33 {
            panic!("Testing failed {}", code);
        }
    }
}

#[doc(hidden)]
fn main() {

    let args = Args::parse();

    println!("{:?}", args);

    let iso_path = env!("ISO_PATH");
    let binary_path = env!("BIN_PATH");
    let initexec_path = env!("INITEXEC_PATH");
    println!("Starting qemu to run {}", iso_path);
    println!("Binary is {}", binary_path);
    println!("Initexec is {}", initexec_path);
    
    let mut cmd = std::process::Command::new("qemu-system-x86_64");

    cmd.arg("-display");
    if args.gtk {
        println!("Using gtk");
        cmd.arg("gtk");
    } else {
        println!("Using sdl");
        cmd.arg("sdl");
    }
    cmd.arg("-serial");
    cmd.arg("stdio");

    if args.debug {
        cmd.arg("-S");
        cmd.arg("-s");
    }
    if args.show_int {
        cmd.arg("-d");
        cmd.arg("int");
    }

    if args.buzzer {
        cmd.args(&["-audiodev","pa,id=speaker", "-machine", "pcspk-audiodev=speaker"]); 
    }
    cmd.arg("-drive");
    cmd.arg("file=disk.img,format=raw");
    cmd.arg("-drive");
    cmd.arg("file=unit_test_disk.img,format=raw");
    // cmd.arg("-monitor");
    // cmd.arg("stdio");
    cmd.arg("-cdrom");
    cmd.arg(format!("{}", iso_path));
    cmd.args(&["-smp", "cpus=2"]);
    let mut child = cmd.spawn().unwrap();
    if !child.wait().expect("unable to run qemu").success() {
        panic!("Launching failed");
    }
}
