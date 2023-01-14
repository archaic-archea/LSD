#![no_std]
#![no_main]
#![feature(pointer_byte_offsets)]
#![feature(integer_atomics)]
#![feature(core_intrinsics)]
#![feature(unboxed_closures)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]

use limine::{LimineMemmapRequest, LimineHhdmRequest, LimineSmpRequest};

pub mod interrupts;
pub mod drivers;
pub mod gdt;
pub mod paging;
pub mod memory;
pub mod bitmap;

pub static SMP: LimineSmpRequest = LimineSmpRequest::new(0).flags(1);
static MMR: LimineMemmapRequest = LimineMemmapRequest::new(0);
static HHDM: LimineHhdmRequest = LimineHhdmRequest::new(0);

pub fn init() {
    let memorymap = MMR.get_response().get().unwrap();

    drivers::output::terminal::init();
    //println!("Terminal initialized");

    gdt::init();
    //println!("GDT initialized");
    interrupts::init_idt();
    //println!("Interrupts initialized");

    memory::init_sect_manager();
    //println!("section manager initialized");
    memory::init_page_manager();
    //println!("Page manager initialized");

    for entry in memorymap.memmap() {
        println!("{:?} 0x{:x}", entry.typ, entry.base);
    }

    paging::paging_init();
    //println!("Paging initialized");
}

/// Efficient loop
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}