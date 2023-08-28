use sbor::*;
use scrypto::prelude::wasm_api::*;
use scrypto::prelude::*;

#[blueprint]
mod wasm_buffers {

    struct WasmBuffersTest {
        kv_store: KeyValueStore<u32, Vec<u8>>,
        key: Vec<u8>,
    }

    impl WasmBuffersTest {
        pub fn new() -> Global<WasmBuffersTest> {
            let kv_store = KeyValueStore::<u32, Vec<u8>>::new();
            let key = scrypto_encode(&1u32).unwrap();

            Self { kv_store, key }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        fn get_kv_store_handle(&self) -> u32 {
            let node_id = self.kv_store.id.as_node_id();
            let handle = unsafe {
                kv_store::kv_store_open_entry(
                    node_id.as_ref().as_ptr(),
                    node_id.as_ref().len(),
                    self.key.as_ptr(),
                    self.key.len(),
                    LockFlags::MUTABLE.bits(),
                )
            };
            handle
        }

        /// Let native function "kv_entry_write" read the data from WASM memory and write it to
        /// the KV store
        /// Arguments:
        /// - buffer_size - WASM buffer to allocate
        /// - read_memory_offs - buffer offset to start reading data from
        /// - read_memory_len - number of bytes to read from memory
        /// WASM memory grows in 64KB chunks.
        /// If attempting to access outside WASM memory, make sure that
        /// read_memory_offset + read_memory_len > buffer_size + 64KB
        pub fn read_memory(
            &self,
            buffer_size: usize,
            read_memory_offs: usize,
            read_memory_len: usize,
        ) {
            // SBOR encoding of Vec<u8>
            let mut buffer = Vec::new();
            let mut encoder = VecEncoder::<ScryptoCustomValueKind>::new(&mut buffer, 100);
            encoder
                .write_payload_prefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
                .unwrap();
            encoder.write_value_kind(ValueKind::Array).unwrap();
            encoder.write_value_kind(ValueKind::U8).unwrap();
            encoder.write_size(buffer_size).unwrap();
            buffer.reserve(buffer_size);
            let new_size = buffer.len() + buffer_size;
            unsafe { buffer.set_len(new_size) };

            let handle = self.get_kv_store_handle();

            unsafe {
                kv_entry::kv_entry_write(
                    handle,
                    buffer.as_ptr().add(read_memory_offs),
                    read_memory_len,
                )
            };
            unsafe { kv_entry::kv_entry_close(handle) };
        }

        /// Let native function "kv_entry_read" get the data from the KV store and then native
        /// function "buffer_consume" writes the data to the WASM memory
        /// Arguments:
        /// - buffer_size - WASM buffer to allocate
        /// - write_memory_offs - buffer offset to start reading data from
        /// WASM memory grows in 64KB chunks.
        /// If attempting to access outside WASM memory, make sure that
        /// write_memory_offs > buffer_size + 64KB
        pub fn write_memory(&self, buffer_size: usize, write_memory_offs: usize) {
            let handle = self.get_kv_store_handle();
            let buffer = unsafe { kv_entry::kv_entry_read(handle) };

            // copy buffer
            let _vec = {
                let mut vec = Vec::<u8>::with_capacity(buffer_size);
                unsafe {
                    buffer::buffer_consume(buffer.id(), vec.as_mut_ptr().add(write_memory_offs));
                    vec.set_len(buffer_size);
                };
                vec
            };
        }
    }
}