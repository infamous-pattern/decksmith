//! A bounded, persistent GIO bridge. Only allowlisted commands cross stdin.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub available: bool,
    pub active: bool,
    pub text: String,
}
pub type States = BTreeMap<String, State>;
pub fn valid(command: &str) -> bool {
    matches!(
        command,
        "lock"
            | "suspend"
            | "reboot"
            | "shutdown"
            | "dnd"
            | "night_light"
            | "bluetooth"
            | "power_cycle"
            | "power_saver"
            | "power_balanced"
            | "power_performance"
    )
}
struct Process {
    child: Child,
    input: ChildStdin,
    output: mpsc::Receiver<String>,
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[derive(Default)]
pub struct Client {
    process: Option<Process>,
    cache: States,
    checked: Option<Instant>,
    commands: Vec<String>,
}
impl Client {
    fn request(&mut self, value: serde_json::Value) -> Result<serde_json::Value, &'static str> {
        if self.process.is_none() {
            let mut child = Command::new("/usr/bin/python3")
                .arg("-B")
                .arg(crate::runtime_paths::helper("system_controls.py"))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|_| "system_unavailable")?;
            let input = child.stdin.take().ok_or("system_unavailable")?;
            let output = child.stdout.take().ok_or("system_unavailable")?;
            let (send, receive) = mpsc::sync_channel(1);
            std::thread::spawn(move || {
                for line in BufReader::new(output).lines() {
                    let Ok(line) = line else { break };
                    if send.send(line).is_err() {
                        break;
                    }
                }
            });
            self.process = Some(Process {
                child,
                input,
                output: receive,
            });
        }
        let process = self.process.as_mut().unwrap();
        let result = (|| {
            writeln!(process.input, "{value}").map_err(|_| "system_unavailable")?;
            process.input.flush().map_err(|_| "system_unavailable")?;
            let line = process
                .output
                .recv_timeout(Duration::from_secs(8))
                .map_err(|_| "system_timeout")?;
            let response: serde_json::Value =
                serde_json::from_str(&line).map_err(|_| "system_unavailable")?;
            if response["ok"] != true {
                return Err("system_action_failed");
            };
            Ok(response)
        })();
        if result.is_err() {
            self.process = None;
        }
        result
    }
    pub fn execute(&mut self, command: &str) -> Result<(), &'static str> {
        if !valid(command) {
            return Err("invalid_system_action");
        };
        self.checked = None;
        self.request(serde_json::json!({"execute":command}))
            .map(|_| ())
    }
    pub fn read(&mut self, commands: &[String]) -> States {
        if commands.is_empty() {
            self.process = None;
            self.cache.clear();
            self.commands.clear();
            return States::new();
        }
        if self.commands == commands
            && self
                .checked
                .is_some_and(|t| t.elapsed() < Duration::from_secs(2))
        {
            return self.cache.clone();
        }
        self.commands = commands.to_vec();
        self.checked = Some(Instant::now());
        self.cache = self
            .request(serde_json::json!({"read":commands}))
            .ok()
            .and_then(|v| serde_json::from_value(v["states"].clone()).ok())
            .unwrap_or_else(|| {
                commands
                    .iter()
                    .map(|c| {
                        (
                            c.clone(),
                            State {
                                text: "Unavailable".into(),
                                ..Default::default()
                            },
                        )
                    })
                    .collect()
            });
        self.cache.clone()
    }
}
