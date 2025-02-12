use super::nbt_value::NbtValue;
use super::tags::TagType;
use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read};

pub struct NbtReader {}

impl NbtReader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_nbt_file(&self, path: &str) -> io::Result<Vec<u8>> {
        let file = File::open(path)?;
        let mut gz = GzDecoder::new(file);

        let mut buffer = Vec::new();
        gz.read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    pub fn parse_nbt<R: Read>(&self, reader: &mut R) -> io::Result<NbtValue> {
        let (tag_type, _) = self.read_tag(reader)?;

        if tag_type != TagType::Compound {
            panic!("The NBT does not start with a compound tag");
        }

        let value = self.read_compound(reader)?;
        Ok(value)
    }

    /// Read a compound tag
    pub fn read_compound<R: Read>(&self, reader: &mut R) -> io::Result<NbtValue> {
        let mut items = BTreeMap::new();

        loop {
            if let Ok((tag_type, tag_name)) = self.read_tag(reader) {
                if tag_type == TagType::End {
                    break;
                }

                let value = self.read_tag_payload(tag_type, reader)?;
                items.insert(tag_name, value);
            } else {
                panic!("Failed to read the compound tag");
            }
        }

        let value = NbtValue::Compound(items);
        Ok(value)
    }

    /// Read and parse the tag payload
    pub fn read_tag_payload<R: Read>(&self, tag_type: TagType, reader: &mut R) -> io::Result<NbtValue> {
        match tag_type {
            TagType::Byte => {
                let value = reader.read_i8()?;
                Ok(NbtValue::Byte(value))
            }
            TagType::Short => {
                let value = reader.read_i16::<BigEndian>()?;
                Ok(NbtValue::Short(value))
            }
            TagType::Int => {
                let value = reader.read_i32::<BigEndian>()?;
                Ok(NbtValue::Int(value))
            }
            TagType::Long => {
                let value = reader.read_i64::<BigEndian>()?;
                Ok(NbtValue::Long(value))
            }
            TagType::Float => {
                let value = reader.read_f32::<BigEndian>()?;
                Ok(NbtValue::Float(value))
            }
            TagType::Double => {
                let value = reader.read_f64::<BigEndian>()?;
                Ok(NbtValue::Double(value))
            }
            TagType::ByteArray => {
                let length = reader.read_i32::<BigEndian>()?;

                let mut array = Vec::new();
                for _i in 0..length {
                    let value = reader.read_i8()?;
                    array.push(value);
                }

                Ok(NbtValue::ByteArray(array))
            }
            TagType::String => {
                let length = reader.read_i16::<BigEndian>()?;

                let mut array = Vec::new();
                for _i in 0..length {
                    let value = reader.read_u8()?;
                    array.push(value);
                }

                let parsed = String::from_utf8(array);
                if let Ok(str) = parsed {
                    Ok(NbtValue::String(str))
                } else {
                    Ok(NbtValue::String(String::from("Invalid UTF-8 string")))
                }
            }
            TagType::List => {
                let content_tag_type_byte = reader.read_u8()?;
                let length = reader.read_i32::<BigEndian>()?;

                let mut array = Vec::new();
                for _i in 0..length {
                    let content_tag_type = TagType::from(content_tag_type_byte);
                    let value = self.read_tag_payload(content_tag_type, reader)?;
                    array.push(value);
                }

                Ok(NbtValue::List(array))
            }
            TagType::Compound => {
                let data = self.read_compound(reader)?;
                Ok(data)
            }
            TagType::IntArray => {
                let length = reader.read_i32::<BigEndian>()?;

                let mut array = Vec::new();
                for _i in 0..length {
                    let value = reader.read_i32::<BigEndian>()?;
                    array.push(value);
                }

                Ok(NbtValue::IntArray(array))
            }
            TagType::LongArray => {
                let length = reader.read_i32::<BigEndian>()?;

                let mut array = Vec::new();
                for _i in 0..length {
                    let value = reader.read_i64::<BigEndian>()?;
                    array.push(value);
                }

                Ok(NbtValue::LongArray(array))
            }
            _ => panic!("The tag type not supported"),
        }
    }

    /// Read the tag type and name from the buffer
    pub fn read_tag<R: Read>(&self, reader: &mut R) -> io::Result<(TagType, String)> {
        let tag_type_byte = reader.read_u8()?;
        let tag_type = TagType::from(tag_type_byte);

        if tag_type == TagType::End {
            return Ok((tag_type, String::from("End")));
        }

        let name_length = reader.read_u16::<BigEndian>()?;
        let mut name_buffer = vec![0; name_length as usize];

        reader.read_exact(&mut name_buffer)?;
        let name = String::from_utf8(name_buffer).unwrap_or_else(|_| String::from("Invalid UTF-8 string"));

        Ok((tag_type, name))
    }
}