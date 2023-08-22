// Helpers for deserialization
/// Processes a string by removing spaces, vowels, and truncating it to a maximum length.
///
/// # Arguments
///
/// * `input` - A string slice that holds the input string.
/// * `len` - A usize that holds the maximum length of the processed string.
///
/// # Returns
///
/// A String that holds the processed string.
pub fn process_string(input: &str, len: usize) -> String {
    let vowels = ['a', 'e', 'i', 'o', 'u', 'A', 'E', 'I', 'O', 'U'];
    let mut processed = input.to_string();
    if processed.len() > len {
        processed = processed.replace(" ", "");
    }
    if processed.len() > len {
        processed = processed
            .chars()
            .filter(|&c| !vowels.contains(&c))
            .collect();
    }
    if processed.len() > len {
        processed.truncate(len);
    }
    processed
}

// Using a macro to reduce repetition
/// Macro to implement traits for types used in WebAssembly.
#[cfg(feature = "wasm")]
#[macro_export]
macro_rules! wasm_impls {
    ($type:ident, $($variant:ident($value:expr),)*) => {
        /// Implements the `FromWasmAbi` trait for the given type.
        impl FromWasmAbi for $type {
            type Abi = u32;

            unsafe fn from_abi(js: u32) -> Self {
                match js {
                    $($value => $type::$variant($value),)*
                    _ => panic!(concat!("Invalid ", stringify!($type))),
                }
            }
        }

        /// Implements the `WasmDescribe` trait for the given type.
        impl WasmDescribe for $type {
            fn describe() {
                u32::describe();
            }
        }

        /// Implements the `OptionFromWasmAbi` trait for the given type.
        impl OptionFromWasmAbi for $type {
            fn is_none(abi: &u32) -> bool {
                *abi == 0
            }
        }
    };
}