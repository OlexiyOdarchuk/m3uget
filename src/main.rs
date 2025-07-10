use base64::{Engine as _, engine::general_purpose};
use clap::{Arg, ArgAction, Command as ClapCommand};
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
enum FileNameMode {
    Auto,
    Base,
    Full,
}

fn generate_name(url: &str, mode: &FileNameMode) -> String {
    match mode {
        FileNameMode::Auto => {
            let re = Regex::new(r"s\d{2}e\d{2}").unwrap();
            re.find(url)
                .map(|m| m.as_str().to_lowercase())
                .unwrap_or_else(|| {
                    let ts = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
                    format!("video_{}", ts)
                })
        }
        FileNameMode::Base => general_purpose::URL_SAFE_NO_PAD
            .encode(url)
            .chars()
            .take(16)
            .collect(),
        FileNameMode::Full => url.split('/').rev().nth(1).unwrap_or("video").to_string(),
    }
}

fn download(
    url: &str,
    filename: &str,
    quiet: bool,
    retries: usize,
    limit: Option<&str>,
    proxy: Option<&str>,
) {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg(url)
        .arg("--downloader")
        .arg("ffmpeg")
        .arg("--hls-use-mpegts")
        .arg("--retries")
        .arg(retries.to_string())
        .arg("--fragment-retries")
        .arg(retries.to_string())
        .arg("-o")
        .arg(format!("{}.mp4", filename))
        .arg("-c"); // continue

    if quiet {
        cmd.arg("-q");
    }
    if let Some(lim) = limit {
        cmd.arg("--limit-rate").arg(lim);
    }
    if let Some(px) = proxy {
        cmd.arg("--proxy").arg(px);
    }

    println!("▶️  Downloading: {}", filename);

    match cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) if status.success() => println!("✅ Done: {}", filename),
        Ok(status) => eprintln!("❌ Failed [{}]: {}", status, filename),
        Err(e) => eprintln!("❌ Error running yt-dlp: {}", e),
    }
}

fn main() {
    let matches = ClapCommand::new("m3uget")
        .version("1.0")
        .author("@OlexiyOdarchuk")
        .about("Fast multithreaded .m3u8 downloader using yt-dlp")
        .arg(
            Arg::new("source")
                .help("Either an m3u8 URL or path to .txt file with URLs")
                .required(true),
        )
        .arg(
            Arg::new("threads")
                .short('t')
                .long("threads")
                .value_name("N")
                .default_value("4")
                .help("Number of parallel downloads"),
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_parser(["auto", "base", "full"])
                .default_value("auto")
                .help("Naming mode: auto | base | full"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Suppress yt-dlp output"),
        )
        .arg(
            Arg::new("limit")
                .long("limit")
                .value_name("RATE")
                .help("Limit download speed, e.g. 5M or 800K"),
        )
        .arg(
            Arg::new("proxy")
                .long("proxy")
                .value_name("URL")
                .help("Proxy for yt-dlp, e.g. socks5://127.0.0.1:9050"),
        )
        .arg(
            Arg::new("retries")
                .long("retries")
                .value_name("N")
                .default_value("5")
                .help("Number of retry attempts"),
        )
        .get_matches();

    let source = matches.get_one::<String>("source").unwrap();
    let threads = matches
        .get_one::<String>("threads")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(4);
    let mode_str = matches.get_one::<String>("mode").unwrap();
    let quiet = matches.get_flag("quiet");
    let limit = matches.get_one::<String>("limit");
    let proxy = matches.get_one::<String>("proxy");
    let retries = matches
        .get_one::<String>("retries")
        .unwrap()
        .parse::<usize>()
        .unwrap_or(5);

    let mode = match mode_str.as_str() {
        "auto" => FileNameMode::Auto,
        "base" => FileNameMode::Base,
        "full" => FileNameMode::Full,
        _ => FileNameMode::Auto,
    };

    let urls: Vec<String> = if PathBuf::from(source).exists() {
        let file = File::open(source).expect("Could not open source file");
        BufReader::new(file)
            .lines()
            .filter_map(Result::ok)
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .collect()
    } else {
        vec![source.to_string()]
    };

    println!("🔽 Total files: {} | Threads: {}", urls.len(), threads);

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    urls.par_iter().for_each(|url| {
        let filename = generate_name(url, &mode);
        download(
            url,
            &filename,
            quiet,
            retries,
            limit.map(String::as_str),
            proxy.map(String::as_str),
        );
    });
}
