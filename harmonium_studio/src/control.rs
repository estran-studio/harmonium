//! Le canal de commande local du studio natif, et la commande `open`.
//!
//! **Protocole** : TCP sur `127.0.0.1` ([`DEFAULT_ADDR`]). Le client envoie une ligne, un lien
//! `harmonium://open?…` (voir [`crate::link`]) ; le studio répond une ligne, `ok` (le profil est
//! lu et validé, la demande est transmise à l'écran) ou `error: <message>` (lien ou profil
//! refusé ; rien n'a changé). Il n'y a pas d'autre message : ni lecture, ni état, ni commande du
//! moteur. Les clients OSC (`harmonium::osc`) pilotent les émotions par UDP sans réponse et ne
//! disent pas si un studio écoute : c'est pourquoi le canal est à part.
//!
//! **Local seulement** : le canal n'écoute que sur une adresse de boucle locale (l'ouverture est
//! refusée sinon) et refuse toute connexion dont l'origine n'est pas locale. La première ligne
//! doit être un lien : une requête HTTP envoyée par une page web vers ce port (`POST / HTTP/1.1`)
//! est refusée.
//!
//! **`harmonium_studio open`** ([`open_command`]) : si un studio répond, lui transmet le lien ;
//! sinon lance le studio (processus détaché) avec `--open <lien>`.

use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    path::Path,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use crate::link::{OpenRequest, percent_encode};

/// Adresse du canal par défaut.
pub const DEFAULT_ADDR: &str = "127.0.0.1:47820";

/// Longueur maximale d'une ligne (un lien, quelques centaines d'octets).
const MAX_LINE: u64 = 8192;

/// Délai d'une connexion : borne ce qu'un client muet retarde les suivants.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(2);

/// Délai de la réponse du studio à `open` : il lit le profil avant de répondre.
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

/// Attente entre deux `accept` : borne l'attente de l'arrêt.
const ACCEPT_PERIOD: Duration = Duration::from_millis(50);

/// Résout `addr` et exige une adresse de boucle locale.
fn loopback(addr: &str) -> Result<SocketAddr, String> {
    let mut addrs: Vec<SocketAddr> =
        addr.to_socket_addrs().map_err(|e| format!("control address `{addr}`: {e}"))?.collect();
    if addrs.is_empty() || addrs.iter().any(|a| !a.ip().is_loopback()) {
        return Err(format!("control address `{addr}` is not local (127.0.0.1 or ::1 only)"));
    }
    Ok(addrs.swap_remove(0))
}

/// Le canal : un fil d'écoute ; s'arrête (et est attendu) quand on le lâche.
pub struct Control {
    local_addr: SocketAddr,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Control {
    /// Ouvre `addr` (local) ; chaque lien reçu et bien formé passe à `on_open`, sur le fil
    /// d'écoute, dont le résultat est la réponse au client.
    pub fn spawn<F>(addr: &str, mut on_open: F) -> Result<Self, String>
    where
        F: FnMut(OpenRequest) -> Result<(), String> + Send + 'static,
    {
        let listener = TcpListener::bind(loopback(addr)?)
            .map_err(|e| format!("control channel {addr}: {e}"))?;
        listener.set_nonblocking(true).map_err(|e| format!("control channel: {e}"))?;
        let local_addr = listener.local_addr().map_err(|e| format!("control channel: {e}"))?;

        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = shutdown.clone();
        let thread = std::thread::Builder::new()
            .name("studio-control".into())
            .spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((stream, peer)) if peer.ip().is_loopback() => {
                            // Un client qui échoue (déconnecté, trop lent) ne gêne pas les suivants.
                            let _ = serve(stream, &mut on_open);
                        }
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            std::thread::sleep(ACCEPT_PERIOD);
                        }
                        Err(_) => std::thread::sleep(ACCEPT_PERIOD),
                    }
                }
            })
            .map_err(|e| format!("control thread: {e}"))?;
        Ok(Self { local_addr, shutdown, thread: Some(thread) })
    }

    /// Adresse réellement ouverte (utile avec le port 0).
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl Drop for Control {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Lectures (de 1 Kio) que le canal consacre à vider une connexion refusée.
const DRAIN_READS: usize = 64;

/// Une connexion : lit une ligne, répond une ligne.
fn serve(
    stream: TcpStream,
    on_open: &mut impl FnMut(OpenRequest) -> Result<(), String>,
) -> io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(CLIENT_TIMEOUT))?;
    stream.set_write_timeout(Some(CLIENT_TIMEOUT))?;
    let mut line = String::new();
    BufReader::new((&stream).take(MAX_LINE)).read_line(&mut line)?;
    let reply = if line.len() as u64 >= MAX_LINE && !line.ends_with('\n') {
        Err("line too long".to_string())
    } else {
        OpenRequest::parse(&line).and_then(&mut *on_open)
    };
    let mut writer = &stream;
    match reply {
        Ok(()) => writeln!(writer, "ok")?,
        Err(message) => writeln!(writer, "error: {}", message.replace('\n', " "))?,
    }
    // Ce que le client a envoyé après sa première ligne (une requête HTTP, une ligne trop
    // longue) est lu et jeté : fermer avec des octets non lus ferait réinitialiser la connexion
    // avant que le client n'ait lu la réponse.
    stream.shutdown(Shutdown::Write)?;
    let mut sink = [0u8; 1024];
    for _ in 0..DRAIN_READS {
        match (&stream).read(&mut sink) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
    Ok(())
}

/// Ce qui est arrivé à un lien transmis à un studio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    /// Un studio l'a accepté.
    Delivered,
    /// Personne n'écoute sur l'adresse.
    NotRunning,
}

/// Transmet `link` au studio qui écoute sur `addr`. `Err` : un studio répond mais refuse (le
/// message est le sien), ou ce qui répond n'est pas un studio.
pub fn forward(addr: &str, link: &str) -> Result<Delivery, String> {
    let target = loopback(addr)?;
    let stream = match TcpStream::connect_timeout(&target, CLIENT_TIMEOUT) {
        Ok(stream) => stream,
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => return Ok(Delivery::NotRunning),
        Err(e) => return Err(format!("studio at {target}: {e}")),
    };
    let io_error = |e: io::Error| format!("studio at {target}: {e}");
    stream.set_write_timeout(Some(CLIENT_TIMEOUT)).map_err(io_error)?;
    stream.set_read_timeout(Some(REPLY_TIMEOUT)).map_err(io_error)?;
    writeln!(&stream, "{link}").map_err(io_error)?;
    let mut reply = String::new();
    BufReader::new(&stream).read_line(&mut reply).map_err(io_error)?;
    match reply.trim() {
        "ok" => Ok(Delivery::Delivered),
        message if message.starts_with("error: ") => Err(message["error: ".len()..].to_string()),
        "" => Err(format!("studio at {target} closed the connection without an answer")),
        other => Err(format!("{target} is not a Harmonium studio (answered `{other}`)")),
    }
}

/// Lance le studio (même exécutable) avec `--open <lien>`, détaché : il survit au terminal.
pub fn launch(link: &str, addr: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot find the studio binary: {e}"))?;
    let mut command = Command::new(exe);
    command
        .args(["--open", link, "--control", addr])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
        command.creation_flags(0x0000_0008 | 0x0000_0200);
    }
    command.spawn().map(drop).map_err(|e| format!("cannot start the studio: {e}"))
}

const OPEN_USAGE: &str = "usage: harmonium_studio open [--control <addr>] <harmonium://open?… link>\n       \
harmonium_studio open [--control <addr>] --profile <id|folder> [--seed <n>] [--bar <n>] [--loop <a-b>]\n           \
[--bpm <n>] [--swing <r>] [--solo <track>] [--mute <track,…>] [--play]";

/// La commande `harmonium_studio open …` (les arguments après `open`). Rend le code de sortie :
/// 0 le lien est ouvert (dans le studio qui tournait, ou dans un studio lancé), 1 refusé, 2 mal
/// appelé.
#[must_use]
pub fn open_command(args: &[String]) -> i32 {
    let (request, addr) = match parse_open_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("harmonium_studio open: {message}");
            return 2;
        }
    };
    let link = request.to_link();
    match forward(&addr, &link) {
        Ok(Delivery::Delivered) => {
            println!("opened in the running studio: {link}");
            0
        }
        Ok(Delivery::NotRunning) => match launch(&link, &addr) {
            Ok(()) => {
                println!("started the studio: {link}");
                0
            }
            Err(message) => {
                eprintln!("harmonium_studio open: {message}");
                1
            }
        },
        Err(message) => {
            eprintln!("harmonium_studio open: {message}");
            1
        }
    }
}

/// Lit les arguments de `open` : le lien (ou les options qui le décrivent) et l'adresse du canal.
pub fn parse_open_args(args: &[String]) -> Result<(OpenRequest, String), String> {
    let mut addr = DEFAULT_ADDR.to_string();
    let mut link: Option<&str> = None;
    let mut query: Vec<String> = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = || {
            args.next()
                .map(String::as_str)
                .ok_or_else(|| format!("{arg} needs a value\n{OPEN_USAGE}"))
        };
        let mut pair =
            |key: &str, value: &str| query.push(format!("{key}={}", percent_encode(value)));
        match arg.as_str() {
            "--control" => addr = value()?.to_string(),
            "--profile" => {
                let value = value()?;
                // Un dossier qui existe (même relatif, ou écrit `~/…` entre guillemets) devient
                // son chemin absolu, pour le studio qui ne tourne pas dans ce dossier.
                let candidate = value
                    .strip_prefix("~/")
                    .and_then(|rest| dirs::home_dir().map(|home| home.join(rest)))
                    .unwrap_or_else(|| Path::new(value).to_path_buf());
                let resolved = candidate
                    .is_dir()
                    .then(|| std::fs::canonicalize(&candidate).ok())
                    .flatten()
                    .map_or_else(|| value.to_string(), |path| path.display().to_string());
                pair("profile", &resolved);
            }
            "--seed" | "--bar" | "--loop" | "--bpm" | "--swing" | "--solo" | "--mute" => {
                pair(&arg[2..], value()?);
            }
            "--play" => pair("play", "1"),
            flag if flag.starts_with("--") => {
                return Err(format!("unknown option `{flag}`\n{OPEN_USAGE}"));
            }
            positional if link.is_none() => link = Some(positional),
            _ => return Err(format!("only one link\n{OPEN_USAGE}")),
        }
    }
    let request = match (link, query.is_empty()) {
        (Some(_), false) => {
            return Err(format!("give a link or options, not both\n{OPEN_USAGE}"));
        }
        (Some(link), true) => OpenRequest::parse(link)?,
        (None, false) => OpenRequest::from_query(&query.join("&"), &[])?,
        (None, true) => return Err(format!("nothing to open\n{OPEN_USAGE}")),
    };
    Ok((request, addr))
}
