//! Isolated single-party Groth16 experiment. Never writes canonical sources/artifacts.
mod circuit;
mod emit;
mod noir;
mod poseidon2;
mod poseidon2_constants;

use ark_bn254::Bn254;
use ark_ff::UniformRand;
use ark_groth16::{prepare_verifying_key, Groth16, Proof, ProvingKey, VerifyingKey};
use ark_relations::gr1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisMode, R1CS_PREDICATE_LABEL,
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::{rngs::StdRng, SeedableRng};
use barretenberg_rs::{
    api::BarretenbergApi,
    backends::FfiBackend,
    generated_types::{CircuitInput, ProofSystemSettings},
};
use base64::Engine;
use flate2::read::GzDecoder;
use outbe_protocol::{codec::field_to_be_bytes, Fr};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
pub const CASES: [&str; 9] = [
    "demo_tribute",
    "flat_aggregation_n1",
    "flat_aggregation_n2",
    "flat_aggregation_n4",
    "flat_aggregation_n8",
    "flat_aggregation_n16",
    "flat_aggregation_n32",
    "flat_aggregation_n64",
    "emit_mint",
];
const REVISION: &str = "a9fe83ed95bc9bf6335ffac4ffe0ee983cd6f821";
const CANONICAL: &str = "crates/outbe-zk-canonical";
const SRS_POINTS: u32 = (1 << 20) + 1;

fn main() {
    if let Err(error) = execute() {
        eprintln!("groth16-tribute: {error}");
        std::process::exit(1);
    }
}

