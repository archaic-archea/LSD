#![no_std]
#![no_main]
#![feature(pointer_byte_offsets)]
#![feature(integer_atomics)]
#![feature(core_intrinsics)]
#![feature(unboxed_closures)]
#![feature(const_mut_refs)]

use limine::{LimineMemmapRequest, LimineHhdmRequest, LimineSmpRequest};

pub mod drivers;
pub mod paging;
pub mod memory;
pub mod bitmap;

pub static SMP: LimineSmpRequest = LimineSmpRequest::new(0).flags(1);
static MMR: LimineMemmapRequest = LimineMemmapRequest::new(0);
static HHDM: LimineHhdmRequest = LimineHhdmRequest::new(0);

pub fn init() {
    drivers::output::terminal::init();
    println!("Terminal initialized");
    memory::init_sect_manager();
    println!("section manager initialized");
    memory::init_page_manager();
    println!("Page manager initialized");
    paging::paging_init();
    println!("Paging initialized");
}

/// Efficient loop
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}