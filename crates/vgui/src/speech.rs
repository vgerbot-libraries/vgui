//! Speech synthesis backed by the platform's real text-to-speech engine.
//!
//! - **Native desktop:** a system TTS program is resolved from `PATH`
//!   ([`SpeechSynthesizer::new`]): Speech Dispatcher (`spd-say`), eSpeak
//!   (`espeak`/`espeak-ng`) or `flite` on Linux, `say` on macOS, and the
//!   SAPI `System.Speech` synthesizer through PowerShell on Windows. Spoken
//!   audio is produced by spawning that program with the text as an argument.
//! - **Web (`wasm`):** the browser [Web Speech API]
//!   (`window.speechSynthesis` + `SpeechSynthesisUtterance`) is called
//!   directly.
//!
//! Availability is detected rather than assumed: [`SpeechSynthesizer::new`]
//! yields a synthesizer whose [`available`](SpeechSynthesizer::available) is
//! `false` (and whose [`speak`](SpeechSynthesizer::speak) returns
//! [`SpeechError::Unsupported`]) when no engine exists. Nothing fakes audio.
//!
//! [Web Speech API]: https://developer.mozilla.org/docs/Web/API/SpeechSynthesis
//!
//! Only synthesis is provided; speech recognition is out of scope.

#[cfg(not(target_family = "wasm"))]
use std::path::PathBuf;

/// Tunable speech parameters. Every field is optional; `None` leaves the
/// engine default. Relative fields (`rate`, `pitch`, `volume`) use `1.0` as
/// the normal value with a sensible range of roughly `0.5..=2.0`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpeechOptions {
    /// Speaking rate multiplier (`1.0` normal).
    pub rate: Option<f32>,
    /// Pitch multiplier (`1.0` normal).
    pub pitch: Option<f32>,
    /// Volume multiplier (`1.0` normal, clamped to the engine range).
    pub volume: Option<f32>,
    /// Voice identifier; interpretation depends on the engine (e.g. a macOS
    /// `say` voice name). Ignored by engines without voice selection.
    pub voice: Option<String>,
}

impl SpeechOptions {
    /// Create options with every field left at the engine default.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the rate multiplier.
    pub fn rate(mut self, rate: f32) -> Self {
        self.rate = Some(rate);
        self
    }

    /// Set the pitch multiplier.
    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = Some(pitch);
        self
    }

    /// Set the volume multiplier.
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume);
        self
    }

    /// Set the voice identifier.
    pub fn voice(mut self, voice: impl Into<String>) -> Self {
        self.voice = Some(voice.into());
        self
    }
}

/// Failure mode for an attempted utterance.
#[derive(Debug)]
#[non_exhaustive]
pub enum SpeechError {
    /// No platform TTS engine is available.
    Unsupported,
    /// The engine program exists but could not be spawned.
    Spawn(String),
}

impl std::fmt::Display for SpeechError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpeechError::Unsupported => write!(f, "no speech synthesis engine available"),
            SpeechError::Spawn(e) => write!(f, "failed to spawn speech engine: {e}"),
        }
    }
}

impl std::error::Error for SpeechError {}

/// A concrete native TTS program and its argument conventions.
#[cfg(not(target_family = "wasm"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Engine {
    /// Speech Dispatcher client (`spd-say`).
    SpdSay,
    /// eSpeak / eSpeak NG.
    Espeak,
    /// CMU Flite (`flite -t`).
    Flite,
    /// macOS `say`.
    Say,
    /// Windows SAPI driven through `powershell`.
    PowerShell,
}

