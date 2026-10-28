//! `CharGenData` (`portal/client/src/clinet/chargendata.cpp.c`, `CharGenData::SetFileName` in
//! `client.cpp.c`): the capture of character creation. Retail's `chargen <file>` command-line
//! argument names a file; from then on every `Proto_UI::SendCharGenResult` (the packet
//! `gmCharGenResult::Pack` builds, opcode `0x1002A`) also writes itself there, with the account
//! and the character's name found in it replaced by markers, so the capture can be replayed for
//! another account or name. Nothing is kept by default: retail's name is empty and no file is
//! written.
//!
//! The file image is retail's record stream: `0x10000`, the body's size (the packet less the
//! account and name it replaces), then records: `1` and a count and the raw bytes, `2` for the
//! account, `3` for the name, and `0` at the end. The name is the packed `PStringBase<unsigned
//! short>` (`gmCharGenResult::GetName`). The file goes to the user's configuration directory
//! unless the argument is absolute (`ac2_prefs::paths::config_file`), and it is written on a
//! worker thread (`CharGenData::Save` runs on the sending thread; the frame thread never waits
//! on the disk).

use std::io;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use ac2_net::{pack_string, pack_wide_string};

/// The image's first word (`0x10000`).
const MAGIC: u32 = 0x1_0000;
/// The packet's opcode word (`Opcode::CHAR_GEN_RESULT`), before the account; the image starts
/// after it, and `CharGenData::Save` writes nothing for another opcode.
const OPCODE: u32 = ac2_net::protocol::Opcode::CHAR_GEN_RESULT;
const BODY: usize = 4;

/// `CharGenData::s_FileName` and the captures in flight.
#[derive(Debug, Default)]
pub struct CharGenCapture {
    file: Option<PathBuf>,
    saves: Vec<JoinHandle<io::Result<()>>>,
}

impl CharGenCapture {
    /// `CharGenData::SetFileName`: the `chargen` argument (`None` keeps nothing).
    pub fn new(file: Option<PathBuf>) -> Self {
        Self {
            file,
            saves: Vec::new(),
        }
    }

    /// The capture file, when one was named.
    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// `CharGenData::Save`, from `Proto_UI::SendCharGenResult`: the packet (opcode included), the
    /// account and the name, built into an image now and written on a worker thread. Nothing
    /// happens without a file.
    pub fn save(&mut self, packet: &[u8], account: &str, name: &str) {
        let Some(file) = self.file.clone() else {
            return;
        };
        let mut packed_account = Vec::new();
        pack_string(&mut packed_account, 0, account.as_bytes());
        let mut packed_name = Vec::new();
        pack_wide_string(&mut packed_name, 0, name);
        let image = capture_image(packet, &packed_account, &packed_name);
        self.saves
            .push(std::thread::spawn(move || write(&file, &image)));
    }

    /// Waits for the captures in flight (the worker threads) and returns their results.
    pub fn finish(&mut self) -> Vec<io::Result<()>> {
        self.saves
            .drain(..)
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err(io::Error::other("capture thread panicked")))
            })
            .collect()
    }
}

impl Drop for CharGenCapture {
    /// The saves in flight finish before the program exits.
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

fn write(file: &Path, image: &[u8]) -> io::Result<()> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(file, image)
}

/// The first place `needle` occurs in the packet's body (`CharGenData::Save`'s search from the
/// body start).
fn find(packet: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > packet.len().saturating_sub(BODY) {
        return None;
    }
    (BODY..=packet.len() - needle.len()).find(|&at| packet[at..].starts_with(needle))
}

/// `CharGenData::Save`'s image of a packet, given the packed account and name.
pub fn capture_image(packet: &[u8], account: &[u8], name: &[u8]) -> Vec<u8> {
    debug_assert_eq!(packet.get(..BODY), Some(&OPCODE.to_le_bytes()[..]));
    let account_at = find(packet, account);
    let name_at = find(packet, name);
    let mut size = packet.len();
    if account_at.is_some() {
        size -= account.len();
    }
    if name_at.is_some() {
        size -= name.len();
    }
    let mut out = MAGIC.to_le_bytes().to_vec();
    out.extend((size as u32).to_le_bytes());
    let mut at = BODY;
    while at < packet.len() {
        if account_at == Some(at) {
            out.extend(2u32.to_le_bytes());
            at += account.len();
        } else if name_at == Some(at) {
            out.extend(3u32.to_le_bytes());
            at += name.len();
        } else {
            let next = [account_at, name_at]
                .into_iter()
                .flatten()
                .filter(|&marker| marker > at)
                .min()
                .unwrap_or(packet.len());
            out.extend(1u32.to_le_bytes());
            out.extend(((next - at) as u32).to_le_bytes());
            out.extend(&packet[at..next]);
            at = next;
        }
    }
    out.extend(0u32.to_le_bytes());
    out
}

