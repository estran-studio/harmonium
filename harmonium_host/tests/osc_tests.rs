//! Un client UDP pilote les émotions de l'hôte par OSC : même adresse
//! (`/harmonium/params`), mêmes quatre flottants que l'ancien binaire.
use std::{
    net::UdpSocket,
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};

use harmonium::{
    LiveHost,
    osc::{OscListener, PARAMS_ADDRESS},
};
use harmonium_core::EngineCommand;
use rosc::{OscMessage, OscPacket, OscType, encoder};

fn params_datagram(values: [f32; 4]) -> Result<Vec<u8>, String> {
    encoder::encode(&OscPacket::Message(OscMessage {
        addr: PARAMS_ADDRESS.into(),
        args: values.into_iter().map(OscType::Float).collect(),
    }))
    .map_err(|e| format!("{e:?}"))
}

#[test]
fn udp_client_receives_commands_from_the_listener() -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    let listener = OscListener::spawn("127.0.0.1:0", move |command| {
        let _ = tx.send(command);
    })?;
    let client = UdpSocket::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    client
        .send_to(&params_datagram([0.9, -0.2, 0.4, 0.6])?, listener.local_addr())
        .map_err(|e| e.to_string())?;
    let command = rx.recv_timeout(Duration::from_secs(2)).map_err(|e| e.to_string())?;
    assert_eq!(
        command,
        EngineCommand::SetEmotionParams { arousal: 0.9, valence: -0.2, density: 0.4, tension: 0.6 }
    );
    Ok(())
}

#[test]
fn udp_params_packet_changes_the_emotions_in_effect_on_the_host() -> Result<(), String> {
    let (host, _playback) = LiveHost::detached(None, None, &[], 44100.0)?;
    let host = Arc::new(Mutex::new(host));
    let before = host.lock().map_err(|e| e.to_string())?.emotions();

    let osc_host = host.clone();
    let listener = OscListener::spawn("127.0.0.1:0", move |command| {
        if let Ok(mut host) = osc_host.lock() {
            let _ = host.apply(command);
        }
    })?;
    let client = UdpSocket::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let wanted = [0.9, -0.2, 0.4, 0.6];
    client.send_to(&params_datagram(wanted)?, listener.local_addr()).map_err(|e| e.to_string())?;

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let (now, bpm) = {
            let host = host.lock().map_err(|e| e.to_string())?;
            (host.emotions(), host.musical_params().bpm)
        };
        if now == wanted {
            assert_ne!(before, now);
            assert!(bpm > 0.0);
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err(format!("émotions inchangées : {now:?}"));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
