mod nestest;
mod ppu;
use nestest::nestest;
// fn prompt(cpu: &mut Processor, mem: &mut Memory) -> u8{
//     let mut buf = String::new();
//     io::stdin().read_line(&mut buf).unwrap();
//     let cmd = buf.trim();

//     match cmd{
//         "e" => return 2,
//         "s" => {
//             println!("{:X}",mem.read(((cpu.s  as u16) + 1) | 0x100));
//             return 1    
//         },
//         _ => match cmd.parse::<u8>() {
//                 Ok(value) => {
//                     mem.display_pg(value);
//                     return 1
//                 },
//                 Err(_) => return 0
//         }
//     }
// }

fn main(){
    nestest().unwrap();
}
