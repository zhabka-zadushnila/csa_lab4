pub struct Memory {
    data: Vec<i32>,
    pub input_buf: Vec<i32>,
    pub input_pos: usize,
    pub output_buf: Vec<i32>,
    size: usize,

    cache_tags: Vec<u32>,
    cache_data: Vec<i32>,
    cache_size: usize,

    busy: bool,
    done: bool,
    counter: u8,
    write: bool,
    addr: u32,
    mdr: i32,
}

impl Memory {
    pub fn new(size: usize, cache_size: usize) -> Self {
        Memory {
            data: vec![0i32; size],
            input_buf: Vec::new(),
            input_pos: 0,
            output_buf: Vec::new(),
            size,
            cache_tags: vec![u32::MAX; cache_size],
            cache_data: vec![0i32; cache_size],
            cache_size,
            busy: false,
            done: false,
            counter: 0,
            write: false,
            addr: 0,
            mdr: 0,
        }
    }

    pub fn read_start(&mut self, addr: u32) {
        if self.busy || self.done {
            return;
        }
        if addr == Self::IN_ADDR || addr == Self::OUT_ADDR {
            self.mdr = self.read_through(addr);
            self.done = true;
            return;
        }
        self.addr = addr;
        self.write = false;
        let word_addr = addr / 4;
        let idx = (word_addr as usize) % self.cache_size;
        let tag = word_addr / self.cache_size as u32;
        if addr.is_multiple_of(4) && self.cache_tags[idx] == tag {
            self.mdr = self.cache_data[idx];
            self.done = true;
        } else {
            self.counter = 10;
            self.busy = true;
        }
    }

    pub fn write_start(&mut self, addr: u32, value: i32) {
        if self.busy || self.done {
            return;
        }
        if addr == Self::IN_ADDR || addr == Self::OUT_ADDR {
            self.write_through(addr, value);
            self.done = true;
            return;
        }
        self.addr = addr;
        self.mdr = value;
        self.write = true;
        let word_addr = addr / 4;
        let idx = (word_addr as usize) % self.cache_size;
        let tag = word_addr / self.cache_size as u32;
        if addr.is_multiple_of(4) && self.cache_tags[idx] == tag {
            self.counter = 1;
        } else {
            self.counter = 10;
        }
        self.busy = true;
    }

    pub fn tick(&mut self) -> bool {
        if !self.busy {
            return false;
        }
        self.counter -= 1;
        if self.counter == 0 {
            let word_addr = self.addr / 4;
            let idx = (word_addr as usize) % self.cache_size;
            let tag = word_addr / self.cache_size as u32;
            if self.write {
                self.write_through(self.addr, self.mdr);
                if self.addr.is_multiple_of(4) && self.cache_tags[idx] == tag {
                    self.cache_data[idx] = self.mdr;
                }
            } else {
                let data = self.read_through(self.addr);
                self.mdr = data;
                if self.addr.is_multiple_of(4) {
                    self.cache_tags[idx] = tag;
                    self.cache_data[idx] = data;
                }
            }
            self.busy = false;
            self.done = true;
        }
        self.busy
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    pub fn is_done(&self) -> bool {
        self.done
    }

    pub fn collect(&mut self) -> i32 {
        self.done = false;
        self.mdr
    }

    pub fn clear_done(&mut self) {
        self.done = false;
    }

    const IN_ADDR: u32 = 0x1FFFFFE;
    const OUT_ADDR: u32 = 0x1FFFFFF;

    fn read_through(&mut self, addr: u32) -> i32 {
        match addr {
            Self::IN_ADDR => {
                if self.input_pos < self.input_buf.len() {
                    let val = self.input_buf[self.input_pos];
                    self.input_pos += 1;
                    val
                } else {
                    0
                }
            }
            _ => {
                let mut result = 0i32;
                for i in 0..=3 {
                    let byte_addr = addr.wrapping_add(i);
                    let word_addr = (byte_addr / 4) as usize;
                    let byte_off = (byte_addr % 4) as usize;
                    let word = if word_addr < self.size {
                        self.data[word_addr]
                    } else {
                        0
                    };
                    let byte = ((word >> (byte_off * 8)) & 0xFF) as i32;
                    result |= byte << (i * 8);
                }
                result
            }
        }
    }

    fn write_through(&mut self, addr: u32, value: i32) {
        if addr == Self::OUT_ADDR {
            self.output_buf.push(value);
            return;
        }
        for i in 0..=3 {
            let byte_addr = addr.wrapping_add(i);
            let word_addr = (byte_addr / 4) as usize;
            let byte_off = (byte_addr % 4) as usize;
            if word_addr < self.size {
                let byte = ((value >> (i * 8)) & 0xFF) as i32;
                let mask = !(0xFF << (byte_off * 8));
                let old = self.data[word_addr];
                self.data[word_addr] = (old & mask) | (byte << (byte_off * 8));
            }
        }
    }

    pub fn input_ready(&self) -> bool {
        self.input_pos < self.input_buf.len()
    }

    pub fn load(&mut self, addr: u32, data: &[i32]) {
        let base = (addr / 4) as usize;
        for (i, &val) in data.iter().enumerate() {
            let a = base + i;
            if a < self.size {
                self.data[a] = val;
            }
        }
    }

    pub fn get(&self, addr: u32) -> i32 {
        self.data[(addr / 4) as usize]
    }

    pub fn set(&mut self, addr: u32, value: i32) {
        let idx = (addr / 4) as usize;
        if idx < self.size {
            self.data[idx] = value;
        }
    }
}
