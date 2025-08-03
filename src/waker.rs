use std::{
    sync::{Arc, Mutex},
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::trace::CaptureTrace;

pub(crate) struct TracingTimeoutWakerInner<C: CaptureTrace + Send + 'static> {
    active_traces: Mutex<Vec<Option<C::Trace>>>,
    capture: C,
    inner_waker: Waker,
}

impl<C: CaptureTrace + Send + 'static> TracingTimeoutWakerInner<C> {
    pub(crate) fn new(capture: C, inner_waker: Waker) -> Arc<Self> {
        Arc::new(Self {
            active_traces: Mutex::new(Vec::with_capacity(4)),
            capture,
            inner_waker,
        })
    }

    pub(crate) fn traces(&self) -> Vec<C::Trace> {
        std::mem::take(&mut *self.active_traces.lock().unwrap())
            .into_iter()
            .flatten()
            .collect()
    }
}

pub(crate) struct TracingTimeoutWaker<C: CaptureTrace + Send + 'static> {
    inner: Arc<TracingTimeoutWakerInner<C>>,
    idx: Option<usize>,
}

impl<C> TracingTimeoutWaker<C>
where
    C: CaptureTrace + Send + 'static,
{
    fn vtable() -> &'static RawWakerVTable {
        // SAFETY:
        // 1. TracingTimeoutWaker is a raw pointer to `Box::into_raw`
        // 2. Each raw pointer is a valid pointer to Box:
        //    - it is unique
        //    - it has valid lifetime
        &RawWakerVTable::new(
            Self::raw_clone,
            Self::raw_wake,
            Self::raw_wake_by_ref,
            Self::raw_drop,
        )
    }

    pub(crate) fn new_std_waker(inner: Arc<TracingTimeoutWakerInner<C>>) -> Waker {
        let data = Box::into_raw(Box::new(Self { inner, idx: None }));
        // SAFETY: (see comment for `vtable` function)
        // `data` is a valid pointer to Box as it was just obtained from `Box::into_raw`
        unsafe { Waker::new(data as *const (), Self::vtable()) }
    }

    #[allow(
        clippy::unnecessary_box_returns,
        reason = "Box<Self> is necessary for correctness"
    )]
    fn clone(&self) -> Box<Self> {
        let trace = self.inner.capture.capture();
        let idx = {
            let mut traces = self.inner.active_traces.lock().unwrap();
            let idx = traces.len();
            traces.push(Some(trace));
            idx
        };
        Box::new(Self {
            inner: self.inner.clone(),
            idx: Some(idx),
        })
    }

    unsafe fn raw_clone(data: *const ()) -> RawWaker {
        // SAFETY: (see comment for `vtable` function)
        // - `data` is a valid pointer to Box due to prerequisites of vtable
        // - every raw Box pointer is a pointer to stored struct
        let this = unsafe { &*data.cast::<Self>() };
        let cloned = this.clone();
        RawWaker::new(Box::into_raw(cloned) as *const (), Self::vtable())
    }

    fn wake(&self) {
        self.inner.inner_waker.wake_by_ref();
    }
    unsafe fn raw_wake(data: *const ()) {
        // SAFETY: (see comment for `vtable` function)
        // - `data` is a valid pointer to Box due to prerequisites of vtable
        // - every raw Box pointer is a pointer to stored struct
        let this = unsafe { &*data.cast::<Self>() };
        this.wake();
    }
    fn wake_by_ref(&self) {
        self.inner.inner_waker.wake_by_ref();
    }
    unsafe fn raw_wake_by_ref(data: *const ()) {
        // SAFETY: (see comment for `vtable` function)
        // - `data` is a valid pointer to Box due to prerequisites of vtable
        // - every raw Box pointer is a pointer to stored struct
        let this = unsafe { &*data.cast::<Self>() };
        this.wake_by_ref();
    }
    unsafe fn raw_drop(data: *const ()) {
        // SAFETY: (see comment for `vtable` function)
        // - `data` is a valid pointer to Box due to prerequisites of vtable
        // - the argument of `raw_drop` is a unique reference
        let this = unsafe { Box::<Self>::from_raw(data as *mut Self) };
        drop(this);
    }
}

impl<C: CaptureTrace + Send + 'static> Drop for TracingTimeoutWaker<C> {
    fn drop(&mut self) {
        let mut traces = self.inner.active_traces.lock().unwrap();
        if let Some(idx) = self.idx {
            // `TimeoutFuture::poll` takes data from `traces.traces`
            if !traces.is_empty() {
                traces[idx] = None;
            }
        }
    }
}
