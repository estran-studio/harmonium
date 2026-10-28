//! Navigateur : `fetch` en flux, Cache Storage, minuteur de Worker, canvas eframe.

use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use eframe::egui;
use harmonium::LiveHost;
use js_sys::{Array, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    Blob, BlobPropertyBag, Cache, CacheStorage, MessageEvent, ReadableStreamDefaultReader,
    RequestCache, RequestInit, Response, Url, UrlSearchParams, Worker,
};

use super::{LoadHandle, update_state};
use crate::{
    app::StudioApp,
    loading::{
        CACHE_NAME, CacheDecision, LoadState, RESOURCES_DIR, ResourceStatus, Source, cache_key,
        check_size, decide, parse_buffer_frames, stale_entries,
    },
    manifest::{LoadedResources, MANIFEST_FILE, Manifest, Resource},
};

/// Tampon audio par défaut : 2048 images (≈ 46 ms à 44,1 kHz). S1 a mesuré zéro tampon en retard
/// à 1024 et à 4096 images avec le moteur et l'interface sur le fil principal ; 2048 est entre les
/// deux, retenu sans mesure propre. `?buffer=<images>` le change.
const DEFAULT_BUFFER_FRAMES: u32 = 2048;

/// Période du minuteur de génération. Il vit dans un Worker : un minuteur du fil de la page est
/// ralenti, puis arrêté, quand l'onglet est masqué. À 100 ms : 9 à 30 réveils par mesure (0,94 s à
/// 3 s, voir `LOOKAHEAD_BARS`), donc un changement de commande est recomposé en 0,1 s au plus, et
/// l'enregistreur (qui veut un `pump` au moins toutes les 3 s) en reçoit trente fois son besoin ;
/// même ralenti à 1 s par le navigateur, il en reçoit trois fois.
const TICK_MS: u32 = 100;

/// Réglages du studio web.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Taille du tampon audio en images (`?buffer=`).
    pub buffer_frames: Option<u32>,
}

/// Lit les paramètres de l'URL : `?buffer=<images>`.
pub fn settings() -> Result<Settings, String> {
    let search =
        web_sys::window().ok_or("no window")?.location().search().map_err(|e| format!("{e:?}"))?;
    let params = UrlSearchParams::new_with_str(&search).map_err(|e| format!("{e:?}"))?;
    let buffer_frames = match params.get("buffer") {
        Some(text) => parse_buffer_frames(&text)?,
        None => DEFAULT_BUFFER_FRAMES,
    };
    Ok(Settings { buffer_frames: Some(buffer_frames) })
}

// === Ressources : manifeste, Cache Storage, téléchargement en flux ===

/// Lit le manifeste, puis chaque ressource : depuis Cache Storage si l'entrée a la taille du
/// manifeste, sinon par `fetch` avec progression. Une ressource n'entre au cache qu'après contrôle
/// de sa taille. Après un chargement réussi, les entrées que le manifeste ne cite plus sont
/// supprimées. Cache absent ou refusé : les octets restent en mémoire et l'écran le dit.
pub fn start_loading(ctx: egui::Context, _settings: &Settings) -> LoadHandle {
    let state: LoadHandle = Arc::default();
    let task = state.clone();
    spawn_local(async move {
        if let Err(error) = load(&task, &ctx).await {
            update_state(&task, &ctx, |s| s.fail(error));
        }
    });
    state
}

async fn load(state: &Mutex<LoadState>, ctx: &egui::Context) -> Result<(), String> {
    // Le manifeste se revalide à chaque visite : c'est lui qui annonce une nouvelle version.
    let text = fetch_text(&format!("{RESOURCES_DIR}/{MANIFEST_FILE}")).await?;
    let manifest = Manifest::parse(&text)?;
    update_state(state, ctx, |s| s.manifest_loaded(&manifest));

    let mut cache = match open_cache().await {
        Ok(cache) => {
            update_state(state, ctx, LoadState::cache_working);
            Some(cache)
        }
        Err(reason) => {
            update_state(state, ctx, |s| s.cache_unavailable(reason));
            None
        }
    };

    let mut contents = BTreeMap::new();
    for resource in &manifest.resources {
        let bytes = load_resource(resource, &mut cache, state, ctx).await?;
        contents.insert(resource.id.clone(), bytes);
    }

    if let Some(cache) = &cache
        && let Err(error) = prune(cache, &manifest).await
    {
        // Le nettoyage attend la prochaine visite ; la musique ne dépend pas de lui.
        log::warn!("cache cleanup failed: {error}");
    }

    let loaded = LoadedResources::assemble(&manifest, contents)?;
    update_state(state, ctx, |s| s.finish(loaded));
    Ok(())
}

