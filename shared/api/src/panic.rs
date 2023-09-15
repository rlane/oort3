use std::io::Cursor;
use std::io::Write;
use std::panic::PanicInfo;

pub const PANIC_BUFFER_SIZE: usize = 1024;
#[no_mangle]
pub static mut PANIC_BUFFER: [u8; PANIC_BUFFER_SIZE] = [0; PANIC_BUFFER_SIZE];

pub unsafe fn install() {
    std::panic::set_hook(Box::new(panic_hook));
}

pub unsafe fn reset() {
    PANIC_BUFFER[0] = 0;
}

fn panic_hook(info: &PanicInfo) {
    unsafe {
        let location = info.location().unwrap();
        let file = location.file();
        let line = location.line();
        let column = location.column();
        let payload = info.payload().downcast_ref::<&str>().unwrap();

        const PREFIX: &str = "/tmp/oort-ai/ai/src/";
        let mut file = file.strip_prefix(PREFIX).unwrap_or(file);
        if file == "user.rs" {
            file = "lib.rs";
        }

        let mut cursor = Cursor::new(&mut PANIC_BUFFER[..]);
        let _ = write!(
            cursor,
            "ship panicked at '{}', {}:{}:{}",
            payload, file, line, column
        );
        let _ = cursor.write(&[0]);
    }
}
