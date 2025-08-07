use tokio::io::{AsyncRead, AsyncReadExt};

/// Parses an EBML-style variable-length integer (vint).
///
/// Reads the first byte to determine the vint's total width, then reads the
/// remaining bytes to construct the final value.
///
/// # Arguments
/// * `reader`: An async reader to pull bytes from.
///
/// # Returns
/// A `Result` containing the parsed `u64` value or an `io::Error`.
async fn read_vint<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<u64> {
    // 1. Read the first byte to determine the width
    let first_byte = reader.read_u8().await?;
    
    // 2. Find the width by counting leading zeros
    let width = first_byte.leading_zeros();

    if width >= 8 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid vint: width marker is 8 or more",
        ));
    }

    // 3. The first byte is part of the number, so start with its value.
    // We must clear the width marker bit before using the value.
    let mut value = u64::from(first_byte & (0xFF >> (width + 1)));

    // 4. Read the remaining bytes (if any)
    // The total width is `width + 1`. We've read one byte, so we need `width` more.
    for _ in 0..width {
        let byte = reader.read_u8().await?;
        // Shift the existing value left by 8 bits and add the new byte
        value = (value << 8) | u64::from(byte);
    }

    Ok(value)
}