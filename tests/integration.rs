use std::{process::Command, thread, time::Duration};

#[test]
fn receive_shutdown_signal() {
    setup();

    let mut child = Command::new(env!("CARGO_BIN_EXE_test_fixture"))
        .spawn()
        .expect("spawn test fixture binary as child process");

    thread::sleep(Duration::from_millis(500));
    kill(child.id());

    let status = child.wait().expect("wait for child process to exit");
    assert_eq!(Some(15), status.code());

    cleanup();
}

#[cfg(unix)]
fn setup() {
    // no setup needed on unix systems.
}

#[cfg(windows)]
fn setup() {
    use winapi::{
        shared::minwindef::TRUE,
        um::{consoleapi, wincon},
    };

    // Have to create a new console as the CTR+C command would not just kill the child and current
    // process, but the test runner as well.

    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        cleanup();
        (default)(info);
    }));

    let success = unsafe { wincon::FreeConsole() };
    assert!(success == TRUE, "must free current console");

    let success = unsafe { consoleapi::AllocConsole() };
    assert!(success == TRUE, "must allocate new console");
}

#[cfg(unix)]
fn cleanup() {
    // no cleanup needed on unix systems.
}

#[cfg(windows)]
fn cleanup() {
    use winapi::{shared::minwindef::TRUE, um::wincon};

    // Re-attach to the test runner's console so we get proper console output. If we don't, the
    // tests would pass but the console wouldn't print the test case result.

    let success = unsafe { wincon::FreeConsole() };
    assert!(success == TRUE, "must free current console");

    let success = unsafe { wincon::AttachConsole(wincon::ATTACH_PARENT_PROCESS) };
    assert!(success == TRUE, "must attach to parent console");
}

#[cfg(unix)]
fn kill(pid: u32) {
    use nix::{
        sys::signal::{self, Signal},
        unistd::Pid,
    };

    let result = signal::kill(Pid::from_raw(pid as _), Signal::SIGINT);
    assert!(result.is_ok(), "must kill the child process");
}

#[cfg(windows)]
fn kill(pid: u32) {
    use winapi::{
        shared::minwindef::{BOOL, DWORD, TRUE},
        um::{consoleapi, wincon},
    };

    unsafe extern "system" fn handler(event: DWORD) -> BOOL {
        (event == wincon::CTRL_C_EVENT) as _
    }

    // Set a custom CTRL+C handler for the current process, as sending the event will trigger all
    // processes, not just the child process (despite giving the child's PID).

    let success = unsafe { consoleapi::SetConsoleCtrlHandler(Some(handler), TRUE) };
    assert!(success == TRUE, "must overwrite ctrl handler");

    let success = unsafe { wincon::GenerateConsoleCtrlEvent(wincon::CTRL_C_EVENT, pid as _) };
    assert!(success == TRUE, "must kill the child process");
}
