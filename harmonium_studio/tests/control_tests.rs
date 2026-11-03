//! Le canal de commande local : un client transmet un lien, le studio répond `ok` ou
//! `error: …`. Aucun studio ni fenêtre ici : le test tient le rôle du studio avec un
//! gestionnaire. Le lancement d'un studio (`launch`) n'est jamais exercé : il ouvrirait une
//! fenêtre avec une sortie audio.

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::mpsc,
    time::Duration,
};

use harmonium_studio::{
    control::{Control, Delivery, forward, open_command, parse_open_args},
    link::{OpenRequest, ProfileRef},
};

/// Un « studio » qui rend ce qu'il reçoit et répond `answer`.
fn studio(
    answer: Result<(), String>,
) -> Result<(Control, mpsc::Receiver<OpenRequest>, String), String> {
    let (sender, received) = mpsc::channel();
    let control = Control::spawn("127.0.0.1:0", move |request| {
        let _ = sender.send(request);
        answer.clone()
    })?;
    let addr = control.local_addr().to_string();
    Ok((control, received, addr))
}

/// Envoie `bytes` tels quels et rend la réponse.
fn raw(addr: &str, bytes: &[u8]) -> Result<String, String> {
    let mut stream = TcpStream::connect(addr).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;
    stream.write_all(bytes).map_err(|e| e.to_string())?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).map_err(|e| e.to_string())?;
    Ok(reply)
}

const LINK: &str = "harmonium://open?profile=reel_quebecois&seed=3&bar=17&loop=17-24&solo=Bass";

#[test]
fn a_link_reaches_the_studio_and_is_acknowledged() -> Result<(), String> {
    let (_control, received, addr) = studio(Ok(()))?;
    assert_eq!(forward(&addr, LINK)?, Delivery::Delivered);
    let request = received.recv_timeout(Duration::from_secs(5)).map_err(|e| e.to_string())?;
    assert_eq!(request, OpenRequest::parse(LINK)?);
    Ok(())
}

#[test]
fn a_studio_that_refuses_says_why() -> Result<(), String> {
    let (_control, received, addr) = studio(Err("cells.toml:12: melody: bad note".into()))?;
    let error = forward(&addr, LINK).err().ok_or("le refus doit remonter")?;
    assert_eq!(error, "cells.toml:12: melody: bad note");
    assert!(received.recv_timeout(Duration::from_secs(5)).is_ok(), "le studio a vu la demande");
    Ok(())
}

#[test]
fn what_is_not_a_link_is_refused_without_reaching_the_studio() -> Result<(), String> {
    let (_control, received, addr) = studio(Ok(()))?;
    // Une page web qui vise ce port envoie une requête HTTP : sa première ligne n'est pas un lien.
    let http =
        raw(&addr, b"POST / HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\nharmonium://open?seed=1\r\n")?;
    assert!(http.starts_with("error: not a harmonium://"), "{http:?}");
    // Un lien mal formé : le message du lecteur de liens.
    let bad = raw(&addr, b"harmonium://open?seed=x\n")?;
    assert_eq!(bad, "error: seed: invalid seed `x`\n");
    // Une ligne sans fin (plus longue que la limite) n'est pas un lien non plus.
    let long = raw(&addr, &vec![b'a'; 20_000])?;
    assert!(long.starts_with("error: "), "{long:?}");
    assert!(received.recv_timeout(Duration::from_millis(200)).is_err(), "rien n'est transmis");
    // Et le canal sert encore le client suivant.
    assert_eq!(forward(&addr, LINK)?, Delivery::Delivered);
    Ok(())
}

#[test]
fn the_channel_only_listens_locally() {
    for addr in ["0.0.0.0:0", "192.168.1.10:0"] {
        let error = Control::spawn(addr, |_| Ok(())).err().unwrap_or_default();
        assert!(error.contains("not local"), "{addr} → {error}");
    }
    let error = forward("0.0.0.0:47820", LINK).err().unwrap_or_default();
    assert!(error.contains("not local"), "{error}");
}

#[test]
fn nobody_listening_is_reported_not_failed() -> Result<(), String> {
    // Un port qui vient d'être libéré.
    let addr = {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        listener.local_addr().map_err(|e| e.to_string())?.to_string()
    };
    assert_eq!(forward(&addr, LINK)?, Delivery::NotRunning);
    Ok(())
}

