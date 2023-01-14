#![no_std]
#![no_main]

use core::ops::Deref;

use limine::LimineSmpInfo;
//use limine::LimineBootInfoRequest;
use lsd_limine::{*, drivers::output::terminal::shift};

//static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);

/// TODO: Use Memory map's provided physical address and map them based off HHDM response's offset

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    lsd_limine::init();

    println!("Thingy!");

    let smp = SMP.get_response().get().unwrap();

    for i in 0..smp.cpu_count {
        let cpus = &smp.cpus;
        
        let core = unsafe {cpus.deref().as_ptr().add(i as usize)};

        unsafe {
            //(*core).goto_address = thread_main;
        }
    }

    hlt_loop()
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    println!("{:#?}", info);
    hlt_loop()
}

pub extern "C" fn thread_main(info_ptr: *const LimineSmpInfo) -> ! {
    let info = unsafe {&*info_ptr};

    println!("Core started\n{:#?}", info);

    loop {}
}