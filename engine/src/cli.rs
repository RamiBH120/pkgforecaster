// use clap::Parser;
// use std::process::Command;
// use std::fs;
// use anyhow::Result;
// use pkgforecaster_engine::parse_apt_simulation;

// #[derive(Parser)]
// #[command(name = "engine-cli")]
// struct Args {
//     /// Path to a file containing apt simulation output (for testing)
//     #[arg(short, long)]
//     file: Option<String>,

//     /// Run real apt simulation locally (requires apt)
//     #[arg(long)]
//     apt_sim: bool,
// }

// fn main() -> Result<()> {
//     let args = Args::parse();

//     let output = if let Some(f) = args.file {
//         fs::read_to_string(f)?
//     } else if args.apt_sim {
//         // spawn apt-get -s upgrade
//         let out = Command::new("apt-get")
//             .arg("-s")
//             .arg("upgrade")
//             .output()?;
//         String::from_utf8_lossy(&out.stdout).into_owned()
//     } else {
//         // fallback: read bundled dummy file
//         fs::read_to_string("data/dummy_simulation.txt")?
//     };

//     let sim = parse_apt_simulation(&output);
//     println!("{}", serde_json::to_string_pretty(&sim)?);
//     Ok(())
// }

use clap::Parser;
use anyhow::Result;
use std::fs;
use crate::runner;
use crate::parse;

#[derive(Parser)]
#[command(name = "engine-cli")]
struct Args {
    /// Path to a file containing apt simulation output (for testing)
    #[arg(short, long)]
    file: Option<String>,

    /// Use a disposable debootstrap release (e.g. focal). If provided, engine creates a transient rootfs and runs apt-get -s inside it (requires root and debootstrap installed)
    #[arg(long)]
    debootstrap: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let output = if let Some(f) = args.file {
        fs::read_to_string(f)?
    } else if let Some(rel) = args.debootstrap {
        println!("Creating debootstrap root for release: {}", rel);
        match runner::simulate_with_debootstrap(&rel) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("debootstrap simulation failed: {:?}", e);
                std::process::exit(1);
            }
        }
    } else {
        // fallback: attempt to run apt-get -s locally (dangerous, but non-destructive)
        let out = std::process::Command::new("apt-get")
            .arg("-s")
            .arg("upgrade")
            .output()?;
        String::from_utf8_lossy(&out.stdout).into_owned()
    };

    let sim = parse::parse_apt_simulation(&output);
    println!("{}", serde_json::to_string_pretty(&sim)?);
    Ok(())
}
