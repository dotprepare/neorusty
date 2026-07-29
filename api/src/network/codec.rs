use std::io::{Read, Write};

pub trait StreamCodec<T>: Send + Sync + 'static {
    fn encode(&self, buf: &mut dyn Write, value: &T) -> Result<(), String>;
    fn decode(&self, buf: &mut dyn Read) -> Result<T, String>;
}

pub struct UnitCodec;

impl StreamCodec<()> for UnitCodec {
    fn encode(&self, _buf: &mut dyn Write, _value: &()) -> Result<(), String> {
        Ok(())
    }

    fn decode(&self, _buf: &mut dyn Read) -> Result<(), String> {
        Ok(())
    }
}

pub struct VarIntCodec;

impl StreamCodec<i32> for VarIntCodec {
    fn encode(&self, buf: &mut dyn Write, value: &i32) -> Result<(), String> {
        let mut val = *value as u64;
        loop {
            if val & !0x7F == 0 {
                let b = [val as u8];
                buf.write_all(&b).map_err(|e| e.to_string())?;
                break;
            }
            let b = [(val & 0x7F) as u8 | 0x80];
            buf.write_all(&b).map_err(|e| e.to_string())?;
            val >>= 7;
        }
        Ok(())
    }

    fn decode(&self, buf: &mut dyn Read) -> Result<i32, String> {
        let mut val = 0u64;
        let mut bits = 0;
        loop {
            let mut b = [0u8];
            buf.read_exact(&mut b).map_err(|e| e.to_string())?;
            val |= ((b[0] & 0x7F) as u64) << bits;
            if b[0] & 0x80 == 0 {
                break;
            }
            bits += 7;
            if bits >= 64 {
                return Err("VarInt too long".to_string());
            }
        }
        Ok(val as i32)
    }
}

pub struct StringCodec;

impl StreamCodec<String> for StringCodec {
    fn encode(&self, buf: &mut dyn Write, value: &String) -> Result<(), String> {
        let bytes = value.as_bytes();
        VarIntCodec.encode(buf, &(bytes.len() as i32))?;
        buf.write_all(bytes).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn decode(&self, buf: &mut dyn Read) -> Result<String, String> {
        let len = VarIntCodec.decode(buf)?;
        if len < 0 || len > 65536 {
            return Err(format!("Invalid string length: {len}"));
        }
        let mut bytes = vec![0u8; len as usize];
        buf.read_exact(&mut bytes).map_err(|e| e.to_string())?;
        String::from_utf8(bytes).map_err(|e| e.to_string())
    }
}
