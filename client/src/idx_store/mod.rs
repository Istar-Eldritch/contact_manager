use futures::{
    future::{self, Either},
    Future,
};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{
    window, IdbDatabase, IdbFactory, IdbOpenDbRequest, IdbRequestReadyState, IdbVersionChangeEvent,
};

//const MAX_SAFE_INTEGER: u64 = 9007199254740991; // 2 ^ 53
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct IdxDb {
    db: IdbDatabase,
}

impl Deref for IdxDb {
    type Target = IdbDatabase;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

struct IdxOpenDbRequest {
    _onupgradeneeded: Closure<dyn Fn(IdbVersionChangeEvent)>,
    _onanything: Option<Closure<dyn FnMut()>>,
    request: IdbOpenDbRequest,
    // onsuccess: Option<Closure<dyn FnMut()>>,
    // onerror: Option<Closure<dyn FnMut()>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl Deref for IdxOpenDbRequest {
    type Target = IdbOpenDbRequest;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl Future for IdxOpenDbRequest {
    type Output = Result<IdxDb, JsValue>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        log::debug!("Request state: {:?}", self.request.ready_state());
        match self.request.ready_state() {
            IdbRequestReadyState::Pending => {
                self.waker.lock().unwrap().replace(cx.waker().clone());

                Poll::Pending
            }
            IdbRequestReadyState::Done => {
                let result = self.request.result().map(|db| {
                    let db: IdbDatabase = db.into();
                    IdxDb { db }
                });
                Poll::Ready(result)
            }
            _ => Poll::Pending,
        }
    }
}

impl IdxDb {
    /// Open a database.
    ///
    /// # Panics
    ///
    /// This function will panic if the new version is 0.
    pub fn open(
        name: &str,
        version: u32,
        on_upgrade_needed: impl Fn(IdbVersionChangeEvent, IdxDb) + 'static,
    ) -> impl Future<Output = Result<Self, JsValue>> {
        if version == 0 {
            panic!("indexeddb version must be >= 1");
        }
        let factory: IdbFactory = window().unwrap().indexed_db().unwrap().unwrap();

        let request = match factory.open_with_u32(name, version) {
            Ok(request) => request,
            Err(e) => return Either::Right(future::err(e)),
        };

        let request_copy = request.clone();

        let _onupgradeneeded = Closure::wrap(Box::new(move |event: IdbVersionChangeEvent| {
            match request_copy.result() {
                Ok(db) => {
                    let db: IdbDatabase = db.into();
                    on_upgrade_needed(event, IdxDb { db });
                }
                Err(e) => {
                    log::error!(
                        "Error getting a handle to the db before running upgrades: {:?}",
                        e
                    )
                }
            }
        }) as Box<dyn Fn(IdbVersionChangeEvent)>);
        request.set_onupgradeneeded(Some(&_onupgradeneeded.as_ref().unchecked_ref()));

        let waker: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        let inner_waker = waker.clone();

        let onanything = Closure::once(Box::new(move || {
            if let Some(waker) = inner_waker.lock().unwrap().take() {
                waker.wake()
            }
        }));

        request.set_onsuccess(Some(onanything.as_ref().unchecked_ref()));
        request.set_onerror(Some(onanything.as_ref().unchecked_ref()));

        Either::Left(IdxOpenDbRequest {
            _onupgradeneeded,
            request,
            _onanything: Some(onanything),
            waker: waker,
        })
    }
}
