use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap(); // location of output directory

    let asm_files = vec!["multiboot_header", "boot", "long_mode_init"];

    for f in &asm_files {
        if !Command::new("nasm").args(&["-felf64"])
                            .arg(&format!("bootloader/src/{}.asm", f))
                            .args(&["-o"])
                            .arg(&format!("{}/{}.o", out_dir, f))
                            .status().expect("nasm failed").success()
        {
            panic!("nasm failed");
        }
    }

    let kernel = std::env::var_os("CARGO_STATICLIB_FILE_KERNEL_kernel").expect("Should have kernel");
    let shell = std::env::var_os("CARGO_BIN_FILE_INITEXEC_initexec").expect("Should have initexec");

    let asm_o_files = asm_files.iter().map(|x| format!("{}/{}.o", out_dir, x)).collect::<Vec<String>>();

    if !Command::new("ld").args(&["-n", "-o"])
                      .arg(&format!("{}/kernel.bin", out_dir))
                      .args(&["-T", "bootloader/linker.ld"])
                      .args(&asm_o_files)
                      .args(&[kernel.clone().into_string().unwrap()])
                      .status()
                      .expect("ld failed").success() 
    {
        panic!("Unable to link");
    }

    for f in &asm_files {
        println!("cargo:rerun-if-changed=bootloader/src/{}.asm", f);
    }

    println!("cargo:rerun-if-changed={}", kernel.into_string().unwrap());
    println!("cargo:rerun-if-changed=bootloader/linker.ld");
    println!("cargo:rerun-if-changed=bootloader/grub.cfg");
    println!("cargo:rerun-if-changed=bootloader/grub-test.cfg");

    Command::new("mkdir").args(&["-p"])
                         .arg(&format!("{}/isofiles/boot/grub", out_dir))
                         .status().expect("unable to execute mkdir");
    
    if !Command::new("cp").args(&["bootloader/grub.cfg"])
                      .arg(&format!("{}/isofiles/boot/grub", out_dir))
                      .status().expect("Copy worked").success() {
        panic!("Unable to copy grub cfg");
    }

    if !Command::new("cp").arg(&format!("{}/kernel.bin", out_dir))
                      .arg(&format!("{}/isofiles/boot/kernel.bin", out_dir))
                      .status().unwrap().success() {
        panic!("Unable to move kernel bin");
    }

    if !Command::new("cp").arg(&format!("{}", shell.clone().into_string().unwrap()))
                      .arg(&format!("{}/isofiles/initexec", out_dir))
                      .status().unwrap().success() {
        panic!("Unable to move initexec");
    }

    let mut grub_cmd = Command::new("grub-mkrescue");
    grub_cmd.args(&["-o"])
            .arg(&format!("{}/kvos.iso", out_dir))
            .arg(&format!("{}/isofiles", out_dir));

    let command_out = grub_cmd.output().expect("unable to use grub");

    if !command_out.status.success()
    {
        panic!("grub-mkrescue failed stderr : {} stdout : {}", String::from_utf8(command_out.stderr).unwrap(), String::from_utf8(command_out.stdout).unwrap());
    }
 
    if !Command::new("cp").args(&["bootloader/grub-test.cfg"])
                      .arg(&format!("{}/isofiles/boot/grub/grub.cfg", out_dir))
                      .status().unwrap().success() {
        panic!("Unable to copy grub test config");
    }

    if !Command::new("grub-mkrescue").args(&["-o"])
                      .arg(&format!("{}/kvos-test.iso", out_dir))
                      .arg(&format!("{}/isofiles", out_dir))
                      .status().expect("unable to use grub")
                      .success()
    {
        panic!("grub-mkrescue failed");
    }
    
    println!("cargo:rustc-env=ISO_PATH={}/kvos.iso", out_dir);
    println!("cargo:rustc-env=TEST_ISO_PATH={}/kvos-test.iso", out_dir);
    println!("cargo:rustc-env=BIN_PATH={}/kernel.bin", out_dir);
    println!("cargo:rustc-env=INITEXEC_PATH={}", shell.clone().into_string().unwrap());

}

