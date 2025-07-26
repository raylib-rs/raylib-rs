use std::{
    ffi::CString,
    path::{Path, PathBuf},
    ptr::null,
};

use crate::{RaylibHandle, ffi};

#[derive(Debug, Clone)]
pub struct AutomationEventIter<'a> {
    iter: std::slice::Iter<'a, ffi::AutomationEvent>,
}
impl AutomationEventIter<'_> {
    #[must_use]
    unsafe fn new(events: *mut ffi::AutomationEvent, count: u32) -> Self {
        // No new items are being created that get dropped here, these are just changes in perspective of how to borrow-check the pointers.
        assert!(!events.is_null(), "automation event array cannot be null");
        assert!(
            events.is_aligned(),
            "automation event array must be aligned"
        );
        let iter = unsafe {
            std::slice::from_raw_parts(
                events,
                count
                    .try_into()
                    .expect("slice should not exceed usize::MAX elements"),
            )
        }
        .iter();
        Self { iter }
    }

    const fn func(e: &ffi::AutomationEvent) -> AutomationEvent {
        // This relies on the fact that `ffi::AutomationEvent` is Copy `unload_automation_event` doesn't actually do anything.
        AutomationEvent(*e)
    }
}
impl Iterator for AutomationEventIter<'_> {
    type Item = AutomationEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Self::func)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(Self::func)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(Self::func)
    }
}
impl DoubleEndedIterator for AutomationEventIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Self::func)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(Self::func)
    }
}
impl ExactSizeIterator for AutomationEventIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

make_thin_wrapper!(
    AutomationEventList,
    ffi::AutomationEventList,
    ffi::UnloadAutomationEventList,
    false
);

impl AutomationEventList {
    /// Length of the automation event list
    #[inline]
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.0.count
    }

    /// The amount of automation events that can be held in this list.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.0.capacity
    }

    /// The events held in this list.
    ///
    /// NOTE: This will copy the values into a vector.
    ///
    /// # Panics
    ///
    /// This method will panic if `self` contains more than [`usize::MAX`] events.
    #[must_use]
    pub fn events(&self) -> Vec<AutomationEvent> {
        unsafe {
            std::slice::from_raw_parts(
                self.0.events,
                self.count()
                    .try_into()
                    .expect("slice should not exceed usize::MAX elements"),
            )
        }
        .iter()
        .map(|f| AutomationEvent(*f))
        .collect()
    }

    /// An iterator over the events held in this list.
    #[must_use]
    pub fn iter(&self) -> AutomationEventIter<'_> {
        unsafe { AutomationEventIter::new(self.0.events, self.count()) }
    }

    /// Export automation events list as text file
    ///
    /// # Panics
    ///
    /// This method will panic if `file_name` contains an internal 0 byte.
    pub fn export(&self, file_name: impl AsRef<Path>) -> bool {
        let c_str = CString::new(file_name.as_ref().to_string_lossy().as_bytes())
            .expect("lossy filename should not contain an internal 0 byte");
        unsafe { ffi::ExportAutomationEventList(self.0, c_str.as_ptr()) }
    }
}

impl<'a> IntoIterator for &'a AutomationEventList {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = AutomationEventIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

make_thin_wrapper!(
    AutomationEvent,
    ffi::AutomationEvent,
    unload_automation_event,
    false
);

impl AutomationEvent {
    /// Event frame
    #[inline]
    #[must_use]
    pub const fn frame(&self) -> u32 {
        self.0.frame
    }
    /// Event type ([`AutomationEventType`])
    #[inline]
    #[must_use]
    pub const fn get_type(&self) -> u32 {
        self.0.type_
    }
    /// Event parameters (if required)
    #[inline]
    #[must_use]
    pub const fn params(&self) -> [i32; 4] {
        self.0.params
    }
}

impl AutomationEvent {
    /// Play a recorded automation event
    #[inline]
    pub fn play(&self) {
        unsafe { ffi::PlayAutomationEvent(self.0) }
    }
}

const fn unload_automation_event(_s: ffi::AutomationEvent) {
    // As far as I can tell, this is actually unloaded when UnloadAnimationEventList is called.
}

impl RaylibHandle {
    /// Load automation events list from file, NULL for empty list, capacity = `MAX_AUTOMATION_EVENTS` (16,384 by default)
    ///
    /// # Panics
    ///
    /// This method will panic if `file_name` contains an internal 0 byte.
    #[must_use]
    pub fn load_automation_event_list(&self, file_name: Option<PathBuf>) -> AutomationEventList {
        match file_name {
            Some(a) => {
                let c_str = CString::new(a.to_string_lossy().as_bytes())
                    .expect("lossy filename should not contain an internal 0 byte");
                AutomationEventList(unsafe { ffi::LoadAutomationEventList(c_str.as_ptr()) })
            }
            None => AutomationEventList(unsafe { ffi::LoadAutomationEventList(null()) }),
        }
    }
    /// Set automation event list to record to
    #[inline]
    pub fn set_automation_event_list(&self, l: &mut AutomationEventList) {
        unsafe {
            ffi::SetAutomationEventList(&raw mut l.0);
        }
    }
    /// Set automation event internal base frame to start recording
    #[inline]
    pub fn set_automation_event_base_frame(&self, b: i32) {
        unsafe { ffi::SetAutomationEventBaseFrame(b) };
    }
    /// Start recording automation events ([`AutomationEventList`] must be set)
    #[inline]
    pub fn start_automation_event_recording(&self) {
        unsafe { ffi::StartAutomationEventRecording() };
    }
    /// Stop recording automation events
    #[inline]
    pub fn stop_automation_event_recording(&self) {
        unsafe { ffi::StopAutomationEventRecording() };
    }
}