async fn load_resource(
    resource: &Resource,
    cache: &mut Option<Cache>,
    state: &Mutex<LoadState>,
    ctx: &egui::Context,
) -> Result<Vec<u8>, String> {
    let id = resource.id.as_str();
    let key = cache_key(resource);
    update_state(state, ctx, |s| s.set_status(id, ResourceStatus::Checking));

    if let Some(open) = cache.as_ref() {
        // Une entrée illisible vaut une absence : on retélécharge.
        let cached = read_cached(open, &key).await.unwrap_or(None);
        if decide(resource, cached.as_ref().map(|bytes| bytes.len() as u64))
            == CacheDecision::UseCached
            && let Some(bytes) = cached
        {
            update_state(state, ctx, |s| s.set_status(id, ResourceStatus::Loaded(Source::Device)));
            return Ok(bytes);
        }
    }

    let bytes = download(resource, |got| {
        update_state(state, ctx, |s| s.set_status(id, ResourceStatus::Downloading(got)));
    })
    .await?;
    check_size(resource, bytes.len() as u64)?;

    if let Some(open) = cache.as_ref()
        && let Err(reason) = put_cached(open, &key, &bytes).await
    {
        // Quota, navigation privée… : on joue avec les octets en mémoire, et on prévient.
        *cache = None;
        update_state(state, ctx, |s| s.cache_unavailable(reason));
    }
    update_state(state, ctx, |s| s.set_status(id, ResourceStatus::Loaded(Source::Network)));
    Ok(bytes)
}

fn js_error(error: JsValue) -> String {
    error.as_string().unwrap_or_else(|| {
        Reflect::get(&error, &"message".into())
            .ok()
            .and_then(|message| message.as_string())
            .unwrap_or_else(|| format!("{error:?}"))
    })
}

async fn fetch_response(url: &str, mode: RequestCache) -> Result<Response, String> {
    let window = web_sys::window().ok_or("no window")?;
    let init = RequestInit::new();
    init.set_cache(mode);
    let response = JsFuture::from(window.fetch_with_str_and_init(url, &init))
        .await
        .map_err(|e| format!("{url}: {}", js_error(e)))?
        .unchecked_into::<Response>();
    if response.ok() { Ok(response) } else { Err(format!("{url}: HTTP {}", response.status())) }
}

async fn fetch_text(url: &str) -> Result<String, String> {
    let response = fetch_response(url, RequestCache::NoCache).await?;
    let text = JsFuture::from(response.text().map_err(js_error)?).await.map_err(js_error)?;
    text.as_string().ok_or_else(|| format!("{url}: not text"))
}

/// Téléchargement en flux : la progression compte les octets reçus, sans `Content-Length` (la
/// taille attendue vient du manifeste). Le cache HTTP est contourné (`no-store`) : la ressource
/// vit dans Cache Storage, pas en double dans le cache HTTP.
async fn download(resource: &Resource, mut progress: impl FnMut(u64)) -> Result<Vec<u8>, String> {
    let url = cache_key(resource);
    let response = fetch_response(&url, RequestCache::NoStore).await?;
    let body = response.body().ok_or_else(|| format!("{url}: no body"))?;
    let reader = body.get_reader().unchecked_into::<ReadableStreamDefaultReader>();

    let mut bytes = Vec::with_capacity(usize::try_from(resource.bytes).map_err(|e| e.to_string())?);
    loop {
        let chunk =
            JsFuture::from(reader.read()).await.map_err(|e| format!("{url}: {}", js_error(e)))?;
        let done =
            Reflect::get(&chunk, &"done".into()).map_err(js_error)?.as_bool().unwrap_or(true);
        if done {
            return Ok(bytes);
        }
        let value =
            Reflect::get(&chunk, &"value".into()).map_err(js_error)?.unchecked_into::<Uint8Array>();
        let start = bytes.len();
        bytes.resize(start + value.length() as usize, 0);
        value.copy_to(&mut bytes[start..]);
        progress(bytes.len() as u64);
        if bytes.len() as u64 > resource.bytes {
            // Plus que le manifeste ne l'annonce : inutile d'aller au bout.
            let _ = reader.cancel();
            return check_size(resource, bytes.len() as u64).map(|()| bytes);
        }
    }
}

async fn open_cache() -> Result<Cache, String> {
    let window = web_sys::window().ok_or("no window")?;
    let storage = Reflect::get(&window, &"caches".into()).map_err(js_error)?;
    if storage.is_undefined() || storage.is_null() {
        return Err("Cache Storage is not available in this context".to_owned());
    }
    let cache = JsFuture::from(storage.unchecked_into::<CacheStorage>().open(CACHE_NAME))
        .await
        .map_err(js_error)?;
    Ok(cache.unchecked_into::<Cache>())
}