struct Options {
    command: String,
    out: PathBuf,
    results: Option<PathBuf>,
    case: Option<String>,
    backend: Option<String>,
    sample: u64,
    samples: u64,
    threads: usize,
    result: Option<PathBuf>,
}
impl Options {
    fn parse() -> Result<Self> {
        let mut args = std::env::args().skip(1);
        let mut o = Self {
            command: args.next().ok_or("expected prepare, check, or run")?,
            out: "target/groth16-tribute".into(),
            results: None,
            case: None,
            backend: None,
            sample: 1,
            samples: 3,
            threads: std::thread::available_parallelism()?.get(),
            result: None,
        };
        while let Some(arg) = args.next() {
            let val = args
                .next()
                .ok_or_else(|| format!("missing value for {arg}"))?;
            match arg.as_str() {
                "--out" => o.out = val.into(),
                "--results" => o.results = Some(val.into()),
                "--case" => o.case = Some(val),
                "--backend" => o.backend = Some(val),
                "--sample" => o.sample = val.parse()?,
                "--samples" => o.samples = val.parse()?,
                "--threads" => o.threads = val.parse()?,
                "--result" => o.result = Some(val.into()),
                _ => return Err(format!("unknown option {arg}").into()),
            }
        }
        if o.threads == 0 || o.samples == 0 {
            return Err("threads and samples must be positive".into());
        }
        if o.case
            .as_ref()
            .is_some_and(|c| !CASES.contains(&c.as_str()))
        {
            return Err("unknown circuit case".into());
        }
        Ok(o)
    }
}
fn execute() -> Result<()> {
    let o = Options::parse()?;
    std::env::set_var("RAYON_NUM_THREADS", o.threads.to_string());
    std::env::set_var("HARDWARE_CONCURRENCY", o.threads.to_string());
    std::env::set_var("BB_SLOW_LOW_MEMORY", "0");
    std::env::remove_var("BB_STORAGE_BUDGET");
    match o.command.as_str() {
        "prepare" => prepare(&o),
        "check" => check(&o),
        "run" => run(&o),
        "__setup" => setup_worker(&o),
        "__sample" => sample_worker(&o, false),
        "__proofcheck" => sample_worker(&o, true),
        _ => Err("expected prepare, check, or run".into()),
    }
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn save_json(path: &Path, v: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn load_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn gunzip(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut v = Vec::new();
    GzDecoder::new(bytes).read_to_end(&mut v)?;
    Ok(v)
}
fn output(command: &mut Command) -> Result<Vec<u8>> {
    let out = command.output()?;
    if !out.status.success() {
        return Err(format!(
            "{command:?}: {}\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(out.stdout)
}
fn text(command: &mut Command) -> Result<String> {
    Ok(String::from_utf8(output(command)?)?.trim().to_owned())
}
fn logged(command: &mut Command, path: &Path) -> Result<()> {
    let log = fs::File::create(path)?;
    let status = command
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .status()?;
    if !status.success() {
        return Err(format!(
            "{command:?} failed ({status}); diagnostic log: {}",
            path.display()
        )
        .into());
    }
    Ok(())
}
fn timed<T>(
    rows: &mut Vec<Value>,
    backend: &str,
    case: &str,
    sample: u64,
    stage: &str,
    f: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let start = Instant::now();
    let result = f()?;
    let elapsed = start.elapsed().as_nanos();
    rows.push(json!({"backend":backend,"circuit":case,"sample":sample,"stage":stage,"elapsed_ns":u64::try_from(elapsed)?}));
    Ok(result)
}
fn settings() -> ProofSystemSettings {
    ProofSystemSettings {
        ipa_accumulation: false,
        oracle_hash_type: "keccak".into(),
        disable_zk: false,
        optimized_solidity_verifier: false,
    }
}
fn srs_path() -> Result<PathBuf> {
    Ok(std::env::var_os("OUTBE_BB_SRS_PATH")
        .map(PathBuf::from)
        .unwrap_or(PathBuf::from(std::env::var("HOME")?).join(".bb-crs/bn254_g1.dat")))
}
fn init_srs() -> Result<()> {
    let path = srs_path()?;
    if fs::metadata(&path)?.len() < u64::from(SRS_POINTS) * 64 {
        return Err(format!("short local SRS: {}", path.display()).into());
    }
    outbe_zk_backend::barretenberg::set_srs_path(path);
    outbe_zk_backend::barretenberg::preinit_srs(SRS_POINTS)?;
    Ok(())
}

/// Darwin's `time -l` reports ru_maxrss in bytes, unlike Linux's KiB.
fn parse_peak_rss(text: &str) -> Result<u64> {
    let mut values = text
        .lines()
        .filter_map(|line| line.trim().strip_suffix("maximum resident set size"));
    let bytes: u64 = values
        .next()
        .ok_or("Darwin time output lacks maximum resident set size")?
        .trim()
        .parse()?;
    if bytes == 0 || values.next().is_some() {
        return Err("Darwin time output has zero or duplicate peak RSS".into());
    }
    Ok(bytes)
}

fn memory_observation(
    observation: &Value,
    backend: &str,
    circuit: &str,
    sample: u64,
) -> Result<Value> {
    let peak = observation["process"]["peak_rss_bytes"]
        .as_u64()
        .filter(|bytes| *bytes > 0)
        .ok_or("worker has no measured peak RSS")?;
    let elapsed = observation["process"]["wall_ns"]
        .as_u64()
        .ok_or("worker has no measured process wall time")?;
    Ok(
        json!({"backend":backend,"circuit":circuit,"sample":sample,"process_wall_ns":elapsed,"peak_rss_bytes":peak}),
    )
}

fn child(
    o: &Options,
    command: &str,
    case: &str,
    backend: &str,
    sample: u64,
    result: &Path,
    log: &Path,
) -> Result<()> {
    if !cfg!(target_os = "macos") {
        return Err(
            "process memory measurement requires Darwin /usr/bin/time -l (RSS bytes)".into(),
        );
    }
    let resource_log = log.with_extension("resources.txt");
    let start = Instant::now();
    logged(
        Command::new("/usr/bin/time")
            .args(["-l", "-o"])
            .arg(&resource_log)
            .arg(std::env::current_exe()?)
            .arg(command)
            .arg("--out")
            .arg(&o.out)
            .args([
                "--case",
                case,
                "--backend",
                backend,
                "--sample",
                &sample.to_string(),
                "--threads",
                &o.threads.to_string(),
            ])
            .arg("--result")
            .arg(result)
            .env("LC_ALL", "C")
            .env("RAYON_NUM_THREADS", o.threads.to_string())
            .env("HARDWARE_CONCURRENCY", o.threads.to_string())
            .env("BB_SLOW_LOW_MEMORY", "0")
            .env_remove("BB_STORAGE_BUDGET"),
        log,
    )?;
    let elapsed_ns = u64::try_from(start.elapsed().as_nanos())?;
    let peak_rss_bytes = parse_peak_rss(&fs::read_to_string(&resource_log)?)
        .map_err(|error| format!("{}: {error}", resource_log.display()))?;
    let mut observation = load_json(result)?;
    observation["process"] = json!({"wall_ns":elapsed_ns,"peak_rss_bytes":peak_rss_bytes});
    observation["timings"]
        .as_array_mut()
        .ok_or("worker timings missing")?
        .push(json!({"backend":backend,"circuit":case,"sample":sample,"stage":"process_wall","elapsed_ns":elapsed_ns}));
    save_json(result, &observation)
}
fn record_file(
    root: &Path,
    relative: &str,
    bytes: &[u8],
    hashes: &mut BTreeMap<String, String>,
) -> Result<()> {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().ok_or("missing parent")?)?;
    fs::write(&path, bytes)?;
    if fs::read(&path)? != bytes {
        return Err(format!("copy mismatch: {}", path.display()).into());
    }
    hashes.insert(relative.to_string(), sha(bytes));
    Ok(())
}
fn source_snapshot(
    from: &Path,
    to: &Path,
    relative: &Path,
    root: &Path,
    hashes: &mut BTreeMap<String, String>,
) -> Result<()> {
    for item in fs::read_dir(from.join(relative))? {
        let item = item?;
        let rel = relative.join(item.file_name());
        if item.file_type()?.is_dir() {
            if item.file_name() == "src" || relative.starts_with("src") {
                source_snapshot(from, to, &rel, root, hashes)?;
            }
        } else if relative.starts_with("src") || item.file_name() == "Nargo.toml" {
            let dst = to.join(&rel);
            let name = dst
                .strip_prefix(root)?
                .to_str()
                .ok_or("non-UTF8 source path")?;
            record_file(root, name, &fs::read(item.path())?, hashes)?;
        }
    }
    Ok(())
}
fn prepare(o: &Options) -> Result<()> {
    if o.out.exists() && fs::read_dir(&o.out)?.next().is_some() {
        return Err(format!(
            "refusing nonempty preparation directory {}",
            o.out.display()
        )
        .into());
    }
    fs::create_dir_all(&o.out)?;
    let root = fs::canonicalize(&o.out)?;
    let target = fs::canonicalize("target")?;
    if !root.starts_with(&target) {
        return Err("large experimental assets must live under target/".into());
    }
    fs::create_dir_all(root.join("logs"))?;
    let versions = versions()?;
    if !versions["nargo"]
        .as_str()
        .unwrap_or("")
        .contains("nargo version = 1.0.0-beta.22")
        || versions["bb"] != "5.0.0-nightly.20260522"
    {
        return Err(format!("pinned tool mismatch: {versions}").into());
    }
    let mut hashes = BTreeMap::new();
    let mut timings = Vec::new();
    let mut memory = Vec::new();
    let tree = text(Command::new("git").args([
        "ls-tree",
        "-r",
        "--name-only",
        REVISION,
        &format!("{CANONICAL}/noir/outbe-circuit-core"),
    ]))?;
    let mut paths: Vec<String> = tree
        .lines()
        .filter(|p| p.ends_with("Nargo.toml") || p.contains("/src/"))
        .map(str::to_owned)
        .collect();
    if paths.is_empty() {
        return Err("historical core sources unavailable".into());
    }
    for case in CASES
        .iter()
        .filter(|case| case.starts_with("flat_aggregation_n"))
    {
        let n = case
            .strip_prefix("flat_aggregation_n")
            .ok_or("bad capacity")?;
        for suffix in ["Nargo.toml", "src/main.nr"] {
            paths.push(format!(
                "{CANONICAL}/noir/outbe-flat-aggregation-circuit-n{n}/{suffix}"
            ));
        }
        for artifact in ["abi.json", "bytecode.b64", "circuit.vk"] {
            let path = format!("{CANONICAL}/resources/circuits/{case}/1.0.0/{artifact}");
            let bytes = output(Command::new("git").args(["show", &format!("{REVISION}:{path}")]))?;
            record_file(
                &root,
                &format!("artifacts/{case}/{artifact}"),
                &bytes,
                &mut hashes,
            )?;
        }
    }
    for path in paths {
        let bytes = output(Command::new("git").args(["show", &format!("{REVISION}:{path}")]))?;
        record_file(
            &root,
            &format!(
                "historical/{}",
                path.strip_prefix(&format!("{CANONICAL}/"))
                    .ok_or("historical prefix")?
            ),
            &bytes,
            &mut hashes,
        )?;
    }
    for rel in [
        "noir/outbe-circuit-core",
        "l2/57005/tribute",
        "noir/emit_mint",
    ] {
        source_snapshot(
            &Path::new(CANONICAL).join(rel),
            &root.join("current").join(rel),
            Path::new(""),
            &root,
            &mut hashes,
        )?;
    }
    let emit_version = outbe_zk_canonical::noir::emit_mint::VERSION;
    let emit_artifacts = format!("resources/circuits/emit_mint/{emit_version}");
    for (case, relative) in [
        ("demo_tribute", "l2/57005/tribute"),
        ("emit_mint", emit_artifacts.as_str()),
    ] {
        for artifact in ["abi.json", "bytecode.b64", "circuit.vk"] {
            record_file(
                &root,
                &format!("artifacts/{case}/{artifact}"),
                &fs::read(Path::new(CANONICAL).join(relative).join(artifact))?,
                &mut hashes,
            )?;
        }
    }
    // Record provenance before expensive tools, preserving diagnostics on failure.
    save_json(
        &root.join("provenance.json"),
        &json!({"historical_revision":REVISION,"current_l1_versions":{"emit_mint":emit_version},"sha256":hashes,"versions":versions}),
    )?;
    let mut stats = BTreeMap::new();
    for case in CASES {
        println!("prepare {case}: reproduce sources and VK");
        let package = if case == "demo_tribute" {
            root.join("current/l2/57005/tribute")
        } else if case == "emit_mint" {
            root.join("current/noir/emit_mint")
        } else {
            root.join(format!(
                "historical/noir/outbe-flat-aggregation-circuit-n{}",
                case.strip_prefix("flat_aggregation_n").ok_or("capacity")?
            ))
        };
        timed(
            &mut timings,
            "noir_bb",
            case,
            0,
            "compile_subprocess",
            || {
                logged(
                    Command::new("nargo").arg("compile").current_dir(&package),
                    &root.join(format!("logs/{case}-compile.log")),
                )
            },
        )?;
        let compiled_path = package.join(format!("target/{case}.json"));
        let compiled = load_json(&compiled_path)?;
        let artifacts = root.join("artifacts").join(case);
        if case != "demo_tribute" {
            if compiled["abi"] != load_json(&artifacts.join("abi.json"))? {
                return Err(
                    format!("frozen ABI mismatch: {case}: {}", compiled_path.display()).into(),
                );
            }
            let expected = fs::read_to_string(artifacts.join("bytecode.b64"))?;
            let actual = compiled["bytecode"]
                .as_str()
                .ok_or("compiled bytecode missing")?;
            if base64::engine::general_purpose::STANDARD.decode(expected.trim())?
                != base64::engine::general_purpose::STANDARD.decode(actual)?
            {
                return Err(
                    format!("frozen ACIR mismatch: {case}: {}", compiled_path.display()).into(),
                );
            }
        }
        let vk_dir = root.join("derived-vk").join(case);
        fs::create_dir_all(&vk_dir)?;
        timed(
            &mut timings,
            "noir_bb",
            case,
            0,
            "vk_derive_subprocess",
            || {
                logged(
                    Command::new("bb")
                        .arg("write_vk")
                        .arg("-b")
                        .arg(&compiled_path)
                        .arg("-o")
                        .arg(&vk_dir)
                        .args(["-t", "evm-no-zk"]),
                    &root.join(format!("logs/{case}-vk.log")),
                )
            },
        )?;
        if fs::read(vk_dir.join("vk"))? != fs::read(artifacts.join("circuit.vk"))? {
            return Err(format!("VK mismatch: {case}: {}", vk_dir.display()).into());
        }
        let raw = gunzip(
            &base64::engine::general_purpose::STANDARD
                .decode(fs::read_to_string(artifacts.join("bytecode.b64"))?.trim())?,
        )?;
        let mut api = BarretenbergApi::new(FfiBackend::new()?);
        let st = timed(&mut timings, "noir_bb", case, 0, "circuit_stats", || {
            Ok(api.circuit_stats(
                CircuitInput {
                    name: case.into(),
                    bytecode: raw,
                    verification_key: Vec::new(),
                },
                false,
                settings(),
            )?)
        })?;
        if st.num_gates_dyadic + 1 > SRS_POINTS {
            return Err(format!(
                "{case} needs {} SRS points, staged prefix is {SRS_POINTS}",
                st.num_gates_dyadic + 1
            )
            .into());
        }
        stats.insert(
            case,
            json!({"gates":st.num_gates,"dyadic_size":st.num_gates_dyadic}),
        );
        save_json(
            &root.join("preparation-progress.json"),
            &json!({"timings":timings,"stats":stats}),
        )?;
    }
    for case in CASES {
        println!("prepare {case}: single-party Groth16 setup");
        let result = root.join(format!("setup-{case}.json"));
        child(
            o,
            "__setup",
            case,
            "groth16",
            0,
            &result,
            &root.join(format!("logs/{case}-setup.log")),
        )?;
        let setup = load_json(&result)?;
        memory.push(memory_observation(&setup, "groth16", case, 0)?);
        timings.extend(
            setup["timings"]
                .as_array()
                .ok_or("setup timings missing")?
                .iter()
                .cloned(),
        );
        for file in ["pk.bin", "vk.bin"] {
            let relative = format!("keys/{case}/{file}");
            hashes.insert(relative.clone(), sha(&fs::read(root.join(relative))?));
        }
    }
    save_json(
        &root.join("preparation.json"),
        &json!({"historical_revision":REVISION,"current_l1_versions":{"emit_mint":emit_version},"sha256":hashes,"versions":versions,"timings":timings,"memory":memory,"stats":stats,"threads":o.threads,"single_party_test_setup":true}),
    )?;
    println!("Prepared all {} shapes at {}", CASES.len(), root.display());
    Ok(())
}
fn versions() -> Result<Value> {
    Ok(
        json!({"rustc":text(Command::new("rustc").arg("--version"))?,"nargo":text(Command::new("nargo").arg("--version"))?,"bb":text(Command::new("bb").arg("--version"))?}),
    )
}

fn setup_worker(o: &Options) -> Result<()> {
    let case = o.case.as_deref().ok_or("setup case missing")?;
    let fixture = noir::fixture(case, 0)?;
    let mut rng = StdRng::from_entropy();
    let mut timings = Vec::new();
    let pk = timed(&mut timings, "groth16", case, 0, "setup", || {
        Ok(Groth16::<Bn254>::generate_random_parameters_with_reduction(
            fixture, &mut rng,
        )?)
    })?;
    let dir = o.out.join("keys").join(case);
    fs::create_dir_all(&dir)?;
    timed(&mut timings, "groth16", case, 0, "pk_write", || {
        let mut bytes = Vec::new();
        pk.serialize_compressed(&mut bytes)?;
        fs::write(dir.join("pk.bin"), bytes)?;
        Ok(())
    })?;
    timed(&mut timings, "groth16", case, 0, "vk_write", || {
        let mut bytes = Vec::new();
        pk.vk.serialize_compressed(&mut bytes)?;
        fs::write(dir.join("vk.bin"), bytes)?;
        Ok(())
    })?;
    save_json(
        o.result.as_deref().ok_or("setup result missing")?,
        &json!({"timings":timings}),
    )
}

fn sample_worker(o: &Options, proof_checks: bool) -> Result<()> {
    let case = o.case.as_deref().ok_or("sample case missing")?;
    let backend = o.backend.as_deref().ok_or("sample backend missing")?;
    let mut timings = Vec::new();
    let fixture = timed(&mut timings, backend, case, o.sample, "fixture", || {
        noir::fixture(case, o.sample)
    })?;
    let fields = noir::input_fields(&fixture);
    let public = noir::public_inputs(&fixture);
    let fixture_bytes: Vec<u8> = fields.iter().flat_map(field_to_be_bytes).collect();
    let public_bytes: Vec<u8> = public.iter().flat_map(field_to_be_bytes).collect();
    let sizes = match backend {
        "groth16" => groth16_sample(o, fixture, &public, &mut timings, proof_checks)?,
        "noir_bb" => bb_sample(o, &fields, &public, &mut timings, proof_checks)?,
        _ => return Err("unknown backend".into()),
    };
    save_json(
        o.result.as_deref().ok_or("sample result missing")?,
        &json!({"backend":backend,"circuit":case,"sample":o.sample,"fixture_sha256":sha(&fixture_bytes),"public_sha256":sha(&public_bytes),"timings":timings,"sizes":sizes,"verified":true,"proof_checks":proof_checks,"threads":o.threads}),
    )
}

fn groth16_sample(
    o: &Options,
    fixture: noir::Fixture,
    public: &[Fr],
    timings: &mut Vec<Value>,
    checks: bool,
) -> Result<Value> {
    let case = o.case.as_deref().ok_or("case missing")?;
    let key_dir = o.out.join("keys").join(case);
    let pk_bytes = timed(timings, "groth16", case, o.sample, "pk_read", || {
        Ok(fs::read(key_dir.join("pk.bin"))?)
    })?;
    let pk_size = pk_bytes.len();
    let pk = timed(timings, "groth16", case, o.sample, "pk_decode", || {
        Ok(ProvingKey::<Bn254>::deserialize_compressed(
            pk_bytes.as_slice(),
        )?)
    })?;
    drop(pk_bytes);
    let vk_bytes = timed(timings, "groth16", case, o.sample, "vk_read", || {
        Ok(fs::read(key_dir.join("vk.bin"))?)
    })?;
    let vk_size = vk_bytes.len();
    let vk = timed(timings, "groth16", case, o.sample, "vk_decode", || {
        Ok(VerifyingKey::<Bn254>::deserialize_compressed(
            vk_bytes.as_slice(),
        )?)
    })?;
    let pvk = timed(timings, "groth16", case, o.sample, "vk_prepare", || {
        Ok(prepare_verifying_key(&vk))
    })?;
    let (matrices, assignment, num_inputs, num_constraints, num_witness) =
        timed(timings, "groth16", case, o.sample, "witness", || {
            let cs = ConstraintSystem::new_ref();
            cs.set_optimization_goal(OptimizationGoal::Constraints);
            cs.set_mode(SynthesisMode::Prove {
                construct_matrices: true,
                generate_lc_assignments: false,
            });
            fixture.generate_constraints(cs.clone())?;
            cs.finalize();
            let matrices = cs
                .to_matrices()?
                .remove(R1CS_PREDICATE_LABEL)
                .ok_or("missing R1CS matrices")?;
            let inner = cs.borrow().ok_or("no constraint system")?;
            let num_constraints = inner
                .get_predicate_num_constraints(R1CS_PREDICATE_LABEL)
                .ok_or("no R1CS constraints")?;
            let mut assignment =
                Vec::with_capacity(inner.num_instance_variables + inner.num_witness_variables);
            assignment.extend_from_slice(inner.instance_assignment()?);
            assignment.extend_from_slice(inner.witness_assignment()?);
            Ok((
                matrices,
                assignment,
                inner.num_instance_variables,
                num_constraints,
                inner.num_witness_variables,
            ))
        })?;
    if assignment[1..num_inputs] != *public {
        return Err("Groth16 public assignment mismatch".into());
    }
    let mut rng = StdRng::from_entropy();
    let proof = timed(timings, "groth16", case, o.sample, "prove", || {
        Ok(Groth16::<Bn254>::create_proof_with_reduction_and_matrices(
            &pk,
            Fr::rand(&mut rng),
            Fr::rand(&mut rng),
            &matrices,
            num_inputs,
            num_constraints,
            &assignment,
        )?)
    })?;
    let encoded = timed(timings, "groth16", case, o.sample, "proof_encode", || {
        let mut bytes = Vec::new();
        proof.serialize_compressed(&mut bytes)?;
        Ok(bytes)
    })?;
    let decoded = timed(timings, "groth16", case, o.sample, "proof_decode", || {
        Ok(Proof::<Bn254>::deserialize_compressed(encoded.as_slice())?)
    })?;
    let verified = timed(timings, "groth16", case, o.sample, "verify", || {
        Ok(Groth16::<Bn254>::verify_proof(&pvk, &decoded, public)?)
    })?;
    if !verified {
        return Err(format!("Groth16 verification false: {case}/{}", o.sample).into());
    }
    if checks {
        for index in 0..public.len() {
            let mut bad = public.to_vec();
            bad[index] += Fr::from(1u64);
            if Groth16::<Bn254>::verify_proof(&pvk, &decoded, &bad)? {
                return Err(format!("Groth16 accepted changed public {index}").into());
            }
        }
        if Groth16::<Bn254>::verify_proof(&pvk, &decoded, &public[..public.len() - 1])
            .unwrap_or(false)
        {
            return Err("Groth16 accepted wrong public arity".into());
        }
        if Proof::<Bn254>::deserialize_compressed(&encoded[..encoded.len() - 1]).is_ok() {
            return Err("Groth16 accepted truncated encoding".into());
        }
        let mut changed = encoded.clone();
        changed[0] ^= 1;
        if let Ok(bad) = Proof::<Bn254>::deserialize_compressed(changed.as_slice()) {
            if Groth16::<Bn254>::verify_proof(&pvk, &bad, public).unwrap_or(false) {
                return Err("Groth16 accepted modified proof".into());
            }
        }
        if case == "flat_aggregation_n1" {
            let wrong = Groth16::<Bn254>::generate_random_parameters_with_reduction(
                noir::fixture(case, o.sample)?,
                &mut rng,
            )?;
            if Groth16::<Bn254>::verify_proof(&prepare_verifying_key(&wrong.vk), &decoded, public)?
            {
                return Err("Groth16 accepted wrong same-shape VK".into());
            }
        }
    }
    Ok(
        json!({"proof_bytes":encoded.len(),"public_bytes":public.len()*32,"pk_bytes":pk_size,"vk_bytes":vk_size,"witness_msgpack_bytes":null,"assignment_fields":assignment.len(),"assignment_fixed_bytes":assignment.len()*32,"constraints":num_constraints,"instance_variables":num_inputs,"witness_variables":num_witness,"gates":null,"dyadic_size":null}),
    )
}

fn decode_bb_proof(bytes: &[u8]) -> Result<Vec<Vec<u8>>> {
    if bytes.is_empty() || bytes.len() % 32 != 0 {
        return Err("bb proof length is not a nonempty multiple of 32".into());
    }
    Ok(bytes.chunks_exact(32).map(<[u8]>::to_vec).collect())
}
fn bb_sample(
    o: &Options,
    fields: &[Fr],
    public: &[Fr],
    timings: &mut Vec<Value>,
    checks: bool,
) -> Result<Value> {
    let case = o.case.as_deref().ok_or("case missing")?;
    let dir = o.out.join("artifacts").join(case);
    let artifact = timed(timings, "noir_bb", case, o.sample, "artifact_read", || {
        Ok(fs::read_to_string(dir.join("bytecode.b64"))?)
    })?;
    let compressed = timed(
        timings,
        "noir_bb",
        case,
        o.sample,
        "artifact_base64",
        || Ok(base64::engine::general_purpose::STANDARD.decode(artifact.trim())?),
    )?;
    let program = timed(timings, "noir_bb", case, o.sample, "program_decode", || {
        Ok(acir::circuit::Program::<acir::FieldElement>::deserialize_program(&compressed)?)
    })?;
    let raw = timed(timings, "noir_bb", case, o.sample, "acir_gunzip", || {
        gunzip(&compressed)
    })?;
    let vk = timed(timings, "noir_bb", case, o.sample, "vk_read", || {
        Ok(fs::read(dir.join("circuit.vk"))?)
    })?;
    let solved = timed(timings, "noir_bb", case, o.sample, "witness", || {
        noir::solve(&program, fields, public)
    })?;
    let witness = timed(timings, "noir_bb", case, o.sample, "witness_encode", || {
        noir::encode_witness(&solved)
    })?;
    let mut api = timed(timings, "noir_bb", case, o.sample, "backend_init", || {
        Ok(BarretenbergApi::new(FfiBackend::new()?))
    })?;
    timed(timings, "noir_bb", case, o.sample, "srs_init", init_srs)?;
    let response = timed(timings, "noir_bb", case, o.sample, "prove", || {
        Ok(api.circuit_prove(
            CircuitInput {
                name: case.into(),
                bytecode: raw,
                verification_key: vk.clone(),
            },
            &witness,
            settings(),
        )?)
    })?;
    let encoded = timed(timings, "noir_bb", case, o.sample, "proof_encode", || {
        if response.proof.iter().any(|f| f.len() != 32) {
            return Err("bb proof has non-32-byte field".into());
        }
        Ok(response.proof.concat())
    })?;
    let decoded = timed(timings, "noir_bb", case, o.sample, "proof_decode", || {
        decode_bb_proof(&encoded)
    })?;
    let verified = timed(timings, "noir_bb", case, o.sample, "verify", || {
        Ok(api
            .circuit_verify(
                &vk,
                public
                    .iter()
                    .map(|f| field_to_be_bytes(f).to_vec())
                    .collect(),
                decoded.clone(),
                settings(),
            )?
            .verified)
    })?;
    if !verified {
        return Err(format!("bb verification false: {case}/{}", o.sample).into());
    }
    if checks {
        for index in 0..public.len() {
            let mut bad = public.to_vec();
            bad[index] += Fr::from(1u64);
            if api
                .circuit_verify(
                    &vk,
                    bad.iter().map(|f| field_to_be_bytes(f).to_vec()).collect(),
                    decoded.clone(),
                    settings(),
                )
                .map(|r| r.verified)
                .unwrap_or(false)
            {
                return Err(format!("bb accepted changed public {index}").into());
            }
        }
        if api
            .circuit_verify(
                &vk,
                public[..public.len() - 1]
                    .iter()
                    .map(|f| field_to_be_bytes(f).to_vec())
                    .collect(),
                decoded.clone(),
                settings(),
            )
            .map(|r| r.verified)
            .unwrap_or(false)
        {
            return Err("bb accepted wrong public arity".into());
        }
        if decode_bb_proof(&encoded[..encoded.len() - 1]).is_ok() {
            return Err("bb accepted truncated encoding".into());
        }
        let mut bad = encoded.clone();
        bad[0] ^= 1;
        if api
            .circuit_verify(
                &vk,
                public
                    .iter()
                    .map(|f| field_to_be_bytes(f).to_vec())
                    .collect(),
                decode_bb_proof(&bad)?,
                settings(),
            )
            .map(|r| r.verified)
            .unwrap_or(false)
        {
            return Err("bb accepted modified proof".into());
        }
    }
    let prep = load_json(&o.out.join("preparation.json"))?;
    Ok(
        json!({"proof_bytes":encoded.len(),"public_bytes":public.len()*32,"pk_bytes":null,"vk_bytes":vk.len(),"witness_msgpack_bytes":witness.len(),"assignment_fields":null,"assignment_fixed_bytes":null,"constraints":null,"instance_variables":null,"witness_variables":null,"gates":prep["stats"][case]["gates"],"dyadic_size":prep["stats"][case]["dyadic_size"]}),
    )
}

fn verify_preparation(out: &Path) -> Result<Value> {
    let prep = load_json(&out.join("preparation.json"))?;
    if prep["historical_revision"] != REVISION {
        return Err("preparation revision mismatch".into());
    }
    for (relative, expected) in prep["sha256"]
        .as_object()
        .ok_or("preparation hashes missing")?
    {
        if hash_file(&out.join(relative))? != expected.as_str().ok_or("invalid hash")? {
            return Err(format!("prepared file SHA-256 mismatch: {relative}").into());
        }
    }
    Ok(prep)
}
fn hash_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn unused(base: &Path) -> PathBuf {
    if !base.exists() {
        return base.to_owned();
    }
    for i in 2.. {
        let next = PathBuf::from(format!("{}-{i}", base.display()));
        if !next.exists() {
            return next;
        }
    }
    unreachable!()
}
fn check(o: &Options) -> Result<()> {
    verify_preparation(&o.out)?;
    noir::check_constraints(&o.out)?;
    let dir = unused(&o.out.join("check"));
    fs::create_dir_all(&dir)?;
    for case in [
        "demo_tribute",
        "flat_aggregation_n2",
        "flat_aggregation_n1",
        "emit_mint",
    ] {
        for backend in ["groth16", "noir_bb"] {
            if case == "flat_aggregation_n1" && backend == "noir_bb" {
                continue;
            }
            println!("proof checks {case}/{backend}");
            child(
                o,
                "__proofcheck",
                case,
                backend,
                1,
                &dir.join(format!("{case}-{backend}.json")),
                &dir.join(format!("{case}-{backend}.log")),
            )?;
        }
    }
    save_json(
        &dir.join("success.json"),
        &json!({"differential_constraints":true,"proof_checks":true}),
    )?;
    println!(
        "All {} statement parity checks and Groth16/bb proof attacks passed.",
        CASES.len()
    );
    Ok(())
}
fn run(o: &Options) -> Result<()> {
    let results = o
        .results
        .as_deref()
        .ok_or("run requires --results <new-directory>")?;
    if results.exists() {
        return Err(format!("refusing existing results directory {}", results.display()).into());
    }
    let prep = verify_preparation(&o.out)?;
    fs::create_dir_all(results)?;
    let diagnostics = unused(&o.out.join("runs"));
    fs::create_dir_all(&diagnostics)?;
    let cases: Vec<&str> = o
        .case
        .as_deref()
        .map_or_else(|| CASES.to_vec(), |c| vec![c]);
    let mut observations = Vec::new();
    let mut timings = prep["timings"]
        .as_array()
        .ok_or("preparation timings missing")?
        .clone();
    let mut memory = prep["memory"]
        .as_array()
        .ok_or("preparation lacks memory measurements; rerun prepare in a new target directory")?
        .clone();
    let mut metadata = metadata(o, &prep)?;
    metadata["diagnostics_path"] = json!(diagnostics);
    metadata["status"] = json!("in_progress");
    save_json(&results.join("metadata.json"), &metadata)?;
    for case in &cases {
        for sample in 1..=o.samples {
            let order = if sample % 2 == 1 {
                ["groth16", "noir_bb"]
            } else {
                ["noir_bb", "groth16"]
            };
            let mut fixture_hash: Option<Value> = None;
            let mut public_hash: Option<Value> = None;
            for backend in order {
                println!("sample {case}/{backend}/{sample}");
                let result = diagnostics.join(format!("{case}-{backend}-{sample}.json"));
                let log = diagnostics.join(format!("{case}-{backend}-{sample}.log"));
                if let Err(error) = child(o, "__sample", case, backend, sample, &result, &log) {
                    metadata["status"] = json!("failed");
                    metadata["error"] = json!(error.to_string());
                    save_json(&results.join("metadata.json"), &metadata)?;
                    return Err(error);
                }
                let observation = load_json(&result)?;
                memory.push(memory_observation(&observation, backend, case, sample)?);
                if observation["verified"] != true {
                    return Err(format!("unverified worker: {case}/{backend}/{sample}").into());
                }
                if fixture_hash
                    .as_ref()
                    .is_some_and(|h| *h != observation["fixture_sha256"])
                    || public_hash
                        .as_ref()
                        .is_some_and(|h| *h != observation["public_sha256"])
                {
                    return Err(format!(
                        "cross-backend fixture/public digest mismatch: {case}/{sample}"
                    )
                    .into());
                }
                fixture_hash = Some(observation["fixture_sha256"].clone());
                public_hash = Some(observation["public_sha256"].clone());
                timings.extend(
                    observation["timings"]
                        .as_array()
                        .ok_or("worker timings missing")?
                        .iter()
                        .cloned(),
                );
                observations.push(observation);
                write_csv(&results.join("timings.csv"), &timings)?;
                write_memory_csv(&results.join("memory.csv"), &memory)?;
                save_json(&results.join("observations.json"), &json!(observations))?;
            }
        }
    }
    if observations.len() != cases.len() * 2 * usize::try_from(o.samples)? {
        return Err("incomplete observation matrix".into());
    }
    for observation in &observations {
        for stage in ["witness", "prove", "verify"] {
            if observation["timings"]
                .as_array()
                .ok_or("timings missing")?
                .iter()
                .filter(|r| r["stage"] == stage)
                .count()
                != 1
            {
                return Err(format!("missing/duplicate {stage} observation").into());
            }
        }
    }
    metadata["status"] = json!("complete");
    metadata["core_rows"] = json!(observations.len());
    save_json(&results.join("metadata.json"), &metadata)?;
    write_report(&results.join("report.md"), &observations, &timings, &memory)?;
    println!(
        "Wrote {} verified core rows to {}",
        observations.len(),
        results.display()
    );
    Ok(())
}
fn write_csv(path: &Path, rows: &[Value]) -> Result<()> {
    use std::fmt::Write;
    let mut s = String::from("backend,circuit,sample,stage,elapsed_ns\n");
    for r in rows {
        writeln!(
            s,
            "{},{},{},{},{}",
            r["backend"].as_str().ok_or("backend")?,
            r["circuit"].as_str().ok_or("case")?,
            r["sample"],
            r["stage"].as_str().ok_or("stage")?,
            r["elapsed_ns"]
        )?;
    }
    fs::write(path, s)?;
    Ok(())
}

fn write_memory_csv(path: &Path, rows: &[Value]) -> Result<()> {
    use std::fmt::Write;
    let mut csv = String::from("backend,circuit,sample,process_wall_ns,peak_rss_bytes\n");
    for row in rows {
        writeln!(
            csv,
            "{},{},{},{},{}",
            row["backend"].as_str().ok_or("memory backend missing")?,
            row["circuit"].as_str().ok_or("memory circuit missing")?,
            row["sample"],
            row["process_wall_ns"],
            row["peak_rss_bytes"],
        )?;
    }
    fs::write(path, csv)?;
    Ok(())
}

fn milliseconds(row: &Value) -> Result<String> {
    let ns = row["elapsed_ns"].as_u64().ok_or("nanoseconds missing")?;
    Ok(format!("{:.3}", ns as f64 / 1_000_000.0))
}
fn stage_ms(observation: &Value, stage: &str) -> Result<String> {
    match observation["timings"]
        .as_array()
        .ok_or("timings")?
        .iter()
        .find(|r| r["stage"] == stage)
    {
        Some(r) => milliseconds(r),
        None => Ok("N/A".into()),
    }
}
fn write_report(
    path: &Path,
    observations: &[Value],
    timings: &[Value],
    memory: &[Value],
) -> Result<()> {
    use std::fmt::Write;
    let mut s=String::from("# Groth16 Tribute measurements\n\nIndividual wall-clock observations, not a statistical benchmark. Fresh process per backend/case/sample; OS filesystem caches left intact. Verification occurs after proving, not in a standalone cold verifier. All proofs use fresh OS-entropy-seeded blinding. Groth16 setup is local single-party experimental setup, never production ceremony material. Production circuits and backend are unchanged.\n\n## Core timings\n\nTimes in milliseconds. Witness means constrained synthesis/finalization/matrix materialization for Groth16 and initial-map construction/ACVM solving/finalization for Noir. Groth16 proving excludes repeated synthesis. N/A is inapplicable, never an unmeasured zero.\n\n| Circuit | Backend | Sample | Witness ms | Witness encode ms | Prove ms | Verify ms |\n|---|---|---:|---:|---:|---:|---:|\n");
    for o in observations {
        writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} |",
            o["circuit"].as_str().ok_or("case")?,
            o["backend"].as_str().ok_or("backend")?,
            o["sample"],
            stage_ms(o, "witness")?,
            stage_ms(o, "witness_encode")?,
            stage_ms(o, "prove")?,
            stage_ms(o, "verify")?
        )?;
    }
    if observations.iter().any(|o| o["circuit"] == "emit_mint") {
        writeln!(
            s,
            "\nEmit rows use frozen `emit_mint@{}` and partial-mint fixtures with wide amounts, cross-limb borrowing, and a real index-5 note inclusion path. Full mints and amount/address boundaries are checked separately. The port mirrors the pinned `noir-bignum v0.10.0` statement, including its `[120,120,17]`-bit ABI limb bounds and subtraction-flag constraints; it does not silently add a stricter uint256 gate or repair the production circuit.",
            outbe_zk_canonical::noir::emit_mint::VERSION,
        )?;
    }
    s.push_str("\n## Technical timings\n\nOne row per observation, in milliseconds. Sample 0 is one-time preparation (one setup per shape), not a per-proof sample. Compilation and VK derivation include subprocess overhead. Groth16 PK/VK writes include compressed serialization plus disk write; checked key decoding is separate from disk reads. bb has no stored proving key or separate VK preparation API: PK read/decode and VK decode/prepare are N/A; native VK deserialization/preprocessing stays inside FFI verify. `srs_init` loads/checksums and initializes the maximum 1,048,577-point prefix, not a circuit-minimal SRS. Requested thread counts are in metadata, not a claim of measured effective utilization.\n\n| Circuit | Backend | Sample | Stage | Elapsed ms |\n|---|---|---:|---|---:|\n");
    for r in timings {
        if [
            "witness",
            "witness_encode",
            "prove",
            "verify",
            "process_wall",
        ]
        .contains(&r["stage"].as_str().unwrap_or(""))
        {
            continue;
        }
        writeln!(
            s,
            "| {} | {} | {} | {} | {} |",
            r["circuit"].as_str().ok_or("case")?,
            r["backend"].as_str().ok_or("backend")?,
            r["sample"],
            r["stage"].as_str().ok_or("stage")?,
            milliseconds(r)?
        )?;
    }
    s.push_str("\n## Process totals and memory\n\nMeasured by Darwin `/usr/bin/time -l` for each fresh worker process; RSS is bytes from the OS, displayed as MiB (1 MiB = 1,048,576 bytes). This is whole-process peak resident memory, including native bb allocations, keys, SRS, witness/matrices and prover/verifier—not serialized sizes, virtual memory, allocator-only usage, or a per-stage peak. It does not identify which phase caused the peak. Sample 0 covers Groth16 setup plus key serialization/writes; samples 1+ cover fixture creation through proof verification and worker exit. Compiler and CLI VK-generation memory is not measured. Process wall time is independently measured with Instant around the parent invocation, including process/timing-wrapper startup, log setup and shutdown; it is not a sum or subtraction of stage timings. The resource collector runs outside the worker's named stage timers. Raw bytes and process times are in `memory.csv`; original OS resource logs are retained under the diagnostics path in metadata.\n\n| Circuit | Backend | Sample | Process wall ms | Peak RSS MiB |\n|---|---|---:|---:|---:|\n");
    for row in memory {
        writeln!(
            s,
            "| {} | {} | {} | {:.3} | {:.3} |",
            row["circuit"].as_str().ok_or("memory circuit missing")?,
            row["backend"].as_str().ok_or("memory backend missing")?,
            row["sample"],
            row["process_wall_ns"]
                .as_u64()
                .ok_or("process wall missing")? as f64
                / 1_000_000.0,
            row["peak_rss_bytes"].as_u64().ok_or("peak RSS missing")? as f64 / 1_048_576.0,
        )?;
    }
    s.push_str("\n## Sizes and structural counts\n\nAll sizes are bytes. Proof payload excludes public inputs (32 bytes each). Groth16 proof/key encodings are checked compressed arkworks canonical encodings, not EVM calldata. bb proofs are concatenated 32-byte fields and do not include a second copy of public inputs. Groth16 assignment footprint is 32 bytes per field including constant-one/public slots, not a persisted witness-file format. Noir witness size is raw msgpack. R1CS constraints/variables and bb gates/dyadic sizes are different metrics, not equivalent work units.\n\n| Circuit | Backend | Sample | Proof B | Public B | PK B | VK B | Witness msgpack B | Assignment fields | Assignment B | R1CS constraints | Instance vars incl. one | Witness vars | bb gates | bb dyadic |\n|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for o in observations {
        write!(
            s,
            "| {} | {} | {} |",
            o["circuit"].as_str().ok_or("case")?,
            o["backend"].as_str().ok_or("backend")?,
            o["sample"]
        )?;
        for key in [
            "proof_bytes",
            "public_bytes",
            "pk_bytes",
            "vk_bytes",
            "witness_msgpack_bytes",
            "assignment_fields",
            "assignment_fixed_bytes",
            "constraints",
            "instance_variables",
            "witness_variables",
            "gates",
            "dyadic_size",
        ] {
            let v = &o["sizes"][key];
            write!(
                s,
                " {} |",
                if v.is_null() {
                    "N/A".into()
                } else {
                    v.to_string()
                }
            )?;
        }
        s.push('\n');
    }
    s.push_str("\nExact nanoseconds are in `timings.csv`; full per-sample fixture/public digests and numeric sizes are in `observations.json`. Hardware, pins, encodings, settings, provenance hashes and stage definitions are in `metadata.json`. No averages, selected best runs, or subtracted cost estimates are reported.\n");
    fs::write(path, s)?;
    Ok(())
}
fn source_hashes(path: &Path, hashes: &mut BTreeMap<String, String>) -> Result<()> {
    for item in fs::read_dir(path)? {
        let item = item?;
        let p = item.path();
        if item.file_type()?.is_dir() {
            if !["target", "results", ".git"].contains(&item.file_name().to_str().unwrap_or("")) {
                source_hashes(&p, hashes)?;
            }
        } else if ["rs", "toml", "nr"]
            .contains(&p.extension().and_then(|s| s.to_str()).unwrap_or(""))
        {
            hashes.insert(p.display().to_string(), hash_file(&p)?);
        }
    }
    Ok(())
}
fn metadata(o: &Options, preparation: &Value) -> Result<Value> {
    let mut sources = BTreeMap::new();
    for path in ["crates", "experiments/groth16-tribute/src", "xtask/src"] {
        source_hashes(Path::new(path), &mut sources)?;
    }
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "experiments/groth16-tribute/Cargo.toml",
        "mise.toml",
    ] {
        sources.insert(path.into(), hash_file(Path::new(path))?);
    }
    let hardware = text(Command::new("sysctl").args([
        "-n",
        "machdep.cpu.brand_string",
        "hw.memsize",
        "hw.logicalcpu",
    ]))?;
    let hardware: Vec<&str> = hardware.lines().collect();
    if hardware.len() != 3 {
        return Err("unexpected sysctl hardware result".into());
    }
    Ok(json!({
        "measured_timestamp_utc":text(Command::new("date").args(["-u","+%Y-%m-%dT%H:%M:%SZ"]))?,
        "os":text(Command::new("uname").args(["-sr"]))?,"architecture":std::env::consts::ARCH,
        "cpu_model":hardware[0],"ram_bytes":hardware[1].parse::<u64>()?,"logical_cores":hardware[2].parse::<u64>()?,
        "requested_threads":o.threads,"thread_environment":["RAYON_NUM_THREADS","HARDWARE_CONCURRENCY"],"effective_native_threads":"not measured",
        "versions":versions()?,"profile":{"name":"release","opt_level":3,"overflow_checks":false,"debug":false},
        "source_sha256":sources,"preparation":preparation,"historical_revision":REVISION,
        "fixture_seed":"0x7472696275746500 ^ ((case_index as u64) << 32) ^ sample_number; Demo index 0, ascending flat capacities indices 1..7, Emit index 8",
        "emit_fixture":"partial mint with a wide amount, cross-limb borrowing and a real index-5 inclusion path among eight notes; canonical native Emit hashes; full mint and frozen limb-range boundaries additionally checked",
        "emit_statement":{"version":outbe_zk_canonical::noir::emit_mint::VERSION,"limb_bits":[120,120,17],"public_words":8,"arithmetic":"mirrors pinned noir-bignum v0.10.0 integer ordering and subtraction-flag constraints; honest ordered subtraction assigns underflow=false"},
        "setup_and_proof_rng":"StdRng::from_entropy (OS entropy); deterministic RNG only for public fixtures",
        "srs":{"path":srs_path()?,"points":SRS_POINTS,"policy":"maximum prefix for all shapes, local only; backend SHA-256 pin enforced"},
        "settings":{"ipa_accumulation":false,"oracle_hash_type":"keccak","disable_zk":false,"optimized_solidity_verifier":false,"BB_SLOW_LOW_MEMORY":"0","BB_STORAGE_BUDGET":"unset"},
        "sampling":"one fresh sequential child per case/backend/sample; alternate backend order; no discarded warmup; OS filesystem caches unmanaged; verification after proving",
        "memory_measurement":{"source":"Darwin /usr/bin/time -l; ru_maxrss in bytes","scope":"entire fresh worker process, not per-stage; Groth16 setup includes key writes; samples include fixture, loading, proving, verification and worker exit","excluded":"parent process, compile subprocess and CLI VK derivation memory","display_unit":"MiB = 1048576 bytes","process_wall":"parent Instant around invocation including timing wrapper, log setup, process startup and shutdown; excludes post-exit resource parsing","raw_logs":"*.resources.txt under diagnostics_path; setup logs under prepared-directory/logs","statistics":"individual process peaks, not averages across runs"},
        "stage_definitions":stage_definitions(),
        "encodings":{"public_inputs":"canonical 32-byte big-endian fields","fixture_sha256":"entire flattened Noir ABI vector as canonical 32-byte big-endian fields","groth16_proof_keys":"arkworks checked compressed canonical, not EVM calldata","bb_proof":"concatenated 32-byte proof fields, excludes publics","bb_witness":"raw msgpack","groth16_assignment":"field count and 32-byte fixed footprint including constant one and publics, not serialized"}
    }))
}

fn stage_definitions() -> Value {
    json!({
            "fixture":"shared deterministic circuit witnesses; native ownership signatures for Tribute/flat and native note/change hashes for Emit; actual inserted-leaf Merkle paths for Demo/Emit",
            "compile_subprocess":"pinned nargo compile including subprocess overhead",
            "vk_derive_subprocess":"pinned bb write_vk -t evm-no-zk including subprocess overhead",
            "circuit_stats":"native bb circuit_stats including FFI packing",
            "setup":"Groth16 random parameters, synthesis and QAP setup, once per shape, single-party experimental",
            "pk_write":"Groth16 compressed PK serialization and disk write","vk_write":"Groth16 compressed VK serialization and disk write",
            "pk_read":"stored compressed Groth16 PK disk read; bb N/A","pk_decode":"checked compressed PK decoding; bb N/A",
            "vk_read":"stored VK disk read","vk_decode":"checked Groth16 compressed decoding; bb included in verify",
            "vk_prepare":"Groth16 prepare_verifying_key; bb N/A, native preprocessing included in verify",
            "artifact_read":"committed/recovered bytecode.b64 disk read","artifact_base64":"bytecode base64 decode",
            "program_decode":"ACIR Program deserialize","acir_gunzip":"compressed ACIR to raw bb msgpack",
            "witness":"Groth16 constrained synthesis, finalize, matrices and full assignment; Noir initial map, ACVM solve/finalize and public parity",
            "witness_encode":"Noir WitnessStack serialize then gunzip to raw msgpack; Groth16 N/A",
            "backend_init":"FfiBackend and API initialization","srs_init":"maximum-prefix local SRS loader/checksum/native initialization",
            "prove":"Groth16 fresh blinding plus matrix QAP/FFT/MSMs; bb circuit_prove including required FFI packing",
            "process_wall":"parent-observed worker invocation wall time including /usr/bin/time wrapper, startup, log setup and shutdown; shown with process memory, not core timing",
            "proof_encode":"Groth16 compressed canonical serialize; bb concatenate 32-byte fields",
            "proof_decode":"Groth16 checked compressed deserialize; bb reconstruct 32-byte vectors (native validation in verify)",
            "verify":"decoded fresh proof against expected public vector; bb includes FFI packing/native VK preprocessing"
    })
}

#[cfg(test)]
mod process_memory_tests {
    use super::parse_peak_rss;

    #[test]
    fn reads_resident_bytes_not_footprint_or_page_counts() {
        let output = "0.10 real 0.08 user 0.01 sys\n\
            134217728  maximum resident set size\n\
            124  page faults\n\
            140000000  peak memory footprint\n";
        assert_eq!(parse_peak_rss(output).unwrap(), 134_217_728);
    }

    #[test]
    fn missing_or_ambiguous_memory_measurements_fail() {
        for output in [
            "",
            "42 peak memory footprint\n",
            "bad maximum resident set size\n",
            "-1 maximum resident set size\n",
            "0 maximum resident set size\n",
            "18446744073709551616 maximum resident set size\n",
            "42 maximum resident set size\n84 maximum resident set size\n",
        ] {
            assert!(parse_peak_rss(output).is_err(), "{output}");
        }
    }
}
