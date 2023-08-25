fn main() {
    #[cfg(target_os = "windows")]
    download_windows_pcap_sdk()
}

#[cfg(target_os = "windows")]
fn download_windows_pcap_sdk() {
    use std::{
        env, fs,
        io::{self, Write},
    };

    use http_req::request;
    use zip::ZipArchive;

    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();

    let mut pcap_zip = Vec::new();
    let res = request::get("https://npcap.com/dist/npcap-sdk-1.13.zip", &mut pcap_zip).unwrap();
    eprintln!("{:?}", res);

    let lib_dir = if cfg!(target_arch = "aarch64") {
        "Lib/ARM64"
    } else if cfg!(target_arch = "x86_64") {
        "Lib/x64"
    } else if cfg!(target_arch = "x86") {
        "Lib"
    } else {
        panic!("Unsupported target!")
    };
    let lib_name = "Packet.lib";
    let lib_path = format!("{}/{}", lib_dir, lib_name);

    let mut archive = ZipArchive::new(io::Cursor::new(pcap_zip)).unwrap();
    let mut pcap_lib = match archive.by_name(&lib_path) {
        Ok(lib) => lib,
        Err(err) => {
            panic!("{}", err);
        }
    };

    fs::create_dir_all(format!("{}/{}", out_dir, lib_dir)).unwrap();
    let mut pcap_lib_file = fs::File::create(format!("{}/{}", out_dir, lib_path)).unwrap();
    io::copy(&mut pcap_lib, &mut pcap_lib_file).unwrap();
    pcap_lib_file.flush().unwrap();

    println!("cargo:rustc-link-search=native={}/{}", out_dir, lib_dir);
}
