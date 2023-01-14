use x86_64::PhysAddr;

use crate::{bitmap::BitMap, println, paging::AddrForm};
//use crate::println;

#[derive(Copy, Clone)]
pub struct Sect {
    pub base: *mut u8,
    pub size: usize,
}

impl Sect {
    pub const fn null() -> Sect {
        Sect { 
            base: 0 as *mut u8, 
            size: 0
        }
    }

    pub fn new(size: usize, base: *mut u8) -> Sect {
        Sect {
            base,
            size
        }
    }
}

#[cfg(not(features = "extended-section-manager"))]
pub const SECTION_MAX: usize = 16;
#[cfg(features = "extended-section-manager")]
pub const SECTION_MAX: usize = 32;

pub struct SectManager {
    sections: [Option<Sect>; SECTION_MAX],
    bitmap_buf: [u8; SECTION_MAX / 8]
}

pub static mut SECTION_MANAGER: SectManager = SectManager::null();

impl SectManager {
    pub const fn null() -> SectManager {
        SectManager {
            sections: [None; SECTION_MAX],
            bitmap_buf: [0; SECTION_MAX / 8]
        }
    }

    pub fn add_sect(&mut self, new_sect: Sect) {
        for i in 0..SECTION_MAX {
            match self.sections[i] {
                None => {
                    self.sections[i] = Some(new_sect);
                    return
                }
                _ => ()
            }
        }

        panic!("Failed to add new section, not enough room, consider enabling extened-section-manager feature");
    }

    pub fn req_sect(&mut self) -> (Sect, usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        for i in 0..self.sect_count() as usize {
            match mybitmap.get_bool(i) {
                true => (),
                false => {
                    match self.sections[i]{
                        None => (),
                        Some(val) => {
                            mybitmap.set_bool(i, true);
                            return (val, i);
                        }
                    }
                }
            }
        }

        panic!("No available sections");
    }

    pub fn req_large_sect(&mut self) -> (Sect, usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        let mut biggest = (Sect::null(), 0);

        for i in 0..self.sect_count() as usize {
            match mybitmap.get_bool(i) {
                true => (),
                false => {
                    match self.sections[i]{
                        None => (),
                        Some(val) => {
                            if val.size > biggest.0.size {
                                biggest = (val, i);
                            }
                        }
                    }
                }
            }
        }

        mybitmap.set_bool(biggest.1, true);
        return (biggest.0, biggest.1);
    }

    pub fn req_sect_size(&mut self, size: usize) -> (Sect, usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        let mut closest = (Sect::null(), 0, usize::MAX);

        for i in 0..self.sect_count() as usize {
            match mybitmap.get_bool(i) {
                true => (),
                false => {
                    match self.sections[i]{
                        None => (),
                        Some(val) => {
                            let difference = (val.size as isize) - (size as isize);

                            if difference == 0 {
                                mybitmap.set_bool(i, true);

                                return (val, i);
                            }

                            if difference > 0 {
                                let absolute = size.abs_diff(val.size);

                                if absolute < closest.2 {
                                    closest = (val, i, absolute);
                                }
                            }
                        }
                    }
                }
            }
        }

        mybitmap.set_bool(closest.1, true);
        return (closest.0, closest.1);
    }

    //unsafe because if called while section is in use, and then the section is requested, the section will be cleared
    pub unsafe fn ret_sect(&mut self, index: usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        mybitmap.set_bool(index, false);
    }

    //unsafe because if called while section is in use, could delete important data
    pub unsafe fn empty_section(&mut self, index: usize) {
        let section = self.sections[index].expect("Section does not exist");

        for i in 0..section.size {
            *section.base.offset(i as isize) = 0;
        }
    }

    pub fn sect_count(&self) -> u8 {
        let mut total = 0;

        for i in 0..SECTION_MAX {
            match self.sections[i] {
                Some(_val) => total += 1,
                _ => ()
            }
        }

        total
    }

    pub fn space(&self) -> usize {
        let mut total = 0;

        for i in 0..SECTION_MAX {
            match self.sections[i] {
                Some(val) => total += val.size,
                _ => ()
            }
        }

        total
    }

    pub fn unused_sect(&mut self) -> u8 {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);
        let mut total = 0;

        for i in 0..sect_count() {
            match mybitmap.get_bool(i as usize) {
                true => (),
                false => total += 1,
            }
        }

        total
    }

    pub fn sect_size(&self, index: usize) -> usize {
        let section = self.sections[index];

        match section {
            None => panic!("Section does not exist"),
            Some(sect) => return sect.size
        }
    }

    pub fn is_used(&mut self, index: usize) -> bool {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        *mybitmap.get_bool(index)
    }
}

pub fn init_sect_manager() {
    let mmap = crate::MMR
        .get_response()
        .get()
        .expect("barebones: recieved no mmap");

    unsafe {
        for entry in mmap.memmap() {
            //println!("{:?}", entry.as_ptr());
            match entry.typ {
                limine::LimineMemoryMapEntryType::Usable => SECTION_MANAGER.add_sect(Sect::new(entry.len as usize, PhysAddr::new(entry.base).switch_form().as_mut_ptr())),
                _ => ()
            }
        }
    }
}

pub fn req_sect() -> (Sect, usize) {
    unsafe {
        SECTION_MANAGER.req_sect()
    }
}

//requests a sector of a certain size, will give you a sector of the closest available size
pub fn req_sect_size(size: usize) -> (Sect, usize) {
    unsafe {
        SECTION_MANAGER.req_sect_size(size)
    }
}

pub fn req_large_sect() -> (Sect, usize) {
    unsafe {
        SECTION_MANAGER.req_large_sect()
    }
}

pub unsafe fn ret_sect(index: usize) {
    SECTION_MANAGER.ret_sect(index);
}

pub unsafe fn empty_section(index: usize) {
    SECTION_MANAGER.empty_section(index);
}

pub fn sect_count() -> u8 {
    unsafe {
        return SECTION_MANAGER.sect_count();
    }
}

pub fn space() -> usize {
    unsafe {
        return SECTION_MANAGER.space();
    }
}

pub fn unused_sect() -> u8 {
    unsafe {
        return SECTION_MANAGER.unused_sect();
    }
}

pub fn sect_size(index: usize) -> usize {
    unsafe {
        SECTION_MANAGER.sect_size(index)
    }
}

pub fn is_used(index: usize) -> bool {
    unsafe {
        SECTION_MANAGER.is_used(index)
    }
}

const PAGE: usize = 4096;
const PAGE_MAX: usize = 4096 * 8;

#[repr(packed)]
pub struct Page([u8; PAGE]);

pub struct PageManager {
    sections: [Option<*mut Page>; PAGE_MAX],
    bitmap_buf: [u8; PAGE_MAX / 8],
    next_slot: usize,
}

impl PageManager {
    pub const fn null() -> PageManager {
        PageManager {
            sections: [None; PAGE_MAX],
            bitmap_buf: [0; PAGE_MAX / 8],
            next_slot: 0
        }
    }

    pub fn add_page(&mut self, new_sect: *mut Page) {
        if self.next_slot >= PAGE_MAX {
            panic!("Page limit reached, consider extending page limit");
        }

        assert!((new_sect as u64 & 0xfff) == 0, "Page provided is not aligned");

        self.sections[self.next_slot] = Some(new_sect);

        self.next_slot += 1;
    }

    pub fn req_page(&mut self) -> (*mut Page, usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        for i in 0..self.page_count() as usize {
            match mybitmap.get_bool(i) {
                true => (),
                false => {
                    match self.sections[i]{
                        None => (),
                        Some(val) => {
                            mybitmap.set_bool(i, true);
                            return (val, i);
                        }
                    }
                }
            }
        }

        panic!("No available pages");
    }

    //unsafe because if called while section is in use, and then the section is requested, the section will be cleared
    pub unsafe fn ret_sect(&mut self, index: usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        mybitmap.set_bool(index, false);
    }

    //unsafe because if called while section is in use, could delete important data
    pub unsafe fn empty_section(&mut self, index: usize) {
        let section = self.sections[index].expect("Section does not exist");

        for i in 0..4096 {
            *(section as *mut u8).byte_add(i) = 0;
        }
    }

    pub fn page_count(&self) -> usize {
        self.next_slot - 1
    }

    pub fn space(&self) -> usize {
        let total = self.page_count() as usize * 4096;

        total
    }

    pub fn unused_sect(&mut self) -> u8 {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);
        let mut total = 0;

        for i in 0..sect_count() {
            match mybitmap.get_bool(i as usize) {
                true => (),
                false => total += 1,
            }
        }

        total
    }

    pub fn is_used(&mut self, index: usize) -> bool {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        *mybitmap.get_bool(index)
    }
}

static mut PAGE_MANAGER: PageManager = PageManager::null();

pub fn init_page_manager() {
    let page_base = req_large_sect(); //size of at least 647169

    println!("Page base taken");
    println!("Section size is {} pages", page_base.0.size / 4096);

    let mut pages = page_base.0.size / 4096;
    if pages > PAGE_MAX {
        pages = PAGE_MAX;
    }

    unsafe {
        for i in 0..pages {
            PAGE_MANAGER.add_page((page_base.0.base as *mut Page).add(i));
        }
    }
}

pub fn zero_page(page: *mut Page) {
    unsafe {
        *page = Page([0; PAGE]);
    }
}

pub fn req_page() -> (*mut Page, usize) {
    unsafe {
        let page_data = PAGE_MANAGER.req_page();

        zero_page(page_data.0);

        page_data
    }
}

pub unsafe fn ret_page(index: usize) {
    PAGE_MANAGER.ret_sect(index);
}