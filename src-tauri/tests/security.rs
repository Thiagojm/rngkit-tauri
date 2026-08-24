//! Path containment, open-target, capability, and redaction hardening.
//! Default tests do not enumerate or open hardware.

use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rngkit_lib::collection::{CollectionHandle, VecSender};
use rngkit_lib::coordinator::{AppCoordinator, pseudo_candidate};
use rngkit_lib::dto::ErrorCode;
use rngkit_lib::reports::{
    ReportsHandle, generate_inspected, inspect_derived, inspect_native, inspect_picked,
    write_native_report,
};

const CAPABILITIES: &str = include_str!("../capabilities/default.json");
const TAURI_CONF: &str = include_str!("../tauri.conf.json");
const LIB_RS: &str = include_str!("../src/lib.rs");
const OPEN_SESSION: &str = include_str!("../src/commands/collection.rs");
const OPEN_REPORTS: &str = include_str!("../src/commands/reports.rs");
const OPEN_COMBINE: &str = include_str!("../src/commands/combine.rs");
const REPORTS_IMPL: &str = include_str!("../src/reports/mod.rs");
const CI_WORKFLOW: &str = include_str!("../../.github/workflows/ci.yml");

struct TempRoot(PathBuf);

impl Deref for TempRoot {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_root() -> TempRoot {
    let dir = std::env::temp_dir().join(format!(
        "rngkit-security-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("temp");
    TempRoot(dir)
}

fn assert_safe_json(dump: &str) {
    let lower = dump.to_ascii_lowercase();
    assert!(!dump.contains(":\\"), "{dump}");
    assert!(!dump.contains("/dev/"), "{dump}");
    assert!(!lower.contains("entropy byte"), "{dump}");
    assert!(!lower.contains("seed="), "{dump}");
    assert!(!lower.contains("serial="), "{dump}");
}

fn completed_session() -> (TempRoot, PathBuf) {
    let root = temp_root();
    let mut coordinator = AppCoordinator::new();
    let generation = coordinator.begin_discover().expect("discover");
    coordinator
        .complete_discover(generation, vec![pseudo_candidate()])
        .expect("results");
    coordinator.set_output_root(&root).expect("root");
    coordinator.select_token("mock-pseudo-1").expect("select");
    let coordinator = Mutex::new(coordinator);
    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    CollectionHandle::fake().run_blocking(&coordinator, &VecSender::new(), plan);
    let directory = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_directory()
        .expect("dir")
        .to_path_buf();
    (root, directory)
}

fn try_file_symlink(target: &Path, link: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).expect("unix file symlink");
        true
    }
    #[cfg(windows)]
    {
        match std::os::windows::fs::symlink_file(target, link) {
            Ok(()) => true,
            Err(err) => {
                eprintln!("UNVERIFIED file-symlink coverage: {err}");
                false
            }
        }
    }
}

fn try_dir_reparse(target: &Path, link: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).expect("unix dir symlink");
        true
    }
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &link.to_string_lossy(),
                &target.to_string_lossy(),
            ])
            .status();
        matches!(status, Ok(s) if s.success())
    }
}

#[test]
fn capabilities_are_core_and_dialog_only() {
    let value: serde_json::Value = serde_json::from_str(CAPABILITIES).expect("json");
    assert_eq!(
        value["permissions"],
        serde_json::json!(["core:default", "dialog:default"])
    );
    assert!(!CAPABILITIES.contains("fs:"), "{CAPABILITIES}");
    assert!(!CAPABILITIES.contains("shell:"), "{CAPABILITIES}");
    assert!(!CAPABILITIES.contains("opener:"), "{CAPABILITIES}");
    assert!(!CAPABILITIES.contains("logging:"), "{CAPABILITIES}");
}

#[test]
fn production_csp_is_restricted_and_debug_command_is_cfg_gated() {
    let conf: serde_json::Value = serde_json::from_str(TAURI_CONF).expect("json");
    let csp = &conf["app"]["security"]["csp"];
    assert!(csp.is_object(), "{csp}");
    assert!(
        csp["default-src"]
            .as_str()
            .is_some_and(|v| v.contains("'self'")),
        "{csp}"
    );
    assert!(
        csp["connect-src"]
            .as_str()
            .is_some_and(|v| v.contains("ipc:")),
        "{csp}"
    );
    assert!(
        !csp["connect-src"]
            .as_str()
            .is_some_and(|v| v.contains("localhost:1420") || v.contains("ws://")),
        "production CSP must not allow development endpoints: {csp}"
    );
    let dev_csp = &conf["app"]["security"]["devCsp"];
    assert!(
        dev_csp["connect-src"]
            .as_str()
            .is_some_and(|v| v.contains("localhost:1420")),
        "{dev_csp}"
    );
    assert_eq!(conf["app"]["windows"][0]["minWidth"], 800);
    assert_eq!(conf["app"]["windows"][0]["minHeight"], 600);

    let release = LIB_RS
        .split("#[cfg(not(debug_assertions))]")
        .nth(1)
        .expect("release handler");
    assert!(
        !release.contains("apply_dev_scenario"),
        "release handler must omit debug scenarios"
    );
    assert!(LIB_RS.contains("commands::dev::apply_dev_scenario"));
}

#[test]
fn open_commands_accept_no_frontend_path() {
    for source in [OPEN_SESSION, OPEN_REPORTS, OPEN_COMBINE] {
        for name in [
            "open_session_folder",
            "open_report",
            "open_report_folder",
            "open_derived_folder",
        ] {
            if !source.contains(&format!("pub fn {name}")) {
                continue;
            }
            let start = source.find(&format!("pub fn {name}")).expect(name);
            let sig = &source[start..];
            let end = sig.find('{').expect("body");
            let header = &sig[..end];
            assert!(
                !header.contains("Path") && !header.contains("path:"),
                "{name} accepts a frontend path: {header}"
            );
        }
    }
    assert!(
        !REPORTS_IMPL.contains("Command::new(\"cmd\")"),
        "artifact opening must not invoke a command interpreter"
    );
}

#[test]
fn open_targets_are_backend_known_only() {
    let (root, directory) = completed_session();
    let empty = AppCoordinator::new();
    let reports = ReportsHandle::fake();
    let collection = CollectionHandle::fake();

    assert_eq!(
        reports
            .open_known_report(&empty)
            .expect_err("no report")
            .code,
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        reports
            .open_known_folder(&empty)
            .expect_err("no folder")
            .code,
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        collection
            .open_known_session_folder(&empty)
            .expect_err("no session")
            .code,
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        reports
            .open_existing_folder(root.join("missing").as_path())
            .expect_err("missing")
            .code,
        ErrorCode::InvalidTransition
    );

    let mut coordinator = AppCoordinator::new();
    inspect_picked(&mut coordinator, &directory).expect("inspect");
    generate_inspected(&mut coordinator, false).expect("xlsx");
    reports
        .open_known_report(&coordinator)
        .expect("open report");
    reports
        .open_known_folder(&coordinator)
        .expect("open folder");
    let dump = serde_json::to_string(&coordinator.snapshot()).expect("json");
    assert_safe_json(&dump);
}

#[test]
fn native_symlink_artifacts_are_rejected_and_do_not_escape() {
    let (root, directory) = completed_session();
    let native = rngkit_recording::NativeSession::open(&directory).expect("native");
    let stem = native.session_stem().as_str().to_owned();
    drop(native);

    let outside = root.join("secret.bin");
    fs::write(&outside, b"do-not-read").expect("secret");
    let csv = directory.join(format!("{stem}.csv"));
    fs::remove_file(&csv).expect("remove csv");
    if !try_file_symlink(&outside, &csv) {
        fs::write(&csv, b"restored").ok();
        return;
    }
    let error = inspect_native(&directory, None).expect_err("symlink csv");
    assert!(
        matches!(
            error.code,
            ErrorCode::UnsupportedInput | ErrorCode::CorruptInput
        ),
        "{error:?}"
    );
    assert_safe_json(&serde_json::to_string(&error).expect("json"));
    assert_eq!(fs::read(&outside).expect("unchanged"), b"do-not-read");
}

#[test]
fn directory_reparse_to_an_outside_session_is_not_followed_as_contained_dest() {
    let (root, directory) = completed_session();
    let decoy = root.join("decoy");
    fs::create_dir_all(&decoy).expect("decoy");
    let link = root.join("linked-session");
    if !try_dir_reparse(&directory, &link) {
        eprintln!("UNVERIFIED directory reparse coverage");
        return;
    }
    let inspected = inspect_native(&link, None);
    match inspected {
        Ok(ok) => {
            let dest = ok.dest;
            let canonical_dir = fs::canonicalize(&directory).unwrap_or(directory.clone());
            let canonical_dest = fs::canonicalize(&dest).unwrap_or(dest.clone());
            assert!(
                canonical_dest.starts_with(&canonical_dir),
                "dest {} escaped {}",
                canonical_dest.display(),
                canonical_dir.display()
            );
        }
        Err(error) => {
            assert!(
                matches!(
                    error.code,
                    ErrorCode::UnsupportedInput | ErrorCode::CorruptInput
                ),
                "{error:?}"
            );
            assert_safe_json(&serde_json::to_string(&error).expect("json"));
        }
    }
}

#[test]
fn malformed_derived_manifest_fails_without_partial_xlsx() {
    let root = temp_root();
    let dir = root.join("20260822T120000_concat_trng_s16_i1");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(dir.join("manifest.json"), b"{not-json").expect("junk");
    let error = inspect_derived(&dir).expect_err("malformed");
    assert!(
        matches!(
            error.code,
            ErrorCode::CorruptInput | ErrorCode::UnsupportedInput
        ),
        "{error:?}"
    );
    assert_safe_json(&serde_json::to_string(&error).expect("json"));
    let xlsx: Vec<_> = fs::read_dir(&dir)
        .expect("list")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "xlsx"))
        .collect();
    assert!(xlsx.is_empty(), "partial xlsx {:?}", xlsx);
}

#[test]
fn ci_is_locked_and_skips_hardware_and_installer() {
    assert!(CI_WORKFLOW.contains("npm ci"), "{CI_WORKFLOW}");
    assert!(CI_WORKFLOW.contains("--locked"), "{CI_WORKFLOW}");
    assert!(CI_WORKFLOW.contains("windows-latest"), "{CI_WORKFLOW}");
    assert!(CI_WORKFLOW.contains("ubuntu-22.04"), "{CI_WORKFLOW}");
    assert!(
        CI_WORKFLOW.contains("libwebkit2gtk-4.1-dev"),
        "{CI_WORKFLOW}"
    );
    assert!(
        CI_WORKFLOW.contains("build --no-bundle -- --locked"),
        "{CI_WORKFLOW}"
    );
    assert!(!CI_WORKFLOW.contains("--ignored"), "{CI_WORKFLOW}");
    assert!(!CI_WORKFLOW.contains("--bundles"), "{CI_WORKFLOW}");
}

#[test]
fn nsis_bundle_is_unsigned_per_user_english_offline() {
    let conf: serde_json::Value = serde_json::from_str(TAURI_CONF).expect("json");
    let identifier = conf["identifier"].as_str().expect("identifier");
    assert_eq!(identifier, "com.rngkit.desktop");
    assert!(
        !identifier.ends_with(".app"),
        "identifier {identifier} ends with .app"
    );
    let bundle = &conf["bundle"];
    assert_eq!(bundle["targets"], serde_json::json!(["nsis"]));
    assert_eq!(bundle["createUpdaterArtifacts"], false);
    let windows = &bundle["windows"];
    assert_eq!(
        windows["webviewInstallMode"]["type"].as_str(),
        Some("offlineInstaller")
    );
    assert_eq!(windows["nsis"]["installMode"].as_str(), Some("currentUser"));
    assert_eq!(windows["nsis"]["languages"], serde_json::json!(["English"]));
    assert_ne!(windows["nsis"]["displayLanguageSelector"], true);
    assert!(windows["certificateThumbprint"].is_null());
    assert!(windows["digestAlgorithm"].is_null());
    assert!(windows["signCommand"].is_null());
    assert!(windows["timestampUrl"].is_null());
}

#[test]
fn existing_native_xlsx_is_not_overwritten_without_replace() {
    let (root, directory) = completed_session();
    write_native_report(&directory, false).expect("first");
    let native = rngkit_recording::NativeSession::open(&directory).expect("native");
    let dest =
        rngkit_xlsx::native_report_path(native.directory(), native.session_stem()).expect("dest");
    drop(native);
    let original = fs::read(&dest).expect("bytes");
    let error = write_native_report(&directory, false).expect_err("exists");
    assert_eq!(error.code, ErrorCode::OutputExists);
    assert_eq!(fs::read(&dest).expect("kept"), original);
    let _ = root;
}
