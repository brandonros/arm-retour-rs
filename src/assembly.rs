#![allow(non_snake_case)]

use bitfield::bitfield;

bitfield! {
    pub struct Movw(u32);
    impl Debug;
    u32;
    pub get_op3, set_op3: 31, 31;
    pub get_imm3, set_imm3: 30, 28;
    pub get_Rd, set_Rd: 27, 24;
    pub get_imm8, set_imm8: 23, 16;
    pub get_op1, set_op1: 15, 11;
    pub get_imm1, set_imm1: 10, 10;
    pub get_op2, set_op2: 9, 4;
    pub get_imm4, set_imm4: 3, 0;
}

bitfield! {
    pub struct Movt(u32);
    impl Debug;
    u32;
    pub get_op3, set_op3: 31, 31;
    pub get_imm3, set_imm3: 30, 28;
    pub get_Rd, set_Rd: 27, 24;
    pub get_imm8, set_imm8: 23, 16;
    pub get_op1, set_op1: 15, 11;
    pub get_imm1, set_imm1: 10, 10;
    pub get_op2, set_op2: 9, 4;
    pub get_imm4, set_imm4: 3, 0;
}

bitfield! {
    pub struct BlxRegister(u16);
    impl Debug;
    u16;
    pub get_opcode1, set_opcode1: 15, 7;
    pub get_Rm, set_Rm: 6, 3;
    pub get_opcode2, set_opcode2: 2, 0;
}

fn get_bit_range(input: u16, start: u8, num_bits: u8) -> u16 {
  let mask = (1u16 << num_bits) - 1;
  (input >> start) & mask
}

pub fn encode_movw(immediate: u16, rd: u8) -> u32 {
  println!("encode_movw immediate = {immediate:x} rd = {rd:x}");
  let mut movw = Movw(0);
  movw.set_op1(0b11110);
  movw.set_imm1(get_bit_range(immediate, 11, 1) as u32); // 11-11
  movw.set_op2(0b100100);
  movw.set_imm4(get_bit_range(immediate, 12, 4) as u32); // 12-15
  movw.set_imm3(get_bit_range(immediate, 8, 3) as u32); // 8-10
  movw.set_Rd(rd as u32);
  movw.set_imm8(get_bit_range(immediate, 0, 8) as u32); // 0-7
  movw.set_op3(0);
  return u32::from_le_bytes(movw.0.to_be_bytes());
}

pub fn encode_movt(immediate: u16, rd: u8) -> u32 {
  println!("encode_movt immediate = {immediate:x} rd = {rd:x}");
  let mut movt = Movt(0);
  movt.set_op1(0b11110);
  movt.set_imm1(get_bit_range(immediate, 11, 1) as u32); // 11-11
  movt.set_op2(0b101100);
  movt.set_imm4(get_bit_range(immediate, 12, 4) as u32); // 12-15
  movt.set_imm3(get_bit_range(immediate, 8, 3) as u32); // 8-10
  movt.set_Rd(rd as u32);
  movt.set_imm8(get_bit_range(immediate, 0, 8) as u32); // 0-7
  movt.set_op3(0);
  return u32::from_le_bytes(movt.0.to_be_bytes());
}

pub fn encode_blx_register(rm: u8) -> u16 {
  println!("encode_blx_register rm = {rm:x}");
  let mut blx_register = BlxRegister(0);
  blx_register.set_Rm(rm as u16);
  blx_register.set_opcode1(0b010001111);
  blx_register.set_opcode2(0);
  return u16::from_le_bytes(blx_register.0.to_be_bytes());
}

pub fn encode_bx() -> u16 {
  // TODO: bitfield for clarity + support something other than r12
  return 0x6047; // bx r12
}

pub fn encode_nop() -> u16 {
  // TODO: bitfield for clarity?
  return 0x00bf;
}
