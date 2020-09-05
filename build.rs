use std::io::Write;

fn main() {
    #[cfg(target_os = "windows")]
    download_winpcap_sdk()
}

#[cfg(target_os = "windows")]
fn download_winpcap_sdk() {
    use http_req::request;
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;

    let out_dir = env::var("OUT_DIR").unwrap();

    let mut reader = Vec::new();
    let res = request::get(
        "https://nmap.org/npcap/dist/npcap-sdk-1.05.zip",
        &mut reader,
    )
    .unwrap();

    println!("Status: {} {}", res.status_code(), res.reason());
    println!("Reader len: {}", reader.len());

    let mut pcapzip = File::create(format!("{}{}", out_dir, "/npcap.zip")).unwrap();
    pcapzip.write_all(reader.as_slice()).unwrap();
    pcapzip.flush().unwrap();

    pcapzip = File::open(format!("{}{}", out_dir, "/npcap.zip")).unwrap();

    #[cfg(target_arch = "x86_64")]
    let lib_path = "Lib/x64/Packet.lib";
    #[cfg(target_arch = "x86")]
    let lib_path = "Lib/Packet.lib";

    let mut zip_archive = zip::ZipArchive::new(pcapzip).unwrap();
    let mut pcaplib = match zip_archive.by_name(lib_path) {
        Ok(pcaplib) => pcaplib,
        Err(err) => {
            panic!(err);
        }
    };

    let mut pcaplib_bytes = Vec::new();
    pcaplib.read_to_end(&mut pcaplib_bytes).unwrap();

    let mut pcaplib_file = File::create(format!("{}{}", out_dir, "/Packet.lib")).unwrap();
    pcaplib_file.write_all(pcaplib_bytes.as_slice()).unwrap();
    pcaplib_file.flush().unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
}
