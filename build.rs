fn main() {
    #[cfg(target_os = "windows")]
    download_windows_npcap_sdk().unwrap();
}

#[cfg(target_os = "windows")]
fn download_windows_npcap_sdk() -> anyhow::Result<()> {
    use std::{
        env, fs,
        io::{self, Write},
        path::PathBuf,
    };

    use anyhow::anyhow;
    use http_req::request;
    use zip::ZipArchive;

    println!("cargo:rerun-if-changed=build.rs");

    // get npcap SDK
    const NPCAP_SDK: &'static str = "npcap-sdk-1.13.zip";

    let npcap_sdk_download_url = format!("https://npcap.com/dist/{NPCAP_SDK}");
    let cache_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("target");
    let npcap_sdk_cache_path = cache_dir.join(NPCAP_SDK);

    let npcap_zip = match fs::read(&npcap_sdk_cache_path) {
        // use cached
        Ok(zip_data) => {
            println!("Found cached npcap SDK");
            zip_data
        }
        // download SDK
        Err(_) => {
            println!("Downloading npcap SDK");

            // download
            let mut zip_data = vec![];
            let _res = request::get(npcap_sdk_download_url, &mut zip_data)?;

            // write cache
            fs::create_dir_all(cache_dir)?;
            let mut cache = fs::File::create(npcap_sdk_cache_path)?;
            cache.write_all(&zip_data)?;

            zip_data
        }
    };

    // extract DLL
    let lib_path = if cfg!(target_arch = "aarch64") {
        "Lib/ARM64/Packet.lib"
    } else if cfg!(target_arch = "x86_64") {
        "Lib/x64/Packet.lib"
    } else if cfg!(target_arch = "x86") {
        "Lib/Packet.lib"
    } else {
        panic!("Unsupported target!")
    };
    let mut archive = ZipArchive::new(io::Cursor::new(npcap_zip))?;
    let mut npcap_lib = archive.by_name(&lib_path)?;

    // write DLL
    let lib_dir = PathBuf::from(env::var("OUT_DIR")?).join("npcap_sdk");
    let lib_path = lib_dir.join("Packet.lib");
    fs::create_dir_all(&lib_dir)?;
    let mut lib_file = fs::File::create(lib_path)?;
    io::copy(&mut npcap_lib, &mut lib_file)?;

    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir
            .to_str()
            .ok_or(anyhow!("{lib_dir:?} is not valid UTF-8"))?
    );

    Ok(())
}
