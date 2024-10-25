#![cfg(feature = "crash-handler")]
use crash_handler::{CrashContext, CrashEventResult, CrashHandler};
use std::sync::LazyLock;

static HANDLER: LazyLock<Result<CrashHandler, crash_handler::Error>> = LazyLock::new(attach);

/// Register the test crash handler
pub fn register() {
    LazyLock::force(&HANDLER);
}

fn attach() -> Result<CrashHandler, crash_handler::Error> {
    crash_handler::CrashHandler::attach(unsafe { crash_handler::make_single_crash_event(handler) })
}

fn handler(_context: &CrashContext) -> CrashEventResult {
    use crash_handler::{debug_print, write_stderr};

    debug_print!("\n === \x1b[91mFLECS-RUST CRASH HANDLER\x1b[0m === \n");

    #[cfg(target_os = "windows")]
    {
        // Try to output some extra Windows-specific information
        write_stderr(format!("Exception code: 0x{:08x}\n", _context.exception_code).leak());

        let er = unsafe { _context.exception_pointers.as_ref() }
            .and_then(|ep| unsafe { ep.ExceptionRecord.as_ref() });

        match _context.exception_code as u32 {
            // access violation
            0xC0000005 => {
                if let Some(er) = er {
                    if er.NumberParameters >= 2 {
                        let info = format!(
                            "Exception: access violation (tried to {} memory at {:#p})\n",
                            match er.ExceptionInformation[0] {
                                0 => "read",
                                1 => "write",
                                8 => "execute",
                                _ => "access",
                            },
                            er.ExceptionInformation[1] as *const u8
                        )
                        .leak();
                        write_stderr(info);
                    }
                }
            }
            // stack overflow
            0xC00000FD => {
                debug_print!("Exception: stack overflow, skipping backtrace");

                if let Some(er) = er {
                    write_stderr(
                        format!("Instruction pointer: {:#p}\n", er.ExceptionAddress).leak(),
                    );
                }
                return false.into();
            }
            _ => {}
        }
    }

    debug_print!("attempting to generate backtrace...");

    let bts = format!("{:#}", std::backtrace::Backtrace::force_capture()).leak();
    write_stderr(bts);

    false.into()
}
