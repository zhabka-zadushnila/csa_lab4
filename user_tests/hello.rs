let msg: String = "Hello, world!";

fn main() {
    let i: i32 = 0;
    while msg[i] != 0 {
        out(msg[i]);
        i = i + 1;
    }
}