#[cfg(not(target_family = "wasm"))]
impl Engine {
    fn program(self) -> &'static str {
        match self {
            Engine::SpdSay => "spd-say",
            Engine::Espeak => "espeak",
            Engine::Flite => "flite",
            Engine::Say => "say",
            Engine::PowerShell => "powershell.exe",
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn clamp(value: f32, lo: f32, hi: f32) -> f32 {
    value.clamp(lo, hi)
}

/// Build the argument vector for `engine` to speak `text` with `opts`.
/// The program itself is not included (it is the spawned executable).
#[cfg(not(target_family = "wasm"))]
fn build_argv(engine: Engine, text: &str, opts: &SpeechOptions) -> Vec<std::ffi::OsString> {
    let mut args: Vec<std::ffi::OsString> = Vec::new();
    match engine {
        Engine::SpdSay => {
            // spd-say: -r rate, -p pitch, -i volume, each -100..100 (0 normal).
            if let Some(r) = opts.rate {
                args.push("-r".into());
                args.push(format!("{}", clamp((r - 1.0) * 100.0, -100., 100.).round() as i8).into());
            }
            if let Some(p) = opts.pitch {
                args.push("-p".into());
                args.push(format!("{}", clamp((p - 1.0) * 100.0, -100., 100.).round() as i8).into());
            }
            if let Some(v) = opts.volume {
                args.push("-i".into());
                args.push(format!("{}", clamp((v - 1.0) * 100.0, -100., 100.).round() as i8).into());
            }
            if let Some(voice) = &opts.voice {
                args.push("-v".into());
                args.push(voice.into());
            }
            args.push("--wait".into());
            args.push(text.into());
        }
        Engine::Espeak => {
            // espeak: -s wpm, -p pitch 0..99 (50 normal), -a amplitude 0..200.
            if let Some(r) = opts.rate {
                args.push("-s".into());
                args.push(format!("{}", clamp(175.0 * r, 1., 600.).round() as u16).into());
            }
            if let Some(p) = opts.pitch {
                args.push("-p".into());
                args.push(format!("{}", clamp(50.0 * p, 0., 99.).round() as u8).into());
            }
            if let Some(v) = opts.volume {
                args.push("-a".into());
                args.push(format!("{}", clamp(100.0 * v, 0., 200.).round() as u16).into());
            }
            if let Some(voice) = &opts.voice {
                args.push("-v".into());
                args.push(voice.into());
            }
            args.push(text.into());
        }
        Engine::Flite => {
            // flite has no runtime rate/pitch/voice flags; text via -t.
            args.push("-t".into());
            args.push(text.into());
        }
        Engine::Say => {
            if let Some(voice) = &opts.voice {
                args.push("-v".into());
                args.push(voice.into());
            }
            if let Some(r) = opts.rate {
                args.push("-r".into());
                args.push(format!("{}", clamp(175.0 * r, 1., 720.).round() as u16).into());
            }
            if let Some(s) = opts.volume {
                // say has no volume flag; there is nothing honest to pass.
                let _ = s;
            }
            args.push(text.into());
        }
        Engine::PowerShell => {
            // SAPI Rate is -10..10 (0 normal), Volume 0..100 (100 normal).
            let mut script = String::from(
                "Add-Type -AssemblyName System.Speech; \
                 $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; ",
            );
            if let Some(r) = opts.rate {
                script.push_str(&format!(
                    "$s.Rate = {}; ",
                    clamp((r - 1.0) * 10.0, -10., 10.).round() as i8
                ));
            }
            if let Some(v) = opts.volume {
                script.push_str(&format!(
                    "$s.Volume = {}; ",
                    clamp(100.0 * v, 0., 100.).round() as u8
                ));
            }
            // Single quotes are escaped by doubling for PowerShell.
            let safe = text.replace('\'', "''");
            script.push_str(&format!("$s.Speak('{safe}');"));
            args.push("-NoProfile".into());
            args.push("-Command".into());
            args.push(script.into());
        }
    }
    args
}

#[cfg(not(target_family = "wasm"))]
fn candidates() -> &'static [Engine] {
    if cfg!(target_os = "macos") {
        &[Engine::Say]
    } else if cfg!(target_os = "windows") {
        &[Engine::PowerShell]
    } else {
        &[Engine::SpdSay, Engine::Espeak, Engine::Flite]
    }
}

/// Whether `program` resolves to an executable file on `PATH`.
#[cfg(not(target_family = "wasm"))]
fn resolve_in_path(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(program);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Pick the first available candidate. `has` decides program presence so the
/// decision is testable without mutating the real `PATH`.
#[cfg(not(target_family = "wasm"))]
fn select_engine(has: &impl Fn(&str) -> bool) -> Option<(Engine, PathBuf)> {
    for engine in candidates() {
        if has(engine.program()) {
            return Some((*engine, PathBuf::from(engine.program())));
        }
    }
    None
}

/// A handle to the platform speech synthesizer.
pub struct SpeechSynthesizer {
    #[cfg(not(target_family = "wasm"))]
    backend: Option<(Engine, PathBuf)>,
    #[cfg(not(target_family = "wasm"))]
    child: std::sync::Mutex<Option<std::process::Child>>,
}

impl Default for SpeechSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_family = "wasm"))]
fn detect() -> Option<(Engine, PathBuf)> {
    select_engine(&|p| resolve_in_path(p).is_some())
}

impl SpeechSynthesizer {
    /// Detect the best available platform TTS engine.
    pub fn new() -> Self {
        #[cfg(not(target_family = "wasm"))]
        {
            Self {
                backend: detect(),
                child: std::sync::Mutex::new(None),
            }
        }
        #[cfg(target_family = "wasm")]
        {
            Self {}
        }
    }

    /// Build a native synthesizer with no detected backend. Test-only: lets
    /// the unsupported path be exercised without depending on host audio.
    #[cfg(all(not(target_family = "wasm"), any(test, feature = "test-support")))]
    pub fn unavailable_for_test() -> Self {
        Self {
            backend: None,
            child: std::sync::Mutex::new(None),
        }
    }

    /// Whether a real backend will service [`speak`](Self::speak).
    pub fn available(&self) -> bool {
        #[cfg(not(target_family = "wasm"))]
        {
            self.backend.is_some()
        }
        #[cfg(target_family = "wasm")]
        {
            web::synth().is_some()
        }
    }

    /// Speak `text` with `options`. Returns [`SpeechError::Unsupported`] when
    /// no engine is present; never reports success for missing audio.
    pub fn speak(&self, text: impl AsRef<str>, options: &SpeechOptions) -> Result<(), SpeechError> {
        let text = text.as_ref();
        if text.is_empty() {
            return Ok(());
        }
        #[cfg(not(target_family = "wasm"))]
        {
            let (engine, exe) = self.backend.as_ref().ok_or(SpeechError::Unsupported)?;
            let args = build_argv(*engine, text, options);
            let mut command = std::process::Command::new(exe);
            command.args(args);
            // Detach stdio so a daemonized TTS server never blocks on pipes.
            command.stdin(std::process::Stdio::null());
            command.stdout(std::process::Stdio::null());
            command.stderr(std::process::Stdio::null());
            let child = command
                .spawn()
                .map_err(|e| SpeechError::Spawn(e.to_string()))?;
            *self.child.lock().expect("speech child lock") = Some(child);
            Ok(())
        }
        #[cfg(target_family = "wasm")]
        {
            web::speak(text, options)
        }
    }

    /// Stop any in-flight utterance.
    pub fn stop(&self) {
        #[cfg(not(target_family = "wasm"))]
        {
            if let Some(mut child) = self.child.lock().expect("speech child lock").take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        #[cfg(target_family = "wasm")]
        {
            web::cancel();
        }
    }
}

impl Drop for SpeechSynthesizer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Convenience constructor mirroring other component entry points.
pub fn speech() -> SpeechSynthesizer {
    SpeechSynthesizer::new()
}

#[cfg(target_family = "wasm")]
mod web {
    use super::{SpeechError, SpeechOptions};
    use js_sys::Array;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        SpeechSynthesis, SpeechSynthesisUtterance, SpeechSynthesisVoice, Window,
    };

    fn window() -> Option<Window> {
        web_sys::window()
    }

    pub fn synth() -> Option<SpeechSynthesis> {
        window()?.speech_synthesis().ok()
    }

    fn js_err(e: JsValue) -> SpeechError {
        SpeechError::Spawn(format!("{e:?}"))
    }

    /// Match a requested voice by exact name, then by language prefix.
    /// Returns `None` (engine default) when no installed voice matches,
    /// rather than fabricating a voice handle.
    fn pick_voice(synth: &SpeechSynthesis, wanted: &str) -> Option<SpeechSynthesisVoice> {
        let voices: Array = synth.get_voices();
        let mut lang_match = None;
        for entry in voices.iter() {
            let Ok(voice) = entry.dyn_into::<SpeechSynthesisVoice>() else {
                continue;
            };
            if voice.name() == wanted {
                return Some(voice);
            }
            if lang_match.is_none()
                && voice
                    .lang()
                    .to_lowercase()
                    .starts_with(&wanted.to_lowercase())
            {
                lang_match = Some(voice);
            }
        }
        lang_match
    }

    pub fn speak(text: &str, options: &SpeechOptions) -> Result<(), SpeechError> {
        let synth = synth().ok_or(SpeechError::Unsupported)?;
        let utter = SpeechSynthesisUtterance::new_with_text(text).map_err(js_err)?;
        if let Some(rate) = options.rate {
            utter.set_rate(rate.clamp(0.1, 10.0));
        }
        if let Some(pitch) = options.pitch {
            utter.set_pitch(pitch.clamp(0.0, 2.0));
        }
        if let Some(volume) = options.volume {
            utter.set_volume(volume.clamp(0.0, 1.0));
        }
        if let Some(wanted) = &options.voice {
            if let Some(voice) = pick_voice(&synth, wanted) {
                utter.set_voice(Some(&voice));
            }
        }
        synth.speak(&utter);
        Ok(())
    }

    pub fn cancel() {
        if let Some(synth) = synth() {
            synth.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_is_a_noop_even_without_engine() {
        let synth = SpeechSynthesizer::new();
        assert!(synth.speak("", &SpeechOptions::new()).is_ok());
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn missing_engine_reports_unsupported_without_faking_audio() {
        // Forced no-backend synthesizer: speak must fail as Unsupported and
        // available() must be false; it must never silently succeed.
        let synth = SpeechSynthesizer::unavailable_for_test();
        assert!(!synth.available());
        assert!(matches!(
            synth.speak("hello", &SpeechOptions::new()),
            Err(SpeechError::Unsupported)
        ));
        synth.stop();
    }

    #[test]
    fn options_builder_sets_every_field() {
        let o = SpeechOptions::new()
            .rate(1.5)
            .pitch(0.9)
            .volume(0.7)
            .voice("Karen");
        assert_eq!(o.rate, Some(1.5));
        assert_eq!(o.pitch, Some(0.9));
        assert_eq!(o.volume, Some(0.7));
        assert_eq!(o.voice.as_deref(), Some("Karen"));
    }

    #[cfg(not(target_family = "wasm"))]
    mod native {
        use super::*;

        fn opts() -> SpeechOptions {
            SpeechOptions::new()
                .rate(1.5)
                .pitch(0.8)
                .volume(1.2)
                .voice("en")
        }

        #[test]
        fn spd_say_argv_maps_relative_scales() {
            let args = build_argv(Engine::SpdSay, "hello world", &opts());
            let args: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
            assert_eq!(
                args,
                vec!["-r", "50", "-p", "-20", "-i", "20", "-v", "en", "--wait", "hello world"]
            );
        }

        #[test]
        fn espeak_argv_maps_words_per_minute() {
            let args = build_argv(Engine::Espeak, "hi", &SpeechOptions::new().rate(2.0));
            let args: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
            assert_eq!(args, vec!["-s", "350", "hi"]);
        }

        #[test]
        fn flite_argv_uses_text_flag_only() {
            let args = build_argv(Engine::Flite, "hi there", &opts());
            let args: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
            assert_eq!(args, vec!["-t", "hi there"]);
        }

        #[test]
        fn macos_say_argv_uses_voice_and_rate() {
            let args = build_argv(Engine::Say, "hi", &opts());
            let args: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
            assert_eq!(args, vec!["-v", "en", "-r", "263", "hi"]);
        }

        #[test]
        fn powershell_argv_escapes_single_quotes() {
            let args = build_argv(Engine::PowerShell, "it's", &SpeechOptions::new());
            let cmd = args[2].to_str().unwrap();
            assert!(cmd.contains("$s.Speak('it''s');"));
            assert!(cmd.contains("Add-Type -AssemblyName System.Speech"));
        }

        #[test]
        fn selection_prefers_first_present_candidate() {
            let expected = candidates()[0];
            let name = expected.program();
            let picked = select_engine(&|p| p == name).expect("first candidate resolves");
            assert_eq!(picked.0, expected);
            assert_eq!(picked.1, PathBuf::from(name));
        }

        #[test]
        fn selection_skips_absent_higher_priority_engine() {
            // Only the lowest-priority engine is present: selection walks past
            // the missing ones. Expected engine is the final host candidate.
            let expected = *candidates().last().unwrap();
            let name = expected.program();
            let picked = select_engine(&|p| p == name).expect("last candidate resolves");
            assert_eq!(picked.0, expected);
        }

        #[test]
        fn selection_is_none_when_no_engine_present() {
            assert!(select_engine(&|_| false).is_none());
        }
    }
}
