use std::{env, fs::File};

use clap::CommandFactory;
use clap_complete::Shell;
use clap_mangen::Man;
use eyre::eyre;

fn main() {
    build_completion_manpage().unwrap();

    #[cfg(target_os = "windows")]
    download_windows_npcap_sdk().unwrap();
}

include!("src/cli.rs");

fn build_completion_manpage() -> eyre::Result<()> {
    let mut cmd = Opt::command();

    // build into `BANDWHICH_GEN_DIR` with a fallback to `OUT_DIR`
    let gen_dir: PathBuf = env::var_os("BANDWHICH_GEN_DIR")
        .or_else(|| env::var_os("OUT_DIR"))
        .ok_or(eyre!("OUT_DIR is unset"))?
        .into();

    // completion
    for &shell in Shell::value_variants() {
        clap_complete::generate_to(shell, &mut cmd, "bandwhich", &gen_dir)?;
    }

    // manpage
    let mut manpage_out = File::create(gen_dir.join("bandwhich.1"))?;
    let manpage = Man::new(cmd);
    manpage.render(&mut manpage_out)?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn download_windows_npcap_sdk() -> eyre::Result<()> {
    use std::{
        fs,
        io::{self, Write},
    };

    use http_req::request;
    use zip::ZipArchive;

    println!("cargo:rerun-if-changed=build.rs");

    // get npcap SDK
    const NPCAP_SDK: &str = "npcap-sdk-1.13.zip";

    let npcap_sdk_download_url = format!("https://npcap.com/dist/{NPCAP_SDK}");
    let cache_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("target");
    let npcap_sdk_cache_path = cache_dir.join(NPCAP_SDK);

    let npcap_zip = match fs::read(&npcap_sdk_cache_path) {
        // use cached
        Ok(zip_data) => {
            eprintln!("Found cached npcap SDK");
            zip_data
        }
        // download SDK
        Err(_) => {
            eprintln!("Downloading npcap SDK");

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
    let mut npcap_lib = archive.by_name(lib_path)?;

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
            .ok_or(eyre!("{lib_dir:?} is not valid UTF-8"))?
    );

    Ok(())
}
