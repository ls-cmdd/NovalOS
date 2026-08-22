use std::fs;
use std::path::PathBuf;

const SECTOR_SIZE: usize = 512;
const NOVELFS_PARTITION_TYPE: u8 = 0xDA;
const NOVELFS_PARTITION_SECTORS: u32 = 8;

fn put_u32(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_directory_entry(image: &mut [u8], offset: usize, name: &[u8], data_lba: u32, length: u32) {
    image[offset..offset + 32].fill(0);
    image[offset..offset + name.len()].copy_from_slice(name);
    put_u32(image, offset + 32, data_lba);
    put_u32(image, offset + 36, length);
}

fn append_novelfs_partition(path: &PathBuf) {
    let mut image = fs::read(path).expect("failed to read BIOS image for NovelFS append");
    assert_eq!(
        image.len() % SECTOR_SIZE,
        0,
        "BIOS image is not sector aligned"
    );
    let partition_start = (image.len() / SECTOR_SIZE) as u32;
    let partition_bytes = NOVELFS_PARTITION_SECTORS as usize * SECTOR_SIZE;
    let old_len = image.len();
    image.resize(old_len + partition_bytes, 0);

    // Add a third MBR entry. CHS fields are legacy placeholders; LBA is used.
    let entry = 446 + 2 * 16;
    image[entry] = 0;
    image[entry + 1..entry + 4].fill(0);
    image[entry + 4] = NOVELFS_PARTITION_TYPE;
    image[entry + 5..entry + 8].fill(0);
    put_u32(&mut image, entry + 8, partition_start);
    put_u32(&mut image, entry + 12, NOVELFS_PARTITION_SECTORS);
    image[510] = 0x55;
    image[511] = 0xAA;

    let base = old_len;
    image[base..base + 8].copy_from_slice(b"NOVELFS1");
    put_u32(&mut image, base + 8, 1); // format version
    put_u32(&mut image, base + 12, 1); // directory entry count
    put_u32(&mut image, base + 16, 1); // directory starts at relative sector 1
    put_u32(&mut image, base + 20, 2); // file data starts at relative sector 2

    let init_data = fs::read("ramdisk/init.txt").expect("ramdisk/init.txt missing");
    let directory = base + SECTOR_SIZE;
    write_directory_entry(
        &mut image,
        directory,
        b"init.txt",
        2,
        init_data.len() as u32,
    );
    let data = base + 2 * SECTOR_SIZE;
    image[data..data + init_data.len()].copy_from_slice(&init_data);
    fs::write(path, image).expect("failed to write NovelFS BIOS image");
}

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR missing"));
    let kernel = match std::env::var_os("NOVEL_KERNEL_ARTIFACT") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(
            std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").expect("kernel artifact missing"),
        ),
    };

    println!("cargo:rerun-if-changed={}", kernel.display());

    let bios_path = out_dir.join("novel_os-bios.img");
    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .expect("failed to create BIOS disk image");
    append_novelfs_partition(&bios_path);
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());

    let uefi_path = out_dir.join("novel_os-uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&uefi_path)
        .expect("failed to create UEFI disk image");
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());

    println!("cargo:rerun-if-changed=kernel/src");
    println!("cargo:rerun-if-changed=ramdisk");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=NOVEL_KERNEL_ARTIFACT");
}
