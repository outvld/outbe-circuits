use ownership_rd::{folding, groth16, measured, sha, statement::Scheme, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

const CAPACITIES: [usize; 7] = [1, 2, 4, 8, 16, 32, 64];
struct Options {
    command: String,
    out: PathBuf,
    results: Option<PathBuf>,
    result: Option<PathBuf>,
    engine: String,
    scheme: String,
    mode: String,
    capacities: Vec<usize>,
    capacity: usize,
    samples: u64,
    sample: u64,
    threads: usize,
}
impl Options {
    fn parse() -> Result<Self> {
        let mut args = std::env::args().skip(1);
        let mut o = Self {
            command: args
                .next()
                .ok_or("expected prepare, prepare-demo, run or a private worker command")?,
            out: "target/ownership-rd".into(),
            results: None,
            result: None,
            engine: "all".into(),
            scheme: "all".into(),
            mode: "both".into(),
            capacities: vec![1, 8, 64],
            capacity: 1,
            samples: 3,
            sample: 1,
            threads: std::thread::available_parallelism()?.get(),
        };
        while let Some(arg) = args.next() {
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {arg}"))?;
            match arg.as_str() {
                "--out" => o.out = value.into(),
                "--results" => o.results = Some(value.into()),
                "--result" => o.result = Some(value.into()),
                "--engine" => o.engine = value,
                "--scheme" => o.scheme = value,
                "--mode" => o.mode = value,
                "--capacity" => o.capacity = value.parse()?,
                "--capacities" => {
                    o.capacities = value
                        .split(',')
                        .map(str::parse)
                        .collect::<std::result::Result<_, _>>()?
                }
                "--samples" => o.samples = value.parse()?,
                "--sample" => o.sample = value.parse()?,
                "--threads" => o.threads = value.parse()?,
                _ => return Err(format!("unknown option {arg}").into()),
            }
        }
        if o.command == "prepare-demo" && o.out == Path::new("target/ownership-rd") {
            o.out = "target/ownership-rd-demo".into();
        }
        if o.samples == 0 || o.threads == 0 || o.capacities.is_empty() {
            return Err("samples, threads and capacities must be positive/nonempty".into());
        }
        if !CAPACITIES.contains(&o.capacity) || o.capacities.iter().any(|n| !CAPACITIES.contains(n))
        {
            return Err("capacity must be 1,2,4,8,16,32,64".into());
        }
        let mut unique = o.capacities.clone();
        unique.sort_unstable();
        unique.dedup();
        if unique.len() != o.capacities.len() {
            return Err("duplicate capacity".into());
        }
        if !["groth16", "nova", "all"].contains(&o.engine.as_str())
            || !["canonical", "poseidon2", "all"].contains(&o.scheme.as_str())
            || !["cold", "warm", "both"].contains(&o.mode.as_str())
        {
            return Err("unknown engine, signature scheme or mode".into());
        }
        Ok(o)
    }
}
fn main() {
    if let Err(e) = execute() {
        eprintln!("ownership-rd: {e}");
        std::process::exit(1);
    }
}
fn execute() -> Result<()> {
    let o = Options::parse()?;
    std::env::set_var("RAYON_NUM_THREADS", o.threads.to_string());
    match o.command.as_str() {
        "prepare" => prepare(&o),
        "prepare-demo" => prepare_demo(&o),
        "run" => run(&o),
        "__prepare" => prepare_worker(&o),
        "__run" => run_worker(&o),
        _ => Err("expected prepare, prepare-demo or run".into()),
    }
}
fn save(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn load(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn text(command: &mut Command) -> Result<String> {
    let r = command.output()?;
    if !r.status.success() {
        return Err(format!("{command:?}: {}", String::from_utf8_lossy(&r.stderr)).into());
    }
    Ok(String::from_utf8(r.stdout)?.trim().into())
}
fn hash_file(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
fn sources(dir: &Path, hashes: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if entry.file_type()?.is_dir() {
            if !["target", "results", "node_modules"]
                .contains(&entry.file_name().to_str().unwrap_or(""))
            {
                sources(&p, hashes)?;
            }
        } else if ["rs", "toml", "js", "mjs", "html", "css", "sh"]
            .contains(&p.extension().and_then(|e| e.to_str()).unwrap_or(""))
        {
            hashes.insert(p.display().to_string(), hash_file(&p)?);
        }
    }
    Ok(())
}
fn source_hashes() -> Result<BTreeMap<String, String>> {
    let mut hashes = BTreeMap::new();
    sources(Path::new("experiments/ownership-rd"), &mut hashes)?;
    sources(Path::new("crates/outbe-protocol/src"), &mut hashes)?;
    sources(Path::new("crates/outbe-zk-canonical/src"), &mut hashes)?;
    for p in [
        "Cargo.toml",
        "Cargo.lock",
        "experiments/ownership-rd/package.json",
        "experiments/ownership-rd/package-lock.json",
        "crates/outbe-protocol/Cargo.toml",
        "crates/outbe-zk-canonical/Cargo.toml",
        "crates/outbe-zk-canonical/build.rs",
        "crates/outbe-zk-canonical/circuits/manifest.toml",
        "crates/outbe-zk-canonical/resources/circuits/ownership_proof/1.0.0/abi.json",
        "experiments/groth16-tribute/src/circuit.rs",
        "experiments/groth16-tribute/src/poseidon2.rs",
        "experiments/groth16-tribute/src/poseidon2_constants.rs",
    ] {
        hashes.insert(p.into(), hash_file(Path::new(p))?);
    }
    Ok(hashes)
}
fn unused(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.into();
    }
    for n in 2.. {
        let p = PathBuf::from(format!("{}-{n}", path.display()));
        if !p.exists() {
            return p;
        }
    }
    unreachable!()
}
fn peak_rss(text: &str) -> Result<u64> {
    let mut values = text
        .lines()
        .filter_map(|l| l.trim().strip_suffix("maximum resident set size"));
    let peak: u64 = values
        .next()
        .ok_or("time output lacks peak RSS")?
        .trim()
        .parse()?;
    if peak == 0 || values.next().is_some() {
        return Err("invalid/duplicate peak RSS".into());
    }
    Ok(peak)
}
#[allow(clippy::too_many_arguments)]
fn child(
    o: &Options,
    command: &str,
    engine: &str,
    scheme: &str,
    capacity: usize,
    mode: &str,
    sample: u64,
    samples: u64,
    result: &Path,
    log: &Path,
) -> Result<Value> {
    if !cfg!(target_os = "macos") {
        return Err("native memory measurement currently requires Darwin /usr/bin/time -l".into());
    }
    let resource = log.with_extension("resources.txt");
    let file = fs::File::create(log)?;
    let start = Instant::now();
    let status = Command::new("/usr/bin/time")
        .args(["-l", "-o"])
        .arg(&resource)
        .arg(std::env::current_exe()?)
        .arg(command)
        .arg("--out")
        .arg(&o.out)
        .args([
            "--engine",
            engine,
            "--scheme",
            scheme,
            "--capacity",
            &capacity.to_string(),
            "--mode",
            mode,
            "--sample",
            &sample.to_string(),
            "--samples",
            &samples.to_string(),
            "--threads",
            &o.threads.to_string(),
        ])
        .arg("--result")
        .arg(result)
        .env("LC_ALL", "C")
        .env("RAYON_NUM_THREADS", o.threads.to_string())
        .stdout(Stdio::from(file.try_clone()?))
        .stderr(Stdio::from(file))
        .status()?;
    let wall_ns = u64::try_from(start.elapsed().as_nanos())?;
    if !status.success() {
        return Err(format!(
            "{engine}/{scheme}/n{capacity}/{mode}/{sample}: {status}; see {}",
            log.display()
        )
        .into());
    }
    let mut value = load(result)?;
    value["process"] = json!({"wall_ns":wall_ns,"peak_rss_bytes":peak_rss(&fs::read_to_string(resource)?)?,"scope":if mode=="warm"{"whole warm series, including one key load and all samples; not per-proof RSS"}else{"whole fresh worker process"}});
    save(result, &value)?;
    Ok(value)
}
fn prepare(o: &Options) -> Result<()> {
    if o.out.exists() && fs::read_dir(&o.out)?.next().is_some() {
        return Err("prepare refuses nonempty output; choose a new target directory".into());
    }
    fs::create_dir_all(&o.out)?;
    if !fs::canonicalize(&o.out)?.starts_with(fs::canonicalize("target")?) {
        return Err("large assets must stay under target/".into());
    }
    fs::create_dir_all(o.out.join("logs"))?;
    let mut manifest = json!({"format":1,"warning":"R&D only: publicly known fixture keys; single-party Groth16 setup; experimental signature unaudited; Nova uses Spartan IPA, not Groth16/EVM proofs","groth16":{"canonical":{},"poseidon2":{}},"nova":{},"preparation":[],"source_sha256":source_hashes()?,"versions":{"rustc":text(Command::new("rustc").arg("--version"))?,"nova_snark":"0.76.0","arkworks":"0.6"},"threads":o.threads});
    for scheme in ["canonical", "poseidon2"] {
        for n in CAPACITIES {
            println!("prepare groth16/{scheme}/n{n}");
            let result = o.out.join(format!("prepare-groth16-{scheme}-{n}.json"));
            let v = child(
                o,
                "__prepare",
                "groth16",
                scheme,
                n,
                "cold",
                0,
                1,
                &result,
                &o.out.join(format!("logs/prepare-groth16-{scheme}-{n}.log")),
            )?;
            manifest["groth16"][scheme][n.to_string()] = v["assets"].clone();
            manifest["preparation"].as_array_mut().unwrap().push(v);
            save(&o.out.join("preparation-progress.json"), &manifest)?;
        }
        println!("prepare nova/{scheme} one-step circuit");
        let result = o.out.join(format!("prepare-nova-{scheme}.json"));
        let v = child(
            o,
            "__prepare",
            "nova",
            scheme,
            1,
            "cold",
            0,
            1,
            &result,
            &o.out.join(format!("logs/prepare-nova-{scheme}.log")),
        )?;
        manifest["nova"][scheme] = v["assets"].clone();
        manifest["preparation"].as_array_mut().unwrap().push(v);
    }
    save(&o.out.join("manifest.json"), &manifest)?;
    println!(
        "Prepared fourteen Groth16 shapes and two reusable Nova step-parameter sets at {}",
        o.out.display()
    );
    Ok(())
}
fn prepare_demo(o: &Options) -> Result<()> {
    if o.out.exists() && fs::read_dir(&o.out)?.next().is_some() {
        return Err("prepare-demo refuses nonempty output; choose a new target directory".into());
    }
    fs::create_dir_all("target")?;
    fs::create_dir_all(&o.out)?;
    if !fs::canonicalize(&o.out)?.starts_with(fs::canonicalize("target")?) {
        return Err("large assets must stay under target/".into());
    }
    let mut manifest = json!({
        "format": 1, "circuit": "demo_tribute", "capacity": 1,
        "merkle_depth": 32, "public_input_count": 4,
        "warning": "R&D only: publicly known fixtures; unaudited experimental signature and single-party Groth16 setup; not production keys",
        "setup_randomness": "OS entropy independent of deterministic fixture randomness",
        "groth16": {}, "preparation": [], "source_sha256": source_hashes()?,
        "versions": {"rustc": text(Command::new("rustc").arg("--version"))?, "arkworks": "0.6"},
        "threads": o.threads
    });
    for name in selection(&o.scheme, &["canonical", "poseidon2"], "all") {
        println!("prepare demo_tribute/groth16/{name}");
        let keys = groth16::prepare_demo(Scheme::parse(name)?)?;
        let mut writes = Vec::new();
        let prefix = format!("keys/demo_tribute/groth16/{name}");
        let pk = asset(
            &o.out,
            &format!("{prefix}/pk.bin"),
            &keys.pk,
            &mut writes,
            "pk_write",
        )?;
        let vk = asset(
            &o.out,
            &format!("{prefix}/vk.bin"),
            &keys.vk,
            &mut writes,
            "vk_write",
        )?;
        manifest["groth16"][name] = json!({"pk": pk, "vk": vk});
        manifest["preparation"].as_array_mut().unwrap().push(json!({
            "engine": "groth16", "circuit": "demo_tribute", "scheme": name,
            "capacity": 1, "report": keys.report, "writes": writes
        }));
        save(&o.out.join("preparation-progress.json"), &manifest)?;
    }
    save(&o.out.join("manifest.json"), &manifest)?;
    println!(
        "Prepared requested Demo Tribute keys at {}",
        o.out.display()
    );
    Ok(())
}
fn asset(
    out: &Path,
    relative: &str,
    bytes: &[u8],
    timings: &mut Vec<Value>,
    stage: &str,
) -> Result<Value> {
    let path = out.join(relative);
    fs::create_dir_all(path.parent().ok_or("asset parent")?)?;
    measured(timings, stage, || {
        fs::write(&path, bytes)?;
        Ok(())
    })?;
    Ok(json!({"path":format!("assets/{relative}"),"bytes":bytes.len(),"sha256":sha(bytes)}))
}
fn prepare_worker(o: &Options) -> Result<()> {
    let scheme = Scheme::parse(&o.scheme)?;
    let mut writes = Vec::new();
    let (report, assets) = match o.engine.as_str() {
        "groth16" => {
            let keys = groth16::prepare(scheme, o.capacity)?;
            let prefix = format!("keys/groth16/{}/n{}", o.scheme, o.capacity);
            let pk = asset(
                &o.out,
                &format!("{prefix}/pk.bin"),
                &keys.pk,
                &mut writes,
                "pk_write",
            )?;
            let vk = asset(
                &o.out,
                &format!("{prefix}/vk.bin"),
                &keys.vk,
                &mut writes,
                "vk_write",
            )?;
            (keys.report, json!({"pk":pk,"vk":vk}))
        }
        "nova" => {
            let keys = folding::prepare(scheme)?;
            let params = asset(
                &o.out,
                &format!("keys/nova/{}/params.bin", o.scheme),
                &keys.params,
                &mut writes,
                "params_write",
            )?;
            (keys.report, json!({"params":params}))
        }
        _ => return Err("worker requires one engine".into()),
    };
    save(
        o.result.as_deref().ok_or("result path required")?,
        &json!({"engine":o.engine,"scheme":o.scheme,"capacity":if o.engine=="nova"{1}else{o.capacity},"sample":0,"report":report,"writes":writes,"assets":assets}),
    )
}
fn read_asset(out: &Path, asset: &Value, timings: &mut Vec<Value>, name: &str) -> Result<Vec<u8>> {
    let relative = asset["path"]
        .as_str()
        .ok_or("asset path")?
        .strip_prefix("assets/")
        .ok_or("asset prefix")?;
    if Path::new(relative)
        .components()
        .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err("unsafe asset path".into());
    }
    let bytes = measured(timings, &format!("{name}_read"), || {
        Ok(fs::read(out.join(relative))?)
    })?;
    measured(timings, &format!("{name}_sha256"), || {
        if asset["bytes"].as_u64() != Some(bytes.len() as u64)
            || asset["sha256"].as_str() != Some(sha(&bytes).as_str())
        {
            return Err(format!("asset length/hash mismatch: {relative}").into());
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn run_worker(o: &Options) -> Result<()> {
    let manifest = load(&o.out.join("manifest.json"))?;
    let scheme = Scheme::parse(&o.scheme)?;
    let mut io = Vec::new();
    let mut runs = Vec::new();
    let load_report = match o.engine.as_str() {
        "groth16" => {
            let a = &manifest["groth16"][&o.scheme][o.capacity.to_string()];
            let pk = read_asset(&o.out, &a["pk"], &mut io, "pk")?;
            let vk = read_asset(&o.out, &a["vk"], &mut io, "vk")?;
            let session = groth16::Session::load(scheme, o.capacity, &pk, &vk)?;
            drop(pk);
            drop(vk);
            for sample in o.sample..o.sample + o.samples {
                runs.push(session.run(sample)?);
            }
            session.load_report()
        }
        "nova" => {
            let a = &manifest["nova"][&o.scheme];
            let params = read_asset(&o.out, &a["params"], &mut io, "params")?;
            let session = folding::Session::load(scheme, &params)?;
            drop(params);
            for sample in o.sample..o.sample + o.samples {
                runs.push(session.run(o.capacity, sample)?);
            }
            session.load_report()
        }
        _ => return Err("worker requires one engine".into()),
    };
    if runs.iter().any(|r| r["verified"] != true) {
        return Err("unverified result".into());
    }
    save(
        o.result.as_deref().ok_or("result path required")?,
        &json!({"engine":o.engine,"scheme":o.scheme,"capacity":o.capacity,"mode":o.mode,"load_sample":if o.mode=="warm"{0}else{o.sample},"io_timings":io,"load":load_report,"runs":runs,"threads":o.threads}),
    )
}
fn selection<'a>(value: &'a str, all: &[&'a str], both: &str) -> Vec<&'a str> {
    if value == both {
        all.to_vec()
    } else {
        vec![value]
    }
}
fn run(o: &Options) -> Result<()> {
    let results = o
        .results
        .as_deref()
        .ok_or("run requires --results <new-directory>")?;
    if results.exists() {
        return Err("refusing existing results directory".into());
    }
    let manifest = load(&o.out.join("manifest.json"))?;
    let dir = unused(&o.out.join("runs"));
    fs::create_dir_all(&dir)?;
    fs::create_dir_all(results)?;
    let hw = text(Command::new("sysctl").args([
        "-n",
        "machdep.cpu.brand_string",
        "hw.memsize",
        "hw.logicalcpu",
    ]))?;
    let hw: Vec<&str> = hw.lines().collect();
    if hw.len() != 3 {
        return Err("hardware metadata missing".into());
    }
    let engines = selection(&o.engine, &["groth16", "nova"], "all");
    let schemes = selection(&o.scheme, &["canonical", "poseidon2"], "all");
    let modes = selection(&o.mode, &["cold", "warm"], "both");
    let mut meta = json!({"status":"in_progress","timestamp_utc":text(Command::new("date").args(["-u","+%Y-%m-%dT%H:%M:%SZ"]))?,"os":text(Command::new("uname").args(["-sr"]))?,"architecture":std::env::consts::ARCH,"cpu":hw[0],"ram_bytes":hw[1].parse::<u64>()?,"logical_cores":hw[2].parse::<u64>()?,"requested_threads":o.threads,"engines":engines,"schemes":schemes,"modes":modes,"capacities":o.capacities,"samples":o.samples,"manifest":manifest,"source_sha256":source_hashes()?,"diagnostics_path":dir,"timer":"std::time::Instant via web_time::Instant; individual observations only","memory":"Darwin ru_maxrss bytes for whole child; warm series peak is NOT a per-proof peak","semantics":{"cold":"one fresh process, checked key load and one proof; OS file caches unmanaged","warm":"one fresh process, one checked key load, then independent proofs on resident keys; no discarded warmups","nova":"one ownership per folding step; output commits count, ordered owner/nft pairs and binding; Spartan IPA compression, not Groth16 or deployed rollup state","signature":"Poseidon2 variant is an unaudited R&D authorization scheme; canonical signatures unchanged","fixtures":"public deterministic fixture keys; setup and proof blinding use OS entropy"}});
    save(&results.join("metadata.json"), &meta)?;
    let mut groups = Vec::new();
    for &capacity in &o.capacities {
        for &scheme in &schemes {
            for &mode in &modes {
                let iterations = if mode == "cold" { o.samples } else { 1 };
                for first_sample in 1..=iterations {
                    let mut order = engines.clone();
                    if first_sample % 2 == 0 {
                        order.reverse();
                    }
                    for engine in order {
                        println!("{engine}/{scheme}/n{capacity}/{mode}/sample{first_sample}");
                        let name = format!("{engine}-{scheme}-n{capacity}-{mode}-{first_sample}");
                        let group = match child(
                            o,
                            "__run",
                            engine,
                            scheme,
                            capacity,
                            mode,
                            first_sample,
                            if mode == "cold" { 1 } else { o.samples },
                            &dir.join(format!("{name}.json")),
                            &dir.join(format!("{name}.log")),
                        ) {
                            Ok(v) => v,
                            Err(e) => {
                                meta["status"] = json!("failed");
                                meta["error"] = json!(e.to_string());
                                save(&results.join("metadata.json"), &meta)?;
                                return Err(e);
                            }
                        };
                        groups.push(group);
                        save(&results.join("observations.json"), &json!(groups))?;
                        write_csv(&results.join("timings.csv"), &groups, &manifest)?;
                    }
                }
            }
        }
    }
    let count = groups
        .iter()
        .map(|g| g["runs"].as_array().map_or(0, Vec::len))
        .sum::<usize>();
    let expected = engines.len()
        * schemes.len()
        * modes.len()
        * o.capacities.len()
        * usize::try_from(o.samples)?;
    if count != expected {
        return Err("incomplete experiment matrix".into());
    }
    let mut public_hashes = BTreeMap::new();
    let mut witness_hashes = BTreeMap::new();
    for g in &groups {
        for r in g["runs"].as_array().ok_or("runs missing")? {
            if r["verified"] != true {
                return Err("unverified proof in dataset".into());
            }
            let key = (
                g["capacity"].as_u64().ok_or("capacity")?,
                r["sample"].as_u64().ok_or("sample")?,
            );
            if let Some(previous) = public_hashes.insert(key, r["statement_digest"].clone()) {
                if previous != r["statement_digest"] {
                    return Err("statement digest differs across engines/signatures/modes".into());
                }
            }
            let key = (g["scheme"].as_str().ok_or("scheme")?.to_owned(), key);
            if let Some(previous) = witness_hashes.insert(key, r["witness_digest"].clone()) {
                if previous != r["witness_digest"] {
                    return Err("witness digest differs across engines/modes".into());
                }
            }
        }
    }
    meta["status"] = json!("complete");
    meta["verified_runs"] = json!(count);
    meta["process_groups"] = json!(groups.len());
    save(&results.join("metadata.json"), &meta)?;
    write_report(&results.join("report.md"), &groups, &manifest)?;
    println!("Wrote {count} verified runs to {}", results.display());
    Ok(())
}
fn stages(value: &Value) -> Result<&Vec<Value>> {
    value["timings"]
        .as_array()
        .ok_or_else(|| "timings missing".into())
}
fn write_csv(path: &Path, groups: &[Value], manifest: &Value) -> Result<()> {
    use std::fmt::Write;
    let mut out = String::from("engine,scheme,capacity,mode,phase,sample,stage,elapsed_ns\n");
    for p in manifest["preparation"]
        .as_array()
        .ok_or("preparation missing")?
    {
        for row in stages(&p["report"])?
            .iter()
            .chain(p["writes"].as_array().ok_or("writes")?)
        {
            writeln!(
                out,
                "{},{},{},once,prepare,0,{},{}",
                p["engine"].as_str().ok_or("engine")?,
                p["scheme"].as_str().ok_or("scheme")?,
                p["capacity"],
                row["stage"].as_str().ok_or("stage")?,
                row["elapsed_ns"]
            )?;
        }
    }
    for g in groups {
        let prefix = format!(
            "{},{},{},{}",
            g["engine"].as_str().ok_or("engine")?,
            g["scheme"].as_str().ok_or("scheme")?,
            g["capacity"],
            g["mode"].as_str().ok_or("mode")?
        );
        for row in g["io_timings"]
            .as_array()
            .ok_or("I/O timings")?
            .iter()
            .chain(stages(&g["load"])?)
        {
            writeln!(
                out,
                "{prefix},load,{},{},{}",
                g["load_sample"],
                row["stage"].as_str().ok_or("stage")?,
                row["elapsed_ns"]
            )?;
        }
        for r in g["runs"].as_array().ok_or("runs")? {
            for row in stages(r)? {
                writeln!(
                    out,
                    "{prefix},run,{},{},{}",
                    r["sample"],
                    row["stage"].as_str().ok_or("stage")?,
                    row["elapsed_ns"]
                )?;
            }
        }
        writeln!(
            out,
            "{prefix},process,{},process_wall,{}",
            g["load_sample"], g["process"]["wall_ns"]
        )?;
    }
    fs::write(path, out)?;
    Ok(())
}
fn ms(report: &Value, stage: &str) -> Result<String> {
    Ok(stages(report)?
        .iter()
        .find(|t| t["stage"] == stage)
        .and_then(|t| t["elapsed_ns"].as_u64())
        .map_or_else(|| "N/A".into(), |n| format!("{:.3}", n as f64 / 1e6)))
}
fn write_report(path: &Path, groups: &[Value], manifest: &Value) -> Result<()> {
    use std::fmt::Write;
    let mut out=String::from("# Ownership proving R&D\n\nPublic deterministic fixtures only. The Poseidon2 signature is a new unaudited authorization protocol, not compatible with production signatures. Groth16 setup is local single-party test material. Nova uses one ownership per step and Spartan IPA compression; this is not a Groth16 final proof, EVM verifier, or a deployed stateful rollup. Production cryptographic statements, canonical artifacts and prior measurements are unchanged. Cargo features preserve native parallel math while selecting serial arkworks for browser WASM.\n\nCold: new process and checked key load per proof, with OS filesystem caches left intact. Warm: one checked load followed by independent fresh blinded proofs in the same process; no warmup is discarded. A warm process's peak RSS covers the whole series, not an individual proof. No statistical aggregates or subtracted cost estimates.\n\n## Individual core observations\n\nMilliseconds. Groth16 witness includes constrained synthesis/finalization/matrices; prove excludes repeated synthesis. Nova folding includes step synthesis and IVC work; compress is actual Spartan compression. The two stage decompositions are not equivalent. All proofs were encoded, decoded and verified against expected claims.\n\n| Engine | Signature | N | Mode | Sample | Fixture ms | Witness ms | Folding ms | Prove ms | Compress ms | Verify ms |\n|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|\n");
    for g in groups {
        for r in g["runs"].as_array().ok_or("runs")? {
            writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                g["engine"].as_str().unwrap_or(""),
                g["scheme"].as_str().unwrap_or(""),
                g["capacity"],
                g["mode"].as_str().unwrap_or(""),
                r["sample"],
                ms(r, "fixture")?,
                ms(r, "witness")?,
                ms(r, "folding")?,
                ms(r, "prove")?,
                ms(r, "compress")?,
                ms(r, "verify")?
            )?;
        }
    }
    out.push_str("\n## Key loading and process memory\n\nOne row per process. Cold load sample is the proof sample; warm load sample 0 denotes the single load shared by all proofs in that series. Native RSS includes keys, matrices/witnesses, proof work and native allocations. Browser WASM linear-memory capacity is a different metric, not RSS. Requested thread counts and hardware are in metadata.\n\n| Engine | Signature | N | Mode | Load sample | PK decode ms | VK decode ms | VK prepare ms | Params decode ms | Process wall ms | Peak RSS MiB |\n|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|\n");
    for g in groups {
        writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.3} | {:.3} |",
            g["engine"].as_str().unwrap_or(""),
            g["scheme"].as_str().unwrap_or(""),
            g["capacity"],
            g["mode"].as_str().unwrap_or(""),
            g["load_sample"],
            ms(&g["load"], "pk_decode")?,
            ms(&g["load"], "vk_decode")?,
            ms(&g["load"], "vk_prepare")?,
            ms(&g["load"], "params_decode")?,
            g["process"]["wall_ns"].as_u64().ok_or("wall")? as f64 / 1e6,
            g["process"]["peak_rss_bytes"].as_u64().ok_or("rss")? as f64 / 1048576.0
        )?;
    }
    out.push_str("\n## Sizes and circuit structure\n\nAll sizes are bytes; N/A means inapplicable, not zero. Groth16 R1CS constraints and Nova augmented primary/secondary constraints per step are different metrics. Nova parameters are for a fixed one-ownership step, independent of N. Groth16 proof payload excludes its expected public fields. Nova's serialized compressed proof already includes the final state: its 96-byte expected final-state value is NOT additional wire overhead and must not be added to proof_bytes. Nova verification also takes the initial state and step count. Neither format is claimed to be EVM calldata.\n\n| Engine | Signature | N | Mode | Sample | PK B | VK B | Params B | Proof B | Expected public/final-state B | R1CS constraints | Nova primary/step | Nova secondary/step |\n|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for g in groups {
        for r in g["runs"].as_array().ok_or("runs")? {
            write!(
                out,
                "| {} | {} | {} | {} | {} |",
                g["engine"].as_str().unwrap_or(""),
                g["scheme"].as_str().unwrap_or(""),
                g["capacity"],
                g["mode"].as_str().unwrap_or(""),
                r["sample"],
            )?;
            for key in [
                "pk_bytes",
                "vk_bytes",
                "params_bytes",
                "proof_bytes",
                "public_bytes",
                "constraints",
                "primary_constraints_per_step",
                "secondary_constraints_per_step",
            ] {
                let value = &r["sizes"][key];
                write!(
                    out,
                    " {} |",
                    if value.is_null() {
                        "N/A".into()
                    } else {
                        value.to_string()
                    }
                )?;
            }
            out.push('\n');
        }
    }
    out.push_str("\n## Preparation observations\n\nSingle-party Groth16 parameter generation and transparent Nova IPA parameter/compression-key preparation happen before samples. Each has its own fresh preparation process.\n\n| Engine | Signature | Shape capacity | Stage | Elapsed ms |\n|---|---|---:|---|---:|\n");
    for p in manifest["preparation"].as_array().ok_or("preparation")? {
        for t in stages(&p["report"])?
            .iter()
            .chain(p["writes"].as_array().ok_or("writes")?)
        {
            writeln!(
                out,
                "| {} | {} | {} | {} | {:.3} |",
                p["engine"].as_str().unwrap_or(""),
                p["scheme"].as_str().unwrap_or(""),
                p["capacity"],
                t["stage"].as_str().unwrap_or(""),
                t["elapsed_ns"].as_u64().ok_or("time")? as f64 / 1e6
            )?;
        }
    }
    out.push_str("\nEvery raw stage, including file reads, asset hashes, encoding/decoding and any recursive verification, is retained in `timings.csv`. `observations.json` retains all load reports, proofs' verification outcomes, claim/witness digests and per-process memory. Metadata includes source/lockfile hashes, tool versions, hardware and original preparation reports. Browser results must be collected on the device; native times are never substituted for them.\n");
    fs::write(path, out)?;
    Ok(())
}
