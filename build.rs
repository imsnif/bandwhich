use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os.as_str() == "windows" {
        download_winpcap_sdk();
    }

    #[cfg(windows)]
    download_winpcap_dll();
}

fn download_winpcap_sdk() {
    use http_req::request;
    use std::fs::File;
    use std::io::prelude::*;

    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();

    let mut reader = Vec::new();
    // curl -Ssf -I https://nmap.org/npcap/dist/npcap-sdk-1.13.zip | grep Location
    // Location: https://npcap.com/dist/npcap-sdk-1.13.zip
    // jayjamesjay/http_req does not support following redirect feature
    let _res = request::get("https://npcap.com/dist/npcap-sdk-1.13.zip", &mut reader).unwrap();

    let mut pcapzip = File::create(format!("{}{}", out_dir, "/npcap.zip")).unwrap();
    pcapzip.write_all(reader.as_slice()).unwrap();
    pcapzip.flush().unwrap();

    pcapzip = File::open(format!("{}{}", out_dir, "/npcap.zip")).unwrap();

    let lib_name = "Packet.lib";

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let lib_dir = match target_arch.as_str() {
        "x86_64" => {
            "Lib/x64"
        }
        _ => {
             "Lib"
        }
    };

    let lib_path = format!("{}/{}", lib_dir, lib_name);
    let mut zip_archive = zip::ZipArchive::new(pcapzip).unwrap();
    let mut pcaplib = match zip_archive.by_name(lib_path.as_str()) {
        Ok(pcaplib) => pcaplib,
        Err(err) => {
            panic!("lib {} not found, err: {}", lib_path, err);
        }
    };

    let mut pcaplib_bytes = Vec::new();
    pcaplib.read_to_end(&mut pcaplib_bytes).unwrap();

    std::fs::create_dir_all(format!("{}/{}", out_dir, lib_dir)).unwrap();
    let mut pcaplib_file = File::create(format!("{}/{}", out_dir, lib_path)).unwrap();
    pcaplib_file.write_all(pcaplib_bytes.as_slice()).unwrap();
    pcaplib_file.flush().unwrap();

    println!("cargo:rustc-link-search=native={}/{}", out_dir, lib_dir);
}

#[cfg(windows)]
fn download_winpcap_dll() {
    use http_req::request;
    use std::fs::File;
    use std::io::prelude::*;
    use std::process::Command;

    let mut reader = Vec::new();
    let res = request::get(
        "https://github.com/ttys3/bandwhich/releases/download/0.20.1/npcap.zip",
        &mut reader,
    )
    .unwrap();

    if res.status_code().is_redirect() {
        reader.clear();
        let url = res.headers().get("Location").unwrap();
        let _res = request::get(url, &mut reader).unwrap();
    }

    let zip_path = format!("{}{}", "c:\\", "npcap.zip");
    let mut pcapzip = File::create(&zip_path).unwrap();
    pcapzip.write_all(reader.as_slice()).unwrap();
    pcapzip.flush().unwrap();

    let zip_reader = File::open(&zip_path).unwrap();

    let mut archive = zip::ZipArchive::new(zip_reader).unwrap();
    archive.extract("c:\\").unwrap();

    let output = Command::new("cmd")
        .env("NPCAP_DIR", "c:\\npcap")
        .current_dir("c:\\npcap")
        .arg("/C")
        .arg("FixInstall.bat")
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        panic!("CFixInstall.bat failing with error: {:?}", output);
    }
}