#[test]
fn something_else_on_the_port_is_not_taken_for_a_studio() -> Result<(), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let addr = listener.local_addr().map_err(|e| e.to_string())?.to_string();
    let thread = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut line = [0u8; 256];
            let _ = stream.read(&mut line);
            let _ = stream.write_all(b"SSH-2.0-OpenSSH\n");
        }
    });
    let error = forward(&addr, LINK).err().ok_or("doit échouer")?;
    assert!(error.contains("not a Harmonium studio"), "{error}");
    let _ = thread.join();
    Ok(())
}

#[test]
fn the_open_command_hands_a_link_to_the_running_studio() -> Result<(), String> {
    let (_control, received, addr) = studio(Ok(()))?;
    let args: Vec<String> = ["--control", &addr, "harmonium://open?seed=7&bar=3"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(open_command(&args), 0);
    let request = received.recv_timeout(Duration::from_secs(5)).map_err(|e| e.to_string())?;
    assert_eq!((request.seed, request.bar), (Some(7), Some(3)));

    // Le studio refuse : code 1, et rien d'autre n'est lancé.
    let (_control, _received, refusing) = studio(Err("no".into()))?;
    let args: Vec<String> =
        ["--control", &refusing, "harmonium://open?seed=7"].into_iter().map(String::from).collect();
    assert_eq!(open_command(&args), 1);
    // Un lien invalide est refusé avant tout envoi : code 2.
    let args: Vec<String> =
        ["--control", &addr, "harmonium://open?seed=x"].into_iter().map(String::from).collect();
    assert_eq!(open_command(&args), 2);
    Ok(())
}

#[test]
fn the_open_options_build_the_same_link_as_the_url() -> Result<(), String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let folder = std::fs::canonicalize(dir.path()).map_err(|e| e.to_string())?;
    let args = |list: &[&str]| -> Vec<String> { list.iter().map(|a| (*a).to_string()).collect() };

    let (request, addr) = parse_open_args(&args(&[
        "--profile",
        &folder.display().to_string(),
        "--seed",
        "3",
        "--bar",
        "17",
        "--loop",
        "17-24",
        "--mute",
        "Melody,Bass",
        "--solo",
        "Chords",
        "--bpm",
        "180",
        "--swing",
        "0.6",
        "--play",
    ]))?;
    assert_eq!(addr, harmonium_studio::control::DEFAULT_ADDR);
    assert_eq!(request.profile, Some(ProfileRef::Folder(PathBuf::from(&folder))));
    assert_eq!(
        (request.seed, request.bar, request.loop_range),
        (Some(3), Some(17), Some((17, 24)))
    );
    assert_eq!((request.bpm, request.swing, request.play), (Some(180.0), Some(0.6), true));
    assert_eq!(OpenRequest::parse(&request.to_link())?, request);

    // Un identifiant qui n'est pas un dossier reste un identifiant.
    let (request, _) = parse_open_args(&args(&["--profile", "valse_quebecoise"]))?;
    assert_eq!(request.profile, Some(ProfileRef::Id("valse_quebecoise".into())));

    // Un dossier relatif (ici, celui des profils du dépôt, depuis le dossier de la crate) devient
    // absolu : le studio ne tourne pas dans le dossier courant.
    let repo_profile =
        std::fs::canonicalize("../profiles/reel_quebecois").map_err(|e| e.to_string())?;
    let (request, _) = parse_open_args(&args(&["--profile", "../profiles/reel_quebecois"]))?;
    assert_eq!(request.profile, Some(ProfileRef::Folder(repo_profile)));

    for (bad, reason) in [
        (args(&[]), "nothing to open"),
        (args(&["harmonium://open", "--seed", "3"]), "not both"),
        (args(&["--seed"]), "needs a value"),
        (args(&["--volume", "3"]), "unknown option"),
        (args(&["--bar", "0"]), "invalid bar"),
    ] {
        let error = parse_open_args(&bad).err().unwrap_or_default();
        assert!(error.contains(reason), "{bad:?} → {error}");
    }
    Ok(())
}
