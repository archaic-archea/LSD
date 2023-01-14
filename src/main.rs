#![no_std]
#![no_main]

mod io;

use limine::{LimineBootInfoRequest, LimineHhdmRequest, LimineMemmapRequest, LimineFramebufferRequest};

static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);

/// TODO: Enable HHDM
/// TODO: Use Memory map's provided physical address and map them based off HHDM response's offset

//static HHDM: LimineHhdmRequest = LimineHhdmRequest::new(0);
static MMR: LimineMemmapRequest = LimineMemmapRequest::new(0);
static FRAME: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

/// Kernel Entry Point
///
/// `_start` is defined in the linker script as the entry point for the ELF file.
/// Unless the [`Entry Point`](limine::LimineEntryPointRequest) feature is requested,
/// the bootloader will transfer control to this function.
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    println!("hello, world!");

    if let Some(bootinfo) = BOOTLOADER_INFO.get_response().get() {
        println!(
            "booted by {} v{}",
            bootinfo.name.to_str().unwrap().to_str().unwrap(),
            bootinfo.version.to_str().unwrap().to_str().unwrap(),
        );
    }

    /*let offset: u64;

    if let Some(hhdm) = HHDM.get_response().get() {
        println!(
            "Provided offset is 0x{:x}",
            hhdm.offset
        );

        offset = hhdm.offset;
    } else {
        panic!("Failed to get HHDM");
    }*/

    for entry in MMR.get_response().get().unwrap().memmap() {
        println!("type: {:?} base: 0x{:x}, len: 0x{:x}", entry.typ, entry.base, entry.len);
    }

    let frames = FRAME.get_response().get().unwrap().framebuffers();
    println!("frame location in vmem: 0x{:x}", frames[0].address.as_ptr().unwrap() as u64);
    //println!("frame location in pmem: 0x{:x}", frames[0].address.as_ptr().unwrap() as u64 - offset);

    loop {}
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    hcf();
}

/// Die, spectacularly.
pub fn hcf() -> ! {
    loop {
        core::hint::spin_loop();
    }
}