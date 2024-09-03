use crate::io;
use negative_impl::negative_impl;
use windows::{
    core::Owned,
    Win32::{
        Foundation::{HANDLE, INVALID_HANDLE_VALUE},
        Storage::FileSystem::SetFileCompletionNotificationModes,
        System::{WindowsProgramming::FILE_SKIP_SET_EVENT_ON_HANDLE, IO::CreateIoCompletionPort},
    },
};

/// The I/O completion port is used to notify the I/O driver that an I/O operation has completed.
/// It must be associated with each file/socket/handle that is capable of asynchronous I/O. We do
/// not expose this in the public API, just use it internally.
///
/// Each async worker thread has a single I/O completion port used for all I/O operations. This type
/// is single-threaded to prevent accidental sharing between threads.
#[derive(Debug)]
pub(crate) struct CompletionPort {
    handle: Owned<HANDLE>,
}

impl CompletionPort {
    pub(crate) fn new() -> Self {
        // SAFETY: We wrap it in Owned, so it is released on drop. Nothing else to worry about.
        unsafe {
            let handle = Owned::new(CreateIoCompletionPort(
                INVALID_HANDLE_VALUE,
                HANDLE::default(),
                0, // We do not use the completion key.
                1, // Only to be used by 1 thread (the current thread).
            ).expect("creating an I/O completion port should never fail unless the OS is critically out of resources"));

            Self { handle }
        }
    }

    /// Binds an I/O primitive to the completion port when provided a handle to the I/O primitive.
    /// This causes notifications from that I/O primitive to arrive at the completion port.
    pub(crate) fn bind(&self, handle: &Owned<HANDLE>) -> io::Result<()> {
        // SAFETY: We only pass in handles, which are safe to pass even if invalid (-> error)
        //         We ignore the return value, because it is the same as our own handle on success.
        unsafe {
            CreateIoCompletionPort(**handle, *self.handle, 0, 1)?;
        }

        // We FILE_SKIP_SET_EVENT_ON_HANDLE
        // Why do we do this: https://devblogs.microsoft.com/oldnewthing/20200221-00/?p=103466/
        unsafe {
            SetFileCompletionNotificationModes(**handle, FILE_SKIP_SET_EVENT_ON_HANDLE as u8)?;
        }

        // TODO: To support immediate completions, remember to take it out of the deferred queue.
        // See https://github.com/svens/pal/blob/f84e64eabe1ad7ed872bb27dbc132b8f763251f2/pal/net/__bits/socket_windows.cpp#L108-L112

        Ok(())
    }

    pub(crate) fn handle(&self) -> HANDLE {
        *self.handle
    }
}

#[negative_impl]
impl !Send for CompletionPort {}
#[negative_impl]
impl !Sync for CompletionPort {}
