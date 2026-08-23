//! RngKit desktop application backend.

/// Reachable `rngkit-core` git revision pinned by this application.
pub const RNGKIT_CORE_REVISION: &str = "3f327e9e88679c26683323f116cd6d7b3ea64fff";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running RngKit");
}

#[cfg(test)]
mod tests {
    use super::RNGKIT_CORE_REVISION;

    #[test]
    fn pins_gate_a_library_revision() {
        assert_eq!(
            RNGKIT_CORE_REVISION,
            "3f327e9e88679c26683323f116cd6d7b3ea64fff"
        );
    }

    #[test]
    fn linked_library_crates_compile_without_hardware() {
        let bits = rngkit_core::SampleBits::new(8).expect("sample bits");
        assert_eq!(bits.get(), 8);
        let _ = std::any::type_name::<rngkit_analysis::Accumulator>();
        let _ = std::any::type_name::<rngkit_sources::SourceConfig>();
        let _ = std::any::type_name::<rngkit_engine::EngineConfig>();
        let _ = rngkit_recording::CONCATENATION_KIND;
        let _ = rngkit_xlsx::REF_PLUS;
    }
}
