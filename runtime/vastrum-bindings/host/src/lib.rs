use vastrum_shared_types::limits::MAX_WASM_HOST_BUFFER_SIZE;
use wasmtime::{AsContext, AsContextMut, Caller, Linker, Module, Store, TypedFunc};

pub fn call_contract<T: HostRuntime + 'static>(
    linker: &Linker<T>,
    store: &mut Store<T>,
    module: &Module,
    calldata: &[u8],
) -> wasmtime::Result<()> {
    invoke_entry_point(linker, store, module, "makecall", calldata)
}

pub fn construct_contract<T: HostRuntime + 'static>(
    linker: &Linker<T>,
    store: &mut Store<T>,
    module: &Module,
    constructor_params: &[u8],
) -> wasmtime::Result<()> {
    invoke_entry_point(linker, store, module, "construct", constructor_params)
}

pub trait HostRuntime {
    fn message_sender(&self) -> Vec<u8>;
    fn block_time(&self) -> u64;
    fn kv_insert(&mut self, args: &[u8]);
    fn kv_get(&self, args: &[u8]) -> Vec<u8>;
    fn log(&mut self, args: &[u8]);
    fn register_static_route(&mut self, args: &[u8]);
}

pub fn add_to_linker<T: HostRuntime + 'static>(linker: &mut Linker<T>) -> wasmtime::Result<()> {
    linker.func_wrap("vastrum", "block_time", |caller: Caller<'_, T>| -> u64 {
        caller.data().block_time()
    })?;

    linker.func_wrap(
        "vastrum",
        "message_sender",
        |mut caller: Caller<'_, T>,
         out_ptr_ptr: u32,
         out_len_ptr: u32|
         -> Result<(), wasmtime::Error> {
            let bytes = caller.data().message_sender();
            return_bytes_to_guest(&mut caller, &bytes, out_ptr_ptr, out_len_ptr)
        },
    )?;

    linker.func_wrap(
        "vastrum",
        "kv_insert",
        |mut caller: Caller<'_, T>, ptr: u32, len: u32| -> Result<(), wasmtime::Error> {
            let buf = read_bytes_from_guest_memory(&mut caller, ptr, len)?;
            caller.data_mut().kv_insert(&buf);
            Ok(())
        },
    )?;

    linker.func_wrap(
        "vastrum",
        "kv_get",
        |mut caller: Caller<'_, T>,
         ptr: u32,
         len: u32,
         out_ptr_ptr: u32,
         out_len_ptr: u32|
         -> Result<(), wasmtime::Error> {
            let args = read_bytes_from_guest_memory(&mut caller, ptr, len)?;
            let value = caller.data().kv_get(&args);
            return_bytes_to_guest(&mut caller, &value, out_ptr_ptr, out_len_ptr)
        },
    )?;

    linker.func_wrap(
        "vastrum",
        "log",
        |mut caller: Caller<'_, T>, ptr: u32, len: u32| -> Result<(), wasmtime::Error> {
            let buf = read_bytes_from_guest_memory(&mut caller, ptr, len)?;
            caller.data_mut().log(&buf);
            Ok(())
        },
    )?;

    linker.func_wrap(
        "vastrum",
        "register_static_route",
        |mut caller: Caller<'_, T>, ptr: u32, len: u32| -> Result<(), wasmtime::Error> {
            let buf = read_bytes_from_guest_memory(&mut caller, ptr, len)?;
            caller.data_mut().register_static_route(&buf);
            Ok(())
        },
    )?;

    Ok(())
}

fn invoke_entry_point<T: HostRuntime + 'static>(
    linker: &Linker<T>,
    store: &mut Store<T>,
    module: &Module,
    entry_point: &str,
    calldata: &[u8],
) -> wasmtime::Result<()> {
    let instance = linker.instantiate(&mut *store, module)?;
    let func = instance.get_typed_func::<(u32, u32), ()>(&mut *store, entry_point)?;

    if calldata.is_empty() {
        func.call(&mut *store, (0, 0))?;
    } else {
        let alloc = get_instance_alloc(&instance, store)?;
        let ptr = alloc.call(&mut *store, calldata.len() as u32)?;
        let memory = get_instance_memory(&instance, store)?;
        memory.write(&mut *store, ptr as usize, calldata)?;
        func.call(&mut *store, (ptr, calldata.len() as u32))?;
    }

    Ok(())
}

fn read_bytes_from_guest_memory<T>(
    caller: &mut Caller<'_, T>,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, wasmtime::Error> {
    if len > MAX_WASM_HOST_BUFFER_SIZE {
        return Err(wasmtime::Error::msg("buffer too large"));
    }
    let mem = get_guest_memory(caller)?;
    let mut buf = vec![0u8; len as usize];
    mem.read(caller.as_context(), ptr as usize, &mut buf)?;
    Ok(buf)
}

fn write_bytes_to_guest_memory<T>(
    caller: &mut Caller<'_, T>,
    ptr: u32,
    data: &[u8],
) -> Result<(), wasmtime::Error> {
    let mem = get_guest_memory(caller)?;
    mem.write(caller.as_context_mut(), ptr as usize, data)?;
    Ok(())
}

fn alloc_bytes_in_guest_memory<T>(
    caller: &mut Caller<'_, T>,
    data: &[u8],
) -> Result<u32, wasmtime::Error> {
    let alloc = get_guest_alloc(caller)?;
    let ptr = alloc.call(&mut *caller, data.len() as u32)?;
    write_bytes_to_guest_memory(caller, ptr, data)?;
    Ok(ptr)
}

fn return_bytes_to_guest<T>(
    caller: &mut Caller<'_, T>,
    bytes: &[u8],
    out_ptr_ptr: u32,
    out_len_ptr: u32,
) -> Result<(), wasmtime::Error> {
    if bytes.is_empty() {
        write_bytes_to_guest_memory(caller, out_ptr_ptr, &0u32.to_le_bytes())?;
        write_bytes_to_guest_memory(caller, out_len_ptr, &0u32.to_le_bytes())?;
        return Ok(());
    }
    let guest_ptr = alloc_bytes_in_guest_memory(caller, bytes)?;
    write_bytes_to_guest_memory(caller, out_ptr_ptr, &guest_ptr.to_le_bytes())?;
    write_bytes_to_guest_memory(caller, out_len_ptr, &(bytes.len() as u32).to_le_bytes())?;
    Ok(())
}

fn get_guest_alloc<T>(caller: &mut Caller<'_, T>) -> Result<TypedFunc<u32, u32>, wasmtime::Error> {
    let Some(export) = caller.get_export("__alloc") else {
        return Err(wasmtime::Error::msg("missing __alloc"));
    };
    let Some(func) = export.into_func() else {
        return Err(wasmtime::Error::msg("__alloc export is not a function"));
    };
    let typed = func.typed(caller.as_context())?;
    Ok(typed)
}

fn get_guest_memory<T>(caller: &mut Caller<'_, T>) -> Result<wasmtime::Memory, wasmtime::Error> {
    let Some(export) = caller.get_export("memory") else {
        return Err(wasmtime::Error::msg("missing memory export"));
    };
    let Some(memory) = export.into_memory() else {
        return Err(wasmtime::Error::msg("memory export is not a memory"));
    };
    return Ok(memory);
}

fn get_instance_memory<T>(
    instance: &wasmtime::Instance,
    store: &mut Store<T>,
) -> Result<wasmtime::Memory, wasmtime::Error> {
    let Some(memory) = instance.get_memory(&mut *store, "memory") else {
        return Err(wasmtime::Error::msg("missing memory export"));
    };
    Ok(memory)
}

fn get_instance_alloc<T>(
    instance: &wasmtime::Instance,
    store: &mut Store<T>,
) -> Result<TypedFunc<u32, u32>, wasmtime::Error> {
    let alloc = instance.get_typed_func::<u32, u32>(&mut *store, "__alloc")?;
    Ok(alloc)
}
