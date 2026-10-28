//! Écouteur OSC : un client externe (TouchOSC, Max, Pure Data…) pilote les
//! émotions par UDP.
//!
//! Un seul message est compris, `/harmonium/params` avec quatre flottants
//! (arousal valence density tension) ; il devient un
//! [`EngineCommand::SetEmotionParams`] remis à l'appelant, qui l'exécute sur
//! son hôte (`LiveHost::apply`). Le module ne connaît pas l'hôte : le REPL et
//! le studio natif l'ouvrent de la même façon.

use std::{
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use harmonium_core::{EngineCommand, log};
use rosc::{OscPacket, OscType};

/// Adresse d'écoute par défaut : locale, port 8080.
pub const DEFAULT_ADDR: &str = "127.0.0.1:8080";

/// Adresse OSC des émotions.
pub const PARAMS_ADDRESS: &str = "/harmonium/params";

/// Délai de lecture du socket : borne l'attente de l'arrêt.
const READ_TIMEOUT: Duration = Duration::from_millis(100);

/// Décode un datagramme UDP en commande ; `None` s'il n'est pas un message
/// `/harmonium/params` de quatre arguments au moins. Un argument qui n'est ni
/// `Float` ni `Double` vaut 0.
#[must_use]
pub fn decode(datagram: &[u8]) -> Option<EngineCommand> {
    let Ok((_, OscPacket::Message(message))) = rosc::decoder::decode_udp(datagram) else {
        return None;
    };
    if message.addr != PARAMS_ADDRESS || message.args.len() < 4 {
        return None;
    }
    let float = |arg: &OscType| match arg {
        OscType::Float(value) => *value,
        OscType::Double(value) => *value as f32,
        _ => 0.0,
    };
    Some(EngineCommand::SetEmotionParams {
        arousal: float(&message.args[0]),
        valence: float(&message.args[1]),
        density: float(&message.args[2]),
        tension: float(&message.args[3]),
    })
}

/// Thread d'écoute UDP ; s'arrête (et est attendu) à la destruction.
pub struct OscListener {
    local_addr: SocketAddr,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl OscListener {
    /// Ouvre `addr` et remet chaque commande reçue à `on_command`, sur le
    /// thread d'écoute.
    pub fn spawn<F>(addr: impl ToSocketAddrs, mut on_command: F) -> Result<Self, String>
    where
        F: FnMut(EngineCommand) + Send + 'static,
    {
        let socket = UdpSocket::bind(addr).map_err(|e| format!("socket OSC : {e}"))?;
        socket.set_read_timeout(Some(READ_TIMEOUT)).map_err(|e| format!("socket OSC : {e}"))?;
        let local_addr = socket.local_addr().map_err(|e| format!("socket OSC : {e}"))?;
        log::info(&format!("OSC Listener bound to {local_addr}"));

        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = shutdown.clone();
        let thread = std::thread::Builder::new()
            .name("harmonium-osc".into())
            .spawn(move || {
                let mut buf = [0u8; 4096];
                while !stop.load(Ordering::Relaxed) {
                    match socket.recv_from(&mut buf) {
                        Ok((size, _)) => {
                            if let Some(command) = decode(&buf[..size]) {
                                on_command(command);
                            }
                        }
                        Err(e)
                            if matches!(
                                e.kind(),
                                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                            ) => {}
                        Err(e) => {
                            log::error(&format!("Error receiving UDP packet: {e}"));
                            std::thread::sleep(READ_TIMEOUT);
                        }
                    }
                }
            })
            .map_err(|e| format!("thread OSC : {e}"))?;

        Ok(Self { local_addr, shutdown, thread: Some(thread) })
    }

    /// Adresse réellement ouverte (utile avec le port 0).
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl Drop for OscListener {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use rosc::{OscMessage, encoder};

    use super::*;

    fn packet(addr: &str, args: Vec<OscType>) -> Vec<u8> {
        encoder::encode(&OscPacket::Message(OscMessage { addr: addr.into(), args }))
            .unwrap_or_default()
    }

    #[test]
    fn params_message_becomes_emotion_command() {
        let datagram = packet(
            PARAMS_ADDRESS,
            vec![
                OscType::Float(0.8),
                OscType::Double(-0.5),
                OscType::Float(0.25),
                OscType::Float(0.75),
            ],
        );
        assert_eq!(
            decode(&datagram),
            Some(EngineCommand::SetEmotionParams {
                arousal: 0.8,
                valence: -0.5,
                density: 0.25,
                tension: 0.75
            })
        );
    }

    #[test]
    fn other_addresses_and_short_messages_are_ignored() {
        assert_eq!(decode(&packet("/harmonium/other", vec![OscType::Float(0.0); 4])), None);
        assert_eq!(decode(&packet(PARAMS_ADDRESS, vec![OscType::Float(0.0); 3])), None);
        assert_eq!(decode(b"not osc"), None);
    }

    #[test]
    fn non_float_argument_counts_as_zero() {
        let datagram = packet(
            PARAMS_ADDRESS,
            vec![OscType::Int(7), OscType::Float(0.5), OscType::Float(0.5), OscType::Float(0.5)],
        );
        assert_eq!(
            decode(&datagram),
            Some(EngineCommand::SetEmotionParams {
                arousal: 0.0,
                valence: 0.5,
                density: 0.5,
                tension: 0.5
            })
        );
    }
}
