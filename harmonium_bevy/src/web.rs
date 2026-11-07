//! Pompage indépendant du rendu et reprise synchrone dans le geste DOM.
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    time::Duration,
};

use harmonium::LiveHost;
use js_sys::Array;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Blob, BlobPropertyBag, Event, MessageEvent, Url, Worker};

pub(super) struct WebDriver {
    worker: Worker,
    url: String,
    _tick: Closure<dyn FnMut(MessageEvent)>,
    gesture: Closure<dyn FnMut(Event)>,
    activated: Rc<Cell<bool>>,
    paused_target: Rc<Cell<bool>>,
    meters: Rc<RefCell<BTreeMap<usize, usize>>>,
}

impl WebDriver {
    pub(super) fn new(host: Rc<RefCell<LiveHost>>, interval: Duration) -> Result<Self, String> {
        let options = BlobPropertyBag::new();
        options.set_type("text/javascript");
        let script = format!("setInterval(() => postMessage(0), {});", interval.as_millis().max(1));
        let blob = Blob::new_with_str_sequence_and_options(
            &Array::of1(&JsValue::from_str(&script)),
            &options,
        )
        .map_err(js_error)?;
        let url = Url::create_object_url_with_blob(&blob).map_err(js_error)?;
        let worker = match Worker::new(&url) {
            Ok(worker) => worker,
            Err(error) => {
                let _ = Url::revoke_object_url(&url);
                return Err(js_error(error));
            }
        };
        let meters = Rc::new(RefCell::new(BTreeMap::new()));
        let pump_host = host.clone();
        let pump_meters = meters.clone();
        let tick = Closure::new(move |_: MessageEvent| {
            let mut host = pump_host.borrow_mut();
            let snapshots = host.pump();
            let current_bar = host.transport_position().bar;
            let mut meters = pump_meters.borrow_mut();
            meters.retain(|&bar, _| bar >= current_bar);
            for snapshot in snapshots.into_iter().filter(|snapshot| snapshot.index >= current_bar) {
                meters.insert(snapshot.index, usize::from(snapshot.pulse.steps_per_pulse));
            }
        });
        worker.set_onmessage(Some(tick.as_ref().unchecked_ref()));
        let activated = Rc::new(Cell::new(false));
        let paused_target = Rc::new(Cell::new(false));
        let active = activated.clone();
        let paused = paused_target.clone();
        let failed = Cell::new(false);
        let gesture = Closure::new(move |_: Event| {
            if !active.get() && !paused.get() {
                match host.borrow_mut().resume() {
                    Ok(()) => active.set(true),
                    Err(error) if !failed.replace(true) => {
                        bevy::log::error!("Harmonium : reprise au geste refusée : {error}")
                    }
                    Err(_) => {}
                }
            }
        });
        let driver = Self { worker, url, _tick: tick, gesture, activated, paused_target, meters };
        let window = web_sys::window().ok_or("fenêtre navigateur absente")?;
        for name in ["pointerdown", "keydown", "touchend"] {
            window
                .add_event_listener_with_callback(name, driver.gesture.as_ref().unchecked_ref())
                .map_err(js_error)?;
        }
        Ok(driver)
    }

    pub(super) fn activated(&self) -> bool {
        self.activated.get()
    }
    pub(super) fn set_paused_target(&self, paused: bool) {
        self.paused_target.set(paused);
    }
    pub(super) fn take_meters(&self) -> BTreeMap<usize, usize> {
        std::mem::take(&mut *self.meters.borrow_mut())
    }
}

impl Drop for WebDriver {
    fn drop(&mut self) {
        self.worker.terminate();
        self.worker.set_onmessage(None);
        let _ = Url::revoke_object_url(&self.url);
        if let Some(window) = web_sys::window() {
            for name in ["pointerdown", "keydown", "touchend"] {
                let _ = window.remove_event_listener_with_callback(
                    name,
                    self.gesture.as_ref().unchecked_ref(),
                );
            }
        }
    }
}

fn js_error(error: JsValue) -> String {
    error.as_string().unwrap_or_else(|| format!("{error:?}"))
}
