use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use crate::{println, hlt_loop, print};


pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    let error = error_code.bits();

    unsafe {
        core::arch::asm!(
            "",
            in("rax") error,
            in("rcx") error,
            in("rdx") error,
        );
    }
    hlt_loop();

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn divide_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nError code: {}", stack_frame, error_code);
}

pub extern "x86-interrupt" fn opcode_handler(
    stack_frame: InterruptStackFrame,
) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64
) {
    panic!("EXCEPTION: INVALID TSS\n{:#?}\nError code: {}", stack_frame, error_code);
}