pub mod memory;
pub mod mpu;
pub mod processor;

pub struct SimulationResult {
    pub trace: String,
    pub output: Vec<i32>,
}

pub fn run(result: &compiler::CompileResult, input: &[i32]) -> Result<SimulationResult, String> {
    let mut memory = memory::Memory::new(1 << 24, 1024);
    let mut mpu = mpu::Mpu::new();
    let mut proc = processor::Processor::new();

    let data_end = result.data_size;

    let num_words = result.binary.len() / 4;
    let mut words = Vec::with_capacity(num_words);
    for chunk in result.binary.chunks(4) {
        let mut val = 0i32;
        for (j, &b) in chunk.iter().enumerate() {
            val |= (b as i32) << (j * 8);
        }
        words.push(val);
    }

    let data_words = data_end as usize;

    memory.load(0, &words[0..data_words]);
    let code_len = words.len() - data_words;
    memory.load(data_end, &words[data_words..]);

    memory.input_buf = input.to_vec();

    proc.pc = data_end;
    proc.sp = data_end + code_len as u32;

    while proc.run {
        mpu.tick(&mut proc, &mut memory);
    }

    Ok(SimulationResult {
        trace: std::mem::take(&mut mpu.logs),
        output: std::mem::take(&mut memory.output_buf),
    })
}