async fn read_cached(cache: &Cache, key: &str) -> Result<Option<Vec<u8>>, String> {
    let hit = JsFuture::from(cache.match_with_str(key)).await.map_err(js_error)?;
    if hit.is_undefined() {
        return Ok(None);
    }
    let buffer = JsFuture::from(hit.unchecked_into::<Response>().array_buffer().map_err(js_error)?)
        .await
        .map_err(js_error)?;
    Ok(Some(Uint8Array::new(&buffer).to_vec()))
}

async fn put_cached(cache: &Cache, key: &str, bytes: &[u8]) -> Result<(), String> {
    let body = Uint8Array::from(bytes);
    let response = Response::new_with_opt_buffer_source(Some(&body)).map_err(js_error)?;
    JsFuture::from(cache.put_with_str(key, &response)).await.map_err(js_error)?;
    Ok(())
}

/// Supprime du cache les entrées que le manifeste ne cite pas.
async fn prune(cache: &Cache, manifest: &Manifest) -> Result<(), String> {
    let keys = JsFuture::from(cache.keys()).await.map_err(js_error)?;
    let urls: Vec<String> = Array::from(&keys)
        .iter()
        .filter_map(|request| Reflect::get(&request, &"url".into()).ok()?.as_string())
        .collect();
    for url in stale_entries(&urls, manifest) {
        JsFuture::from(cache.delete_with_str(url)).await.map_err(js_error)?;
    }
    Ok(())
}

// === Hôte partagé et minuteur de génération ===

/// Le moteur, partagé entre l'écran et le minuteur de génération : même fil, donc `Rc<RefCell>`.
#[derive(Clone)]
pub struct HostHandle(Rc<RefCell<LiveHost>>);

impl HostHandle {
    pub(super) fn new(host: LiveHost) -> Self {
        Self(Rc::new(RefCell::new(host)))
    }

    /// Accès exclusif à l'hôte le temps de `f`.
    pub fn with<R>(&self, f: impl FnOnce(&mut LiveHost) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
}

/// Le minuteur de génération ; s'arrête quand on le lâche.
pub struct Pump {
    worker: Worker,
    url: String,
    _on_tick: Closure<dyn FnMut(MessageEvent)>,
}

/// Lance un Worker qui poste un message toutes les [`TICK_MS`] ms ; chaque message appelle
/// `LiveHost::pump` sur le fil de la page, qui avance aussi l'enregistreur. Le minuteur est dans
/// le Worker, pas dans `requestAnimationFrame` : l'onglet masqué ne rend plus (S1 : lecture et
/// écriture figées à la mesure 8 avec la génération dans la boucle de l'interface).
pub fn start_pump(host: &HostHandle) -> Result<Pump, String> {
    let script = format!("setInterval(() => postMessage(0), {TICK_MS});");
    let options = BlobPropertyBag::new();
    options.set_type("text/javascript");
    let blob =
        Blob::new_with_str_sequence_and_options(&Array::of1(&JsValue::from_str(&script)), &options)
            .map_err(js_error)?;
    let url = Url::create_object_url_with_blob(&blob).map_err(js_error)?;
    let worker = Worker::new(&url).map_err(js_error)?;

    let host = host.clone();
    let on_tick = Closure::<dyn FnMut(MessageEvent)>::new(move |_| {
        host.with(|host| host.pump());
    });
    worker.set_onmessage(Some(on_tick.as_ref().unchecked_ref()));
    Ok(Pump { worker, url, _on_tick: on_tick })
}

impl Drop for Pump {
    fn drop(&mut self) {
        self.worker.terminate();
        let _ = Url::revoke_object_url(&self.url);
    }
}

// === Page ===

fn boot_element() -> Option<web_sys::Element> {
    web_sys::window()?.document()?.get_element_by_id("boot")
}

/// Démarre eframe sur le canvas de la page ; la page de démarrage (HTML) s'efface dès que
/// l'application tourne, et montre l'erreur si elle ne démarre pas.
pub fn run() {
    // Journal dans la console du navigateur.
    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    let settings = settings().unwrap_or_else(|message| {
        log::warn!("{message}");
        Settings { buffer_frames: Some(DEFAULT_BUFFER_FRAMES) }
    });
    spawn_local(async move {
        let result = start_app(settings).await;
        match (result, boot_element()) {
            (Ok(()), Some(boot)) => boot.remove(),
            (Err(message), Some(boot)) => {
                boot.set_text_content(Some(&format!("Could not start: {message}")))
            }
            (Err(message), None) => log::error!("could not start: {message}"),
            (Ok(()), None) => {}
        }
    });
}

async fn start_app(settings: Settings) -> Result<(), String> {
    let canvas = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("studio_canvas"))
        .ok_or("no canvas `studio_canvas` in the page")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| "`studio_canvas` is not a canvas")?;
    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(move |cc| Ok(Box::new(StudioApp::new(cc, settings)))),
        )
        .await
        .map_err(js_error)
}
