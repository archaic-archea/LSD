use limine::{LimineKernelAddressRequest, LimineMemoryMapEntryType};
use x86_64::{structures::paging::{PageTable, PageTableFlags, page_table::PageTableEntry, PhysFrame}, PhysAddr, VirtAddr, registers::control::{Cr3, Cr3Flags}};
use crate::{*, memory::req_page};

static mut PML4: PageTable = PageTable::new();

static KERN_ADDR: LimineKernelAddressRequest = LimineKernelAddressRequest::new(0);

pub fn paging_init() {
    let memmap = MMR.get_response().get().unwrap();
    let _kern_addr = KERN_ADDR.get_response().get().unwrap();

    unsafe {
        for entry in memmap.memmap() {
            match entry.typ {
                LimineMemoryMapEntryType::BadMemory => (),
                LimineMemoryMapEntryType::Reserved => (),
                _ => {
                    println!("Mapping section of type {:?} with page length of {}", entry.typ, entry.len / 4096);

                    for i in 0..entry.len / 4096 {
                        let phys = PhysAddr::new(entry.base/* + (4096 * i)*/);
                        let virt = phys.switch_form();
            
                        map_virt(virt, &mut PML4, phys);
                    }
                }
            }

        }

        let pml4_virt = VirtAddr::new((&mut PML4 as *mut PageTable) as u64);
        let pml4_phys = pml4_virt.switch_form();

        println!("Mapping completed");

        Cr3::write(PhysFrame::from_start_address(pml4_phys).unwrap(), Cr3Flags::empty());
        println!("Cr3 loaded");
    }
}

pub fn map_virt(virt_addr: VirtAddr, pml4: &mut PageTable, phys_addr: PhysAddr) {
    let p1 = add_tables(virt_addr, pml4);

    let p1_index = virt_addr.p1_index();

    let std_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    p1[p1_index].set_addr(phys_addr, std_flags);
}

pub fn add_tables(virt_addr: VirtAddr, pml4: &mut PageTable) -> &mut PageTable {
    let pml4_index = virt_addr.p4_index();
    let pml3_index = virt_addr.p3_index();
    let pml2_index = virt_addr.p2_index();
    //let pml1_index = virt_addr.p1_index();

    let std_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    match pml4[pml4_index].flags() | PageTableFlags::PRESENT {
        PageTableFlags::PRESENT => (),
        _ => {
            let pml3_ptr = VirtAddr::new(req_page().0 as u64).switch_form();
            pml4[pml4_index].set_addr(pml3_ptr, std_flags);
        }
    }
    let pml3 = pml4[pml4_index].get_table();

    match pml4[pml4_index].flags() | PageTableFlags::PRESENT {
        PageTableFlags::PRESENT => (),
        _ => {
            let pml2_ptr = VirtAddr::new(req_page().0 as u64).switch_form();
            pml3[pml3_index].set_addr(pml2_ptr, std_flags);
        }
    }
    let pml2 = pml3[pml3_index].get_table();

    match pml4[pml4_index].flags() | PageTableFlags::PRESENT {
        PageTableFlags::PRESENT => (),
        _ => {
            let pml1_ptr = VirtAddr::new(req_page().0 as u64).switch_form();
            pml2[pml2_index].set_addr(pml1_ptr, std_flags);
        }
    }
    pml2[pml2_index].get_table()
}

impl AddrForm<VirtAddr> for PhysAddr {
    fn switch_form(&self) -> VirtAddr {
        let hhdm = crate::HHDM.get_response().get().unwrap();

        let result = self.as_u64().overflowing_add(hhdm.offset);
        
        if result.1 {
            panic!("Overflow occured, tried adding offset 0x{:x} to physical address {:?}", hhdm.offset, self)
        }

        VirtAddr::new(result.0)
    }
}

impl AddrForm<PhysAddr> for VirtAddr {
    fn switch_form(&self) -> PhysAddr {
        let hhdm = crate::HHDM.get_response().get().unwrap();

        let result = self.as_u64().overflowing_sub(hhdm.offset);

        match result {
            (val, true) => panic!("Overflow while subtracting occured\nLHS: {}\nRHS: {}\nResult: {}", self.as_u64(), hhdm.offset, val),
            _ => ()
        }

        PhysAddr::new(result.0)
    }
}

impl GetTable for PageTableEntry {
    fn get_table(&self) -> &mut PageTable {
        unsafe {&mut *self.addr().switch_form().as_mut_ptr::<PageTable>()}
    }
}

pub trait AddrForm<T> {
    fn switch_form(&self) -> T;
}

pub trait GetTable {
    fn get_table(&self) -> &mut PageTable;
}