//! Fixed, bounded helper for application/device audio, outside the HID loop.
use crate::{audio::State, pages::Action};
use std::{
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
pub fn valid(target: &str) -> bool {
    target.len() <= 512
        && !target.chars().any(char::is_control)
        && (matches!(target, "system" | "microphone")
            || ["output:", "input:"].iter().any(|prefix| {
                target
                    .strip_prefix(prefix)
                    .is_some_and(|s| !s.is_empty() && !s.starts_with('-'))
            })
            || [
                "app:application.id=",
                "app:application.process.binary=",
                "app:application.name=",
            ]
            .iter()
            .any(|prefix| target.strip_prefix(prefix).is_some_and(|s| !s.is_empty())))
}
fn helper(operation: &str, value: &str) -> Result<Vec<u8>, &'static str> {
    let path = crate::runtime_paths::helper("audio_targets.py");
    let mut child = Command::new("/usr/bin/python3")
        .arg(path)
        .args([operation, value])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "audio_target_failed")?;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(match status.code() {
                        Some(2) => "audio_target_unavailable",
                        Some(3) => "audio_target_unsupported",
                        Some(4) => "audio_target_timeout",
                        _ => "audio_target_failed",
                    });
                }
                let mut output = Vec::new();
                child
                    .stdout
                    .take()
                    .unwrap()
                    .take(8192)
                    .read_to_end(&mut output)
                    .map_err(|_| "audio_target_failed")?;
                return Ok(output);
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("audio_target_timeout");
            }
        }
    }
}
pub fn execute(action: &Action) -> Result<(), &'static str> {
    helper("execute", &serde_json::to_string(action).unwrap()).map(|_| ())
}
// One persistent reader belongs to the audio worker; snapshots are never cached.
struct Reader {
    child: std::process::Child,
    input: std::process::ChildStdin,
    output: std::sync::mpsc::Receiver<Vec<u8>>,
}
impl Drop for Reader {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[derive(Default)]
pub struct Client {
    reader: Option<Reader>,
}
impl Client {
    fn request(&mut self, targets: &[String]) -> Result<Vec<Option<State>>, ()> {
        use std::io::{BufRead, BufReader, Write};
        if self.reader.is_none() {
            let mut child = Command::new("/usr/bin/python3")
                .arg("-B")
                .arg(crate::runtime_paths::helper("audio_targets.py"))
                .arg("serve-read")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|_| ())?;
            let input = child.stdin.take().unwrap();
            let output = child.stdout.take().unwrap();
            let (send, receive) = std::sync::mpsc::sync_channel(1);
            std::thread::spawn(move || {
                let mut output = BufReader::new(output);
                loop {
                    let mut line = Vec::new();
                    if output
                        .by_ref()
                        .take(8193)
                        .read_until(b'\n', &mut line)
                        .is_err()
                        || line.len() > 8192
                        || line.last() != Some(&b'\n')
                    {
                        break;
                    }
                    if send.send(line).is_err() {
                        break;
                    }
                }
            });
            self.reader = Some(Reader {
                child,
                input,
                output: receive,
            });
        }
        let reader = self.reader.as_mut().unwrap();
        writeln!(
            reader.input,
            "{}",
            serde_json::to_string(targets).map_err(|_| ())?
        )
        .map_err(|_| ())?;
        reader.input.flush().map_err(|_| ())?;
        let line = reader
            .output
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| ())?;
        let states: Vec<Option<State>> = serde_json::from_slice(&line).map_err(|_| ())?;
        if states.len() != targets.len() {
            return Err(());
        }
        Ok(states)
    }
    pub fn read(&mut self, targets: &[String]) -> Vec<Option<State>> {
        if targets.is_empty() {
            return Vec::new();
        }
        match self.request(targets) {
            Ok(states) => states,
            Err(()) => {
                // Discard the pipe on every failed request so a late reply cannot
                // become the next request's state. Next poll starts a fresh helper.
                self.reader = None;
                vec![None; targets.len()]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fake(response: &[u8]) -> Client {
        let mut child = Command::new("/usr/bin/python3")
            .args(["-c", "import sys; sys.stdin.read()"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let (send, output) = std::sync::mpsc::sync_channel(2);
        send.send(response.to_vec()).unwrap();
        send.send(response.to_vec()).unwrap();
        Client {
            reader: Some(Reader {
                child,
                input,
                output,
            }),
        }
    }
    #[test]
    fn successive_reads_reuse_one_process() {
        let mut client = fake(b"[null]\n");
        let pid = client.reader.as_ref().unwrap().child.id();
        for _ in 0..2 {
            assert_eq!(client.read(&["system".into()]), vec![None]);
            assert_eq!(client.reader.as_ref().unwrap().child.id(), pid);
        }
    }
    #[test]
    fn malformed_or_mismatched_reply_discards_connection() {
        for reply in [b"invalid\n".as_slice(), b"[]\n"] {
            let mut client = fake(reply);
            assert_eq!(client.read(&["system".into()]), vec![None]);
            assert!(client.reader.is_none());
        }
    }
    #[test]
    fn closed_reply_channel_discards_connection() {
        let mut client = fake(b"[null]\n");
        for _ in 0..3 {
            client.read(&["system".into()]);
        }
        assert!(client.reader.is_none());
    }
}
