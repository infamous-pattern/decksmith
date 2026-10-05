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
// Bounded helper connections belong to the audio worker; Rust caches no replies.
struct Connection {
    child: std::process::Child,
    input: std::process::ChildStdin,
    output: std::sync::mpsc::Receiver<Vec<u8>>,
    isolated_group: bool,
}
impl Connection {
    fn start(mode: &str) -> Result<Self, &'static str> {
        use std::io::{BufRead, BufReader};
        use std::os::unix::process::CommandExt;
        let mut child = Command::new("/usr/bin/python3")
            .arg("-B")
            .arg(crate::runtime_paths::helper("audio_targets.py"))
            .arg(mode)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map_err(|_| "audio_target_failed")?;
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
        Ok(Self {
            child,
            input,
            output: receive,
            isolated_group: true,
        })
    }
    fn exchange(&mut self, request: &impl serde::Serialize) -> Result<Vec<u8>, &'static str> {
        use std::io::Write;
        let bytes = serde_json::to_vec(request).map_err(|_| "audio_target_failed")?;
        if bytes.len() > 8191 {
            return Err("audio_target_failed");
        }
        self.input
            .write_all(&bytes)
            .and_then(|()| self.input.write_all(b"\n"))
            .and_then(|()| self.input.flush())
            .map_err(|_| "audio_target_failed")?;
        self.output
            .recv_timeout(Duration::from_secs(3))
            .map_err(|error| match error {
                std::sync::mpsc::RecvTimeoutError::Timeout => "audio_target_timeout",
                std::sync::mpsc::RecvTimeoutError::Disconnected => "audio_target_failed",
            })
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        if self.isolated_group {
            // The owned, unreaped child created this process group. Kill its
            // command descendants too, so a timed-out helper leaves no writer.
            unsafe {
                libc::killpg(self.child.id() as libc::pid_t, libc::SIGKILL);
            }
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[derive(Default)]
pub struct Client {
    reader: Option<Connection>,
    writer: Option<Connection>,
    writer_used_at: Option<Instant>,
}
#[derive(serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum ActionReply {
    Ok {},
    Error { code: ActionError },
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum ActionError {
    Unavailable,
    Unsupported,
    Timeout,
    Failed,
}
impl Client {
    fn request(&mut self, targets: &[String]) -> Result<Vec<Option<State>>, ()> {
        if self.reader.is_none() {
            self.reader = Some(Connection::start("serve-read").map_err(|_| ())?);
        }
        let reader = self.reader.as_mut().unwrap();
        let line = reader.exchange(&targets).map_err(|_| ())?;
        let states: Vec<Option<State>> = serde_json::from_slice(&line).map_err(|_| ())?;
        if states.len() != targets.len() {
            return Err(());
        }
        Ok(states)
    }
    pub fn read(&mut self, targets: &[String]) -> Vec<Option<State>> {
        self.expire_idle_writer();
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
    pub fn execute(&mut self, action: &Action) -> Result<(), &'static str> {
        if !matches!(
            action,
            Action::AudioAdjust { .. } | Action::AudioMute { .. } | Action::AudioSelect { .. }
        ) {
            return Err("audio_target_unsupported");
        }
        if self.writer.is_none() {
            self.writer = Some(Connection::start("serve-execute")?);
        }
        let answer = self
            .writer
            .as_mut()
            .unwrap()
            .exchange(action)
            .and_then(|line| {
                serde_json::from_slice::<ActionReply>(&line).map_err(|_| "audio_target_failed")
            });
        match answer {
            Ok(reply) => {
                self.writer_used_at = Some(Instant::now());
                match reply {
                    ActionReply::Ok {} => Ok(()),
                    ActionReply::Error { code } => Err(match code {
                        ActionError::Unavailable => "audio_target_unavailable",
                        ActionError::Unsupported => "audio_target_unsupported",
                        ActionError::Timeout => "audio_target_timeout",
                        ActionError::Failed => "audio_target_failed",
                    }),
                }
            }
            Err(code) => {
                // The write may already have happened. Discard the connection,
                // never replay the uncertain action; only a later input reconnects.
                self.writer = None;
                self.writer_used_at = None;
                Err(code)
            }
        }
    }
    pub fn expire_idle_writer(&mut self) {
        if let Some(last) = self.writer_used_at {
            self.expire_writer_at(Instant::now(), last);
        }
    }
    fn expire_writer_at(&mut self, now: Instant, last: Instant) {
        if now
            .checked_duration_since(last)
            .is_some_and(|idle| idle >= Duration::from_secs(30))
        {
            self.writer = None;
            self.writer_used_at = None;
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
            reader: Some(Connection {
                child,
                input,
                output,
                isolated_group: false,
            }),
            writer: None,
            writer_used_at: None,
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
    fn fake_writer(response: &[u8]) -> Client {
        let mut client = fake(response);
        client.writer = client.reader.take();
        client
    }
    fn action() -> Action {
        Action::AudioAdjust {
            target: "output:fixture".into(),
            percent: 1,
        }
    }
    #[test]
    fn successive_actions_reuse_one_process_and_explicit_errors_do_not_reconnect() {
        for (reply, expected) in [
            (b"{\"status\":\"ok\"}\n".as_slice(), Ok(())),
            (
                b"{\"status\":\"error\",\"code\":\"unavailable\"}\n",
                Err("audio_target_unavailable"),
            ),
            (
                b"{\"status\":\"error\",\"code\":\"timeout\"}\n",
                Err("audio_target_timeout"),
            ),
        ] {
            let mut client = fake_writer(reply);
            let pid = client.writer.as_ref().unwrap().child.id();
            for _ in 0..2 {
                assert_eq!(client.execute(&action()), expected);
                assert_eq!(client.writer.as_ref().unwrap().child.id(), pid);
            }
        }
    }
    #[test]
    fn malformed_or_closed_action_reply_discards_writer() {
        for reply in [
            b"invalid\n".as_slice(),
            b"{\"status\":\"ok\",\"extra\":true}\n",
            b"{\"status\":\"error\",\"code\":\"unknown\"}\n",
        ] {
            let mut client = fake_writer(reply);
            assert_eq!(client.execute(&action()), Err("audio_target_failed"));
            assert!(client.writer.is_none());
            assert!(client.writer_used_at.is_none());
        }
        let mut client = fake_writer(b"{\"status\":\"ok\"}\n");
        assert_eq!(client.execute(&action()), Ok(()));
        assert_eq!(client.execute(&action()), Ok(()));
        assert_eq!(client.execute(&action()), Err("audio_target_failed"));
        assert!(client.writer.is_none());
    }
    #[test]
    fn uncertain_acknowledgement_does_not_replay_action() {
        let path =
            std::env::temp_dir().join(format!("decksmith-action-once-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut child = Command::new("/usr/bin/python3").args(["-B", "-u", "-c",
            "import sys\nfor line in sys.stdin:\n with open(sys.argv[1], 'a') as f: f.write(line)\n print('invalid', flush=True)"])
            .arg(&path).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
        let input = child.stdin.take().unwrap();
        let output = child.stdout.take().unwrap();
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            use std::io::BufRead;
            let mut line = Vec::new();
            std::io::BufReader::new(output)
                .read_until(b'\n', &mut line)
                .unwrap();
            let _ = send.send(line);
        });
        let mut client = Client {
            reader: None,
            writer: Some(Connection {
                child,
                input,
                output: receive,
                isolated_group: false,
            }),
            writer_used_at: None,
        };
        assert_eq!(client.execute(&action()), Err("audio_target_failed"));
        assert!(client.writer.is_none());
        let lines = std::fs::read_to_string(&path).unwrap();
        assert_eq!(lines.lines().count(), 1);
        let request: serde_json::Value = serde_json::from_str(lines.trim()).unwrap();
        assert_eq!(request["percent"], 1);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn dropping_isolated_connection_terminates_its_command_descendants() {
        use std::io::{BufRead, BufReader};
        use std::os::unix::process::CommandExt;
        let mut child = Command::new("/usr/bin/python3")
            .args(["-B", "-u", "-c", "import subprocess,sys,time\np=subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)\nprint(p.pid,flush=True)\ntime.sleep(60)"])
            .process_group(0).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
        let input = child.stdin.take().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        let descendant: u32 = line.trim().parse().unwrap();
        let (_, receive) = std::sync::mpsc::sync_channel(1);
        drop(Connection {
            child,
            input,
            output: receive,
            isolated_group: true,
        });
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            let status = std::fs::read_to_string(format!("/proc/{descendant}/stat"));
            let dead = match status {
                Ok(stat) => stat
                    .rsplit(')')
                    .next()
                    .and_then(|tail| tail.split_whitespace().next())
                    .is_some_and(|state| matches!(state, "Z" | "X")),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
                Err(error) => panic!("cannot verify command cleanup: {error}"),
            };
            if dead {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "owned command survived connection cleanup"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    #[test]
    fn idle_writer_is_reaped_without_discarding_reader() {
        let mut client = fake(b"[null]\n");
        let reader_pid = client.reader.as_ref().unwrap().child.id();
        client.writer = fake(b"{\"status\":\"ok\"}\n").reader;
        let writer_pid = client.writer.as_ref().unwrap().child.id();
        let last = Instant::now();
        client.writer_used_at = Some(last);
        client.expire_writer_at(last + Duration::from_secs(29), last);
        assert_eq!(client.writer.as_ref().unwrap().child.id(), writer_pid);
        client.expire_writer_at(last + Duration::from_secs(30), last);
        assert!(client.writer.is_none());
        assert!(client.writer_used_at.is_none());
        assert_eq!(client.reader.as_ref().unwrap().child.id(), reader_pid);
        assert!(!std::path::Path::new(&format!("/proc/{writer_pid}")).exists());
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
