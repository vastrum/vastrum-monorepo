#[cfg(target_arch = "wasm32")]
mod raw {
    #[link(wasm_import_module = "vastrum")]
    unsafe extern "C" {
        pub fn message_sender(out_ptr: *mut u32, out_len: *mut u32);
        pub fn block_time() -> u64;
        pub fn kv_insert(ptr: *const u8, len: u32);
        pub fn kv_get(ptr: *const u8, len: u32, out_ptr: *mut u32, out_len: *mut u32);
        pub fn log(ptr: *const u8, len: u32);
        pub fn register_static_route(ptr: *const u8, len: u32);
    }
}

#[cfg(target_arch = "wasm32")]
unsafe fn read_output(out_ptr: u32, out_len: u32) -> Vec<u8> {
    if out_len == 0 {
        return Vec::new();
    }
    Vec::from_raw_parts(out_ptr as *mut u8, out_len as usize, out_len as usize)
}

#[cfg(target_arch = "wasm32")]
pub mod runtime_raw {
    pub fn message_sender() -> Vec<u8> {
        let mut out_ptr: u32 = 0;
        let mut out_len: u32 = 0;
        unsafe {
            super::raw::message_sender(&mut out_ptr, &mut out_len);
            super::read_output(out_ptr, out_len)
        }
    }

    pub fn block_time() -> u64 {
        unsafe { super::raw::block_time() }
    }

    pub fn kv_get(args: &[u8]) -> Vec<u8> {
        let mut out_ptr: u32 = 0;
        let mut out_len: u32 = 0;
        unsafe {
            super::raw::kv_get(args.as_ptr(), args.len() as u32, &mut out_ptr, &mut out_len);
            super::read_output(out_ptr, out_len)
        }
    }

    pub fn kv_insert(args: &[u8]) {
        unsafe { super::raw::kv_insert(args.as_ptr(), args.len() as u32) }
    }

    pub fn log(args: &[u8]) {
        unsafe { super::raw::log(args.as_ptr(), args.len() as u32) }
    }

    pub fn register_static_route(args: &[u8]) {
        unsafe { super::raw::register_static_route(args.as_ptr(), args.len() as u32) }
    }
}

//stubs for rust analyzer
#[cfg(not(target_arch = "wasm32"))]
pub mod runtime_raw {
    pub fn message_sender() -> Vec<u8> {
        unimplemented!()
    }
    pub fn block_time() -> u64 {
        unimplemented!()
    }
    pub fn kv_get(_args: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
    pub fn kv_insert(_args: &[u8]) {
        unimplemented!()
    }
    pub fn log(_args: &[u8]) {
        unimplemented!()
    }
    pub fn register_static_route(_args: &[u8]) {
        unimplemented!()
    }
}
