#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct BitMap {
    size: usize,
    buffer: *mut u8,
}

impl BitMap {
    pub const fn new(size: usize, buffer: *mut u8) -> BitMap {
        BitMap { size, buffer }
    }

    pub fn get_bool(&self, index: usize) -> &bool {
        let byte_index: u64 = index as u64 / 8;
        let bit_index: u8 = (index & 0b111) as u8;
        let bit_indexer: u8 = 0b10000000 >> bit_index;
        if (unsafe {*self.buffer.offset(byte_index as isize)} & bit_indexer) > 0 {
            return &true
        }
        return &false;
    }
    
    pub fn set_bool(&self, index: usize, val: bool) -> bool {
        if index as usize > self.size * 8 {return false;}

        let byte_index: u64 = index as u64 / 8;
        let bit_index: u8 = (index & 0b111) as u8;
        let bit_indexer: u8 = 0b10000000 >> bit_index;
        
        unsafe {*self.buffer.offset(byte_index as isize) &= !bit_indexer;}
        
        if val {
            unsafe {*self.buffer.offset(byte_index as isize) |= bit_indexer;}
        }
        
        return true;
    }
}

impl core::ops::Index<usize> for BitMap {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        return self.get_bool(index);
    }
}