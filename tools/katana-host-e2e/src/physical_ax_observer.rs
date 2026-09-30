#[cfg(target_os = "macos")]
use crate::{AxObserverError, physical_bootstrap_types::{AxApplicationElement, KatanAChild}};

const NOTIFICATION_WAIT_SECONDS: f64 = 30.0;

#[cfg(target_os = "macos")]
pub struct AxWindowCreatedObserver {
    observer: objc2_core_foundation::CFRetained<objc2_application_services::AXObserver>,
    run_loop: objc2_core_foundation::CFRetained<objc2_core_foundation::CFRunLoop>,
    source: objc2_core_foundation::CFRetained<objc2_core_foundation::CFRunLoopSource>,
    observed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    callback_state: std::ptr::NonNull<CallbackState>,
}

#[cfg(target_os = "macos")]
impl AxWindowCreatedObserver {
    pub fn register(
        application: &AxApplicationElement,
        child: &KatanAChild,
    ) -> Result<Self, AxObserverError> {
        Self::register_with_expectation(application, child, AxWindowExpectation::AnyWindow)
    }

    pub fn register_native_dialog(
        application: &AxApplicationElement,
        child: &KatanAChild,
    ) -> Result<Self, AxObserverError> {
        Self::register_with_expectation(application, child, AxWindowExpectation::NativeDialog)
    }

    fn register_with_expectation(
        application: &AxApplicationElement,
        child: &KatanAChild,
        expectation: AxWindowExpectation,
    ) -> Result<Self, AxObserverError> {
        use objc2_application_services::{AXError, AXObserver};
        use objc2_core_foundation::{CFRunLoop, CFString};
        use std::ffi::c_void;
        use std::ptr::{NonNull, null_mut};

        let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let callback_state = Box::new(CallbackState {
            observed: observed.clone(),
            expectation,
        });
        let callback_state = Box::into_raw(callback_state);
        let mut raw = null_mut();
        let status = unsafe {
            AXObserver::create(
                child.process_id() as i32,
                Some(window_created_callback),
                NonNull::from(&mut raw),
            )
        };
        if status != AXError::Success {
            unsafe { drop(Box::from_raw(callback_state)) };
            return Err(AxObserverError::CreateFailed);
        }
        let Some(raw) = NonNull::new(raw) else {
            unsafe { drop(Box::from_raw(callback_state)) };
            return Err(AxObserverError::CreateFailed);
        };
        let observer = unsafe { objc2_core_foundation::CFRetained::from_raw(raw) };
        let notification = CFString::from_str("AXWindowCreated");
        let status = unsafe {
            observer.add_notification(
                &application.element,
                &notification,
                callback_state.cast::<c_void>(),
            )
        };
        if status != AXError::Success {
            unsafe { drop(Box::from_raw(callback_state)) };
            return Err(AxObserverError::RegisterFailed);
        }
        let Some(run_loop) = CFRunLoop::current() else {
            unsafe { drop(Box::from_raw(callback_state)) };
            return Err(AxObserverError::RunLoopUnavailable);
        };
        let source = unsafe { observer.run_loop_source() };
        run_loop.add_source(Some(&source), unsafe {
            objc2_core_foundation::kCFRunLoopDefaultMode
        });
        Ok(Self {
            observer,
            run_loop,
            source,
            observed,
            callback_state: unsafe { NonNull::new_unchecked(callback_state) },
        })
    }

    pub fn wait_for_notification(self) -> Result<(), AxObserverError> {
        let run_result = objc2_core_foundation::CFRunLoop::run_in_mode(
            unsafe { objc2_core_foundation::kCFRunLoopDefaultMode },
            NOTIFICATION_WAIT_SECONDS,
            true,
        );
        let observed = self.observed.load(std::sync::atomic::Ordering::Acquire);
        let _ = &self.observer;
        if observed {
            Ok(())
        } else {
            let _ = run_result;
            Err(AxObserverError::NotificationNotObserved)
        }
    }

    pub fn wait_for_existing_window_or_notification(
        self,
        application: &AxApplicationElement,
    ) -> Result<(), AxObserverError> {
        if application
            .has_existing_window()
            .map_err(|_| AxObserverError::ExistingWindowQueryFailed)?
        {
            return Ok(());
        }
        self.wait_for_notification()
    }
}

#[cfg(target_os = "macos")]
impl Drop for AxWindowCreatedObserver {
    fn drop(&mut self) {
        self.run_loop.remove_source(Some(&self.source), unsafe {
            objc2_core_foundation::kCFRunLoopDefaultMode
        });
        unsafe { drop(Box::from_raw(self.callback_state.as_ptr())) };
    }
}

#[cfg(target_os = "macos")]
struct CallbackState {
    observed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    expectation: AxWindowExpectation,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
enum AxWindowExpectation {
    AnyWindow,
    NativeDialog,
}

#[cfg(target_os = "macos")]
unsafe extern "C-unwind" fn window_created_callback(
    _observer: std::ptr::NonNull<objc2_application_services::AXObserver>,
    element: std::ptr::NonNull<objc2_application_services::AXUIElement>,
    _notification: std::ptr::NonNull<objc2_core_foundation::CFString>,
    refcon: *mut std::ffi::c_void,
) {
    let state = unsafe { &*(refcon.cast::<CallbackState>()) };
    if !notification_matches_expectation(element, state.expectation) {
        return;
    }
    state
        .observed
        .store(true, std::sync::atomic::Ordering::Release);
    if let Some(run_loop) = objc2_core_foundation::CFRunLoop::current() {
        run_loop.stop();
    }
}

#[cfg(target_os = "macos")]
fn notification_matches_expectation(
    element: std::ptr::NonNull<objc2_application_services::AXUIElement>,
    expectation: AxWindowExpectation,
) -> bool {
    match expectation {
        AxWindowExpectation::AnyWindow => true,
        AxWindowExpectation::NativeDialog => crate::physical_ax_dialog::is_native_dialog(element),
    }
}