/// The packet an image was made from: the markers filled with the packed account and name.
/// `None` when the image is not a capture.
pub fn reassemble(image: &[u8], account: &[u8], name: &[u8]) -> Option<Vec<u8>> {
    let word = |at: usize| -> Option<u32> {
        Some(u32::from_le_bytes(image.get(at..at + 4)?.try_into().ok()?))
    };
    if word(0)? != MAGIC {
        return None;
    }
    let mut out = OPCODE.to_le_bytes().to_vec();
    let mut at = 8;
    loop {
        match word(at)? {
            0 => return Some(out),
            1 => {
                let len = word(at + 4)? as usize;
                out.extend(image.get(at + 8..at + 8 + len)?);
                at += 8 + len;
            }
            2 => {
                out.extend(account);
                at += 4;
            }
            3 => {
                out.extend(name);
                at += 4;
            }
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_net::protocol::{CharGenResult, ClientEvent};

    fn packet(account: &str, name: &str) -> Vec<u8> {
        let result = CharGenResult {
            name: name.into(),
            entity_did: 0x7000_0001,
            start_area: 2,
            race: 1,
            sex: 4096,
            class: 0,
            implement: 0,
            physical: vec![(2, 1.25), (1, 0.5)],
        };
        ClientEvent::CharGenResult {
            account: account.into(),
            result,
        }
        .encode()
        .1
    }

    #[test]
    fn the_capture_replays_the_packet_with_its_account_and_name_markers() {
        let packet = packet("acct", "Zeda");
        let mut account = Vec::new();
        pack_string(&mut account, 0, b"acct");
        let mut name = Vec::new();
        pack_wide_string(&mut name, 0, "Zeda");
        let image = capture_image(&packet, &account, &name);
        // Size: the packet less the account and name, which are replaced by markers.
        assert_eq!(
            u32::from_le_bytes(image[4..8].try_into().unwrap()) as usize,
            packet.len() - account.len() - name.len()
        );
        assert_eq!(reassemble(&image, &account, &name).unwrap(), packet);
        // The records: the account (right after the opcode), then the name it precedes.
        let tokens: Vec<u32> = image[8..]
            .chunks_exact(4)
            .take(2)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        assert_eq!(tokens, vec![2, 3]);
    }

    #[test]
    fn a_packet_without_its_name_keeps_the_name_bytes_in_its_records() {
        let packet = packet("acct", "Zeda");
        let mut account = Vec::new();
        pack_string(&mut account, 0, b"acct");
        let mut other = Vec::new();
        pack_wide_string(&mut other, 0, "Other");
        let image = capture_image(&packet, &account, &other);
        assert_eq!(
            u32::from_le_bytes(image[4..8].try_into().unwrap()) as usize,
            packet.len() - account.len()
        );
        assert_eq!(reassemble(&image, &account, &other).unwrap(), packet);
    }

    #[test]
    fn nothing_is_saved_without_a_file_name() {
        let mut capture = CharGenCapture::new(None);
        capture.save(&packet("a", "B"), "a", "B");
        assert!(capture.finish().is_empty());
    }

    #[test]
    fn the_capture_is_written_on_a_worker_thread() {
        let dir = std::env::temp_dir().join(format!("ac2-chargen-{}", std::process::id()));
        let file = dir.join("capture.bin");
        let mut capture = CharGenCapture::new(Some(file.clone()));
        capture.save(&packet("acct", "Zeda"), "acct", "Zeda");
        let results = capture.finish();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
        let image = std::fs::read(&file).unwrap();
        let mut account = Vec::new();
        pack_string(&mut account, 0, b"acct");
        let mut name = Vec::new();
        pack_wide_string(&mut name, 0, "Zeda");
        assert_eq!(
            reassemble(&image, &account, &name).unwrap(),
            packet("acct", "Zeda")
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
