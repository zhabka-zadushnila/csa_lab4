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
        let idx = (addr as usize) % self.cache_size;
        let tag = addr / self.cache_size as u32;
        if self.cache_tags[idx] == tag {
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
        let idx = (addr as usize) % self.cache_size;
        let tag = addr / self.cache_size as u32;
        if self.cache_tags[idx] == tag {
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
            let idx = (self.addr as usize) % self.cache_size;
            let tag = self.addr / self.cache_size as u32;
            if self.write {
                self.write_through(self.addr, self.mdr);
                if self.cache_tags[idx] == tag {
                    self.cache_data[idx] = self.mdr;
                }
            } else {
                let data = self.read_through(self.addr);
                self.mdr = data;
                self.cache_tags[idx] = tag;
                self.cache_data[idx] = data;
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
                } else if (addr as usize) < self.size {
                    self.data[addr as usize]
                } else {
                    0
                }
            }
            _ if (addr as usize) < self.size => self.data[addr as usize],
            _ => 0,
        }
    }

    fn write_through(&mut self, addr: u32, value: i32) {
        if addr == Self::OUT_ADDR {
            self.output_buf.push(value);
        } else if (addr as usize) < self.size {
            self.data[addr as usize] = value;
        }
    }

    pub fn input_ready(&self) -> bool {
        self.input_pos < self.input_buf.len()
    }

    pub fn load(&mut self, addr: u32, data: &[i32]) {
        for (i, &val) in data.iter().enumerate() {
            let a = (addr as usize) + i;
            if a < self.size {
                self.data[a] = val;
            }
        }
    }

    pub fn get(&self, addr: u32) -> i32 {
        self.data[addr as usize]
    }

    pub fn set(&mut self, addr: u32, value: i32) {
        if (addr as usize) < self.size {
            self.data[addr as usize] = value;
        }
    }
}
