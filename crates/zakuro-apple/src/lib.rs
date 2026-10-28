//! Zakuro behind a C interface, which the iOS app drives. the app links the
//! code 3dsrecomp made for one game beside this library, so there is one app
//! per game, and calls these from the thread that runs the game, bar the two
//! that take input, which may come from any.

use std::ffi::{c_char, c_void, CStr};
use std::ptr::addr_of;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use zakuro_common::Screen;
use zakuro_core::recompiled::Linked;
use zakuro_core::services::hid::{InputState, PadState};
use zakuro_core::{loader, Config, FrameOutcome, System};

// the tables 3dsrecomp writes beside the code of the game
extern "C" {
    static recomp_abi: u32;
    static recomp_generation: u32;
    static recomp_entry_count: u32;
    static recomp_entries: u8;
    static recomp_module_count: u32;
    static recomp_modules: u8;
    static recomp_origins: u8;
}

/// the buttons held, as PadState's bits, and where the screen is touched,
/// x in the top half and y in the bottom, or NO_TOUCH.
static BUTTONS: AtomicU32 = AtomicU32::new(0);
static TOUCH: AtomicU32 = AtomicU32::new(NO_TOUCH);
const NO_TOUCH: u32 = u32::MAX;

pub struct Game {
    system: System,
    /// the picture handed out last, kept until the next is asked for.
    picture: Option<Arc<Vec<u8>>>,
}

/// starts the game at rom, whose program id is program_id, with its saves in
/// data_dir, drawn on the GPU at resolution times the console's. null when
/// it can't start.
///
/// # Safety
/// rom and data_dir are NUL-terminated strings.
#[no_mangle]
pub unsafe extern "C" fn zakuro_open(rom: *const c_char, data_dir: *const c_char, program_id: u64, resolution: u32) -> *mut Game {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).try_init();
    // SAFETY: the caller's strings, and the tables the app links in
    let (rom, data_dir, code) = unsafe {
        let code = Linked::new(
            program_id,
            recomp_abi,
            addr_of!(recomp_entries) as *const c_void,
            recomp_entry_count,
            addr_of!(recomp_modules) as *const c_void,
            recomp_module_count,
        )
        .with_origins(recomp_generation, addr_of!(recomp_origins) as *const c_void);
        (CStr::from_ptr(rom).to_string_lossy().into_owned(), CStr::from_ptr(data_dir).to_string_lossy().into_owned(), code)
    };
    let config = Config {
        linked: Some(code),
        data_dir: Some(data_dir.into()),
        hardware_renderer: true,
        resolution: resolution.max(1),
        ..Config::default()
    };
    match loader::load(&rom, config) {
        Ok(system) => Box::into_raw(Box::new(Game { system, picture: None })),
        Err(error) => {
            log::error!("could not open {rom}, {error}");
            std::ptr::null_mut()
        }
    }
}

/// runs one frame of the game with the input given, false once it ended.
///
/// # Safety
/// game came from zakuro_open and isn't used by another thread.
#[no_mangle]
pub unsafe extern "C" fn zakuro_run_frame(game: *mut Game) -> bool {
    // SAFETY: as the caller says
    let game = unsafe { &mut *game };
    let touch = TOUCH.load(Ordering::Relaxed);
    let input = InputState {
        buttons: PadState::from_bits_truncate(BUTTONS.load(Ordering::Relaxed)),
        touch: (touch != NO_TOUCH).then_some(((touch >> 16) as u16, touch as u16)),
        ..InputState::default()
    };
    game.system.set_input(input);
    matches!(game.system.run_frame(), FrameOutcome::Completed)
}

/// the picture of a screen, 0 for the top and 1 for the bottom, as RGBA
/// bytes, its size in width and height. it is good until the next call.
///
/// # Safety
/// game came from zakuro_open and isn't used by another thread, width and
/// height can be written.
#[no_mangle]
pub unsafe extern "C" fn zakuro_screen(game: *mut Game, screen: u32, width: *mut u32, height: *mut u32) -> *const u8 {
    // SAFETY: as the caller says
    let (game, width, height) = unsafe { (&mut *game, &mut *width, &mut *height) };
    let (pixels, w, h) = game.system.read_screen_scaled(if screen == 0 { Screen::Top } else { Screen::Bottom });
    (*width, *height) = (w, h);
    game.picture.insert(pixels).as_ptr()
}

/// the buttons held from now on, as the console's bits, A in the lowest.
#[no_mangle]
pub extern "C" fn zakuro_set_buttons(buttons: u32) {
    BUTTONS.store(buttons, Ordering::Relaxed);
}

/// the touch screen is touched at x and y in its own pixels, 320 by 240, or
/// not when released is true.
#[no_mangle]
pub extern "C" fn zakuro_set_touch(x: u16, y: u16, released: bool) {
    TOUCH.store(if released { NO_TOUCH } else { (x as u32) << 16 | y as u32 }, Ordering::Relaxed);
}

/// ends the game.
///
/// # Safety
/// game came from zakuro_open and isn't used again.
#[no_mangle]
pub unsafe extern "C" fn zakuro_close(game: *mut Game) {
    // SAFETY: as the caller says
    drop(unsafe { Box::from_raw(game) });
}
