use simulator::memory::Memory;
use simulator::mpu::Mpu;
use simulator::processor::Processor;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let bin_path = &args[1];
    let binary = fs::read(bin_path).unwrap();
    let data_end = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(2);
    let mem_size = args
        .get(3)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1 << 20);

    let mut memory = Memory::new(mem_size, 128);
    let mut mpu = Mpu::new();
    let mut proc = Processor::new();

    let num_words = binary.len() / 4;
    let mut words = Vec::with_capacity(num_words);
    for chunk in binary.chunks(4) {
        let mut val = 0i32;
        for (j, &b) in chunk.iter().enumerate() {
            val |= (b as i32) << (j * 8);
        }
        words.push(val);
    }

    let code_start = data_end as usize;
    let data_words = if code_start <= words.len() {
        code_start
    } else {
        words.len()
    };
    memory.load(0, &words[0..data_words]);
    let code_len = words.len() - data_words;
    if code_len > 0 {
        memory.load(data_end, &words[data_words..]);
    }

    proc.pc = data_end;
    proc.sp = data_end + code_len as u32;

    if let Some(in_path) = args.get(4) {
        let in_data = fs::read(in_path).unwrap();
        memory.input_buf = in_data
            .chunks(4)
            .map(|c| {
                let mut v = 0i32;
                for (j, &b) in c.iter().enumerate() {
                    v |= (b as i32) << (j * 8);
                }
                v
            })
            .collect();
    }

    while proc.run {
        mpu.tick(&mut proc, &mut memory);
    }

    for &v in &memory.output_buf {
        print!("{} ", v);
    }
    println!("Ticks: {}", proc.tick_count);
}
