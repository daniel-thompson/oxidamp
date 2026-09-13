// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022, 2023 Daniel Thompson

//! JACK plumbing: the notification handler plus a small auto-patch routine that
//! wires oxidamp's ports into whatever the running server provides.

/// JACK's port type for a single channel of floating point audio.
const AUDIO_TYPE: &str = "32 bit float mono audio";

/// JACK's port type for raw MIDI.
const MIDI_TYPE: &str = "8 bit raw midi";

pub struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("jack: thread init");
    }

    // SAFETY: called from the JACK server thread during shutdown; the body
    // only prints, so it upholds the async-signal-safety contract in practice.
    unsafe fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "jack: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "jack: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("jack: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "jack: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "jack: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "jack: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "jack: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("jack: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("jack: xrun occurred");
        jack::Control::Continue
    }
}

/// The oxidamp ports the auto-patch routine routes.
///
/// Every field is a full JACK port name (`client:port`), captured when the
/// ports were registered.
pub struct PatchBay {
    pub synth_out: String,
    pub amp_in: String,
    pub tuner_in: String,
    pub synth_in: String,
    pub amp_out: String,
    pub drums_l: String,
    pub drums_r: String,
    pub metronome: String,
}

impl PatchBay {
    /// Wire up the standard oxidamp patch against the running server.
    ///
    /// The synth is fed to the amp and the tuner, a default MIDI source is fed
    /// to the synth, and the outputs are sent to the default stereo output:
    /// the amp and metronome are mono so they go to both channels, while the
    /// drums already have a left and right.
    ///
    /// Ports that cannot be found are reported on stderr and skipped - what is
    /// available depends on the machine, so a partial patch is better than
    /// nothing. Connections that already exist are left alone.
    pub fn connect(&self, client: &jack::Client) {
        let own = client.name();

        self.join(client, &self.synth_out, &self.amp_in);
        self.join(client, &self.synth_out, &self.tuner_in);

        match default_midi_source(client, own) {
            Some(source) => self.join(client, &source, &self.synth_in),
            None => eprintln!("jack: no MIDI source found to connect to {}", self.synth_in),
        }

        match default_speakers(client, own) {
            Some((left, right)) => {
                self.join(client, &self.amp_out, &left);
                self.join(client, &self.amp_out, &right);
                self.join(client, &self.metronome, &left);
                self.join(client, &self.metronome, &right);
                self.join(client, &self.drums_l, &left);
                self.join(client, &self.drums_r, &right);
            }
            None => eprintln!("jack: no speaker outputs found to connect to"),
        }
    }

    /// Connect `source` to `destination`, reporting rather than propagating
    /// failure: one missing port should not abandon the rest of the patch.
    fn join(&self, client: &jack::Client, source: &str, destination: &str) {
        match client.connect_ports_by_name(source, destination) {
            Ok(()) => println!("jack: connected {source} -> {destination}"),
            Err(jack::Error::PortAlreadyConnected(..)) => {
                println!("jack: {source} -> {destination} is already connected");
            }
            Err(e) => eprintln!("jack: cannot connect {source} -> {destination}: {e}"),
        }
    }
}

/// Pick a MIDI source to drive the synth.
///
/// JACK exposes hardware MIDI as `system:midi_capture_*` and PipeWire's ALSA
/// bridge uses similar names, so those are preferred. The `Midi Through`
/// loopback, and anything that does not look like MIDI at all, sorts last.
/// The first suitable port wins.
fn default_midi_source(client: &jack::Client, own: &str) -> Option<String> {
    let mut ports = client.ports(None, Some(MIDI_TYPE), jack::PortFlags::IS_OUTPUT);

    // Never feed oxidamp's own MIDI output back into itself.
    ports.retain(|port| !is_ours(port, own));

    ports.sort_by_key(|port| (midi_rank(port), port.clone()));
    ports.into_iter().next()
}

/// Rank a MIDI source so that the most useful default sorts first.
fn midi_rank(port: &str) -> u8 {
    let lower = port.to_ascii_lowercase();

    if port.starts_with("system:midi_capture") {
        0
    } else if lower.contains("midi") && !lower.contains("through") {
        1
    } else if lower.contains("midi") {
        2
    } else {
        3
    }
}

/// Pick the left and right speaker outputs.
///
/// PipeWire presents its default sink as `system:playback_1`/`_2`, so those
/// are preferred, then any physical output, then any other port that can accept
/// audio. oxidamp's own inputs are never chosen: feeding an output back into the
/// amp is never what was wanted.
fn default_speakers(client: &jack::Client, own: &str) -> Option<(String, String)> {
    let inputs = client.ports(None, Some(AUDIO_TYPE), jack::PortFlags::IS_INPUT);
    let physical = client.ports(
        None,
        Some(AUDIO_TYPE),
        jack::PortFlags::IS_INPUT | jack::PortFlags::IS_PHYSICAL,
    );

    let mut ordered = Vec::new();
    for port in inputs.iter().filter(|p| p.starts_with("system:playback")) {
        add_unique(&mut ordered, port, own);
    }
    for port in &physical {
        add_unique(&mut ordered, port, own);
    }
    for port in &inputs {
        add_unique(&mut ordered, port, own);
    }

    match ordered.as_slice() {
        [] => None,
        [only] => Some((only.clone(), only.clone())),
        [left, right, ..] => Some((left.clone(), right.clone())),
    }
}

/// Push `port` onto `ordered` unless it is already there or belongs to us.
fn add_unique(ordered: &mut Vec<String>, port: &str, own: &str) {
    if !is_ours(port, own) && !ordered.iter().any(|p| p == port) {
        ordered.push(port.to_string());
    }
}

/// Whether `port` belongs to the client called `own`.
fn is_ours(port: &str, own: &str) -> bool {
    port.strip_prefix(own)
        .is_some_and(|rest| rest.starts_with(':'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ours() {
        assert!(is_ours("Oxidamp:amp", "Oxidamp"));
        assert!(!is_ours("system:playback_1", "Oxidamp"));
        // A client whose name merely starts with ours is a different client.
        assert!(!is_ours("Oxidamp2:amp", "Oxidamp"));
    }

    #[test]
    fn test_midi_rank_prefers_hardware() {
        // The classic JACK hardware capture ports win...
        assert!(midi_rank("system:midi_capture_1") < midi_rank("Midi-Bridge:foo"));
        // ... then anything MIDI-shaped, with the loopback last...
        assert!(midi_rank("Midi-Bridge:foo") < midi_rank("Midi Through:midi_out"));
        // ... and anything that is not MIDI at all after that.
        assert!(midi_rank("Midi Through:midi_out") < midi_rank("weird:port"));
    }

    #[test]
    fn test_add_unique_skips_duplicates_and_our_ports() {
        let mut ordered = Vec::new();
        add_unique(&mut ordered, "system:playback_1", "Oxidamp");
        add_unique(&mut ordered, "system:playback_1", "Oxidamp");
        add_unique(&mut ordered, "Oxidamp:amp_in", "Oxidamp");
        add_unique(&mut ordered, "system:playback_2", "Oxidamp");
        assert_eq!(ordered, ["system:playback_1", "system:playback_2"]);
    }
}
