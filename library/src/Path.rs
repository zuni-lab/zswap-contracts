// use bytes_utils::BytesLib;
// use bytes_utils_ext::BytesLibExt;
//
// pub struct BytesLibExt;
//
// impl BytesLibExt {
//   pub fn to_uint24(bytes: &[u8], start: usize) -> u32 {
//     assert!(bytes.len() >= start + 3, "to_uint24_outOfBounds");
//     let mut temp_uint: u32 = 0;
//
//     unsafe {
//       let ptr = bytes.as_ptr().add(start);
//       temp_uint = core::ptr::read_unaligned(ptr as *const u32);
//     }
//
//     temp_uint
//   }
// }
//
// pub struct Path;
//
// impl Path {
//   const ADDR_SIZE: usize = 20;
//   const FEE_SIZE: usize = 3;
//   const NEXT_OFFSET: usize = Path::ADDR_SIZE + Path::FEE_SIZE;
//   const POP_OFFSET: usize = Path::NEXT_OFFSET + Path::ADDR_SIZE;
//   const MULTIPLE_POOLS_MIN_LENGTH: usize = Path::POP_OFFSET + Path::NEXT_OFFSET;
//
//   pub fn has_multiple_pools(path: &[u8]) -> bool {
//     path.len() >= Path::MULTIPLE_POOLS_MIN_LENGTH
//   }
//
//   pub fn num_pools(path: &[u8]) -> usize {
//     (path.len() - Path::ADDR_SIZE) / Path::NEXT_OFFSET
//   }
//
//   pub fn get_first_pool(path: &[u8]) -> Vec<u8> {
//     path[0..Path::POP_OFFSET].to_vec()
//   }
//
//   pub fn skip_token(path: &[u8]) -> Vec<u8> {
//     path[Path::NEXT_OFFSET..].to_vec()
//   }
//
//   pub fn decode_first_pool(path: &[u8]) -> (Vec<u8>, Vec<u8>, u32) {
//     let token_in = path[0..Path::ADDR_SIZE].to_vec();
//     let fee = BytesLibExt::to_uint24(path, Path::ADDR_SIZE) as u32;
//     let token_out = path[Path::NEXT_OFFSET..Path::POP_OFFSET].to_vec();
//
//     (token_in, token_out, fee)
//   }
// }
