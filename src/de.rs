// https://serde.rs/impl-deserializer.html

use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::{forward_to_deserialize_any, Deserialize};

use crate::{Error, Result};

pub struct Deserializer {
    args: Vec<String>,
    empty: bool,
}

/// to be used with `env::args()` to get command line parameters parsed.
/// This automatically skips the binary from the first position of the
/// args, so can be used directly as-is.
pub fn from_args<'a, T, I>(iter: I) -> Result<T>
where
    I: Iterator<Item = String>,
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_args(iter.skip(1));
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.args.is_empty() {
        Ok(t)
    } else {
        Err(Error::Message("premature cancel of parse".to_string()))
    }
}

// to be used with any other string array
pub fn from_iter<'a, T, I>(iter: I) -> Result<T>
where
    I: Iterator<Item = &'static str>,
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_iter(iter);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.args.is_empty() {
        Ok(t)
    } else {
        Err(Error::Message("premature cancel of parse".to_string()))
    }
}

impl Deserializer {
    fn from_args<I>(iter: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let mut d = Deserializer {
            args: iter
                .map(|s| s.trim().to_owned()) // trim whitespace
                .filter(|p| !p.is_empty()) // remove elements that are zero sized
                .collect(),
            empty: false,
        };
        d.args.reverse();
        d
    }
    fn from_iter<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'static str>,
    {
        Self::from_args(iter.map(|s| s.to_owned()))
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut s = self;
        if s.args.is_empty() {
            return visitor.visit_none();
        }
        match s.args.pop().unwrap().as_str() {
            "-t" => visitor.visit_bool(true),
            "-f" => visitor.visit_bool(false),
            "-n" => visitor.visit_none(),
            "--" => {
                let arg = s.args.pop().unwrap();
                visitor.visit_str(&arg)
            }
            "[" => {
                // Object or array about to start, depends if key next
                let next = s.args.last().unwrap();
                if next.starts_with("--") && next.len() > 2 {
                    let result = visitor.visit_map(&mut s);
                    s.args.pop().unwrap(); // TODO errors on these if they are bad
                    result
                } else {
                    let result = visitor.visit_seq(&mut s);
                    s.args.pop().unwrap(); // TODO errors on these if they are bad
                    result
                }
            }
            "[]" => {
                s.empty = true;
                visitor.visit_seq(s)
            }
            "[--]" => {
                s.empty = true;
                visitor.visit_map(s)
            }
            v => {
                // We're dealing with a key
                if v.starts_with("--") && v.len() > 2 {
                    visitor.visit_str(v.strip_prefix("--").unwrap())
                } else {
                    // We're dealing with a number or a string next
                    if let Ok(uint) = v.parse::<u64>() {
                        visitor.visit_u64(uint)
                    } else if let Ok(int) = v.parse::<i64>() {
                        visitor.visit_i64(int)
                    } else if let Ok(float) = v.parse::<f64>() {
                        visitor.visit_f64(float)
                    } else {
                        visitor.visit_str(v)
                    }
                }
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.args.last().unwrap() == "-n" {
            self.args.pop();
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let item = self.args.last().unwrap();
        if item != "[" {
            // Visit a unit variant.
            visitor.visit_enum(self.args.pop().unwrap().into_deserializer())
        } else {
            self.args.pop().unwrap();
            let value = visitor.visit_enum(Enum::new(self))?;
            // TODO: check that next char is ]
            self.args.pop().unwrap();
            Ok(value)
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for Deserializer {
    type Error = crate::Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.empty || self.args.last().unwrap() == "]" {
            self.empty = false;
            return Ok(None);
        }
        seed.deserialize(self).map(Some)
    }
}

impl<'de> MapAccess<'de> for Deserializer {
    type Error = crate::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.empty || self.args.last().unwrap() == "]" {
            self.empty = false;
            return Ok(None);
        }
        seed.deserialize(self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }
}

struct Enum<'a> {
    de: &'a mut Deserializer,
}

impl<'a> Enum<'a> {
    fn new(de: &'a mut Deserializer) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        // Was handled earlier
        panic!();
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use serde::Serialize;

    use crate::ser;

    use super::*;

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    struct Test {
        str: String,
        int: u32,
        int1: Option<u32>,
        int2: Option<u32>,
        int3: Option<u32>,
        data: bool,
        seq: Vec<String>,
        map: HashMap<String, i32>,
        e: E,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    #[test]
    fn deserialize() {
        let v: Vec<&str> = vec![
            "./binary",
            "[",
            "--str",
            "data",
            "--int",
            "123",
            "--int1",
            "456",
            "--int2",
            "-n",
            "--seq",
            "[",
            "--",
            "'hello there'",
            "general",
            "kenobi",
            "]",
            "--data",
            "-t",
            "--map",
            "[",
            "--one",
            "2",
            "--three",
            "4",
            "]",
            "--e",
            "Unit",
            "]",
        ];
        let v = v.into_iter().map(String::from);
        let t: Test = from_args(v).unwrap();
        assert_eq!(
            t,
            Test {
                str: "data".to_string(),
                int: 123,
                int1: Some(456),
                int2: None,
                int3: None,
                data: true,
                seq: vec![
                    "'hello there'".to_string(),
                    "general".to_string(),
                    "kenobi".to_string()
                ],
                map: HashMap::from([("one".to_string(), 2), ("three".to_string(), 4)]),
                e: E::Unit,
            }
        );
    }

    #[test]
    fn unit_struct() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Test {}
        let v: Vec<&str> = vec!["[--]"];
        let t: Test = from_iter(v.into_iter()).unwrap();
        assert_eq!(t, Test {});
    }

    #[test]
    fn empty_seq() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Test {
            seq: Vec<String>,
            extra: i32,
        }
        let v: Vec<&str> = vec!["[", "--seq", "[]", "--extra", "-1", "]"];
        let t: Test = from_iter(v.into_iter()).unwrap();
        assert_eq!(
            t,
            Test {
                seq: vec![],
                extra: -1
            }
        );
    }

    #[test]
    fn empty_map() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Test {
            map: HashMap<String, String>,
            extra: i32,
        }
        let v: Vec<&str> = vec!["[", "--map", "[--]", "--extra", "-1", "]"];
        let t: Test = from_iter(v.into_iter()).unwrap();
        assert_eq!(
            t,
            Test {
                map: HashMap::new(),
                extra: -1,
            }
        );
    }

    #[test]
    fn deserialize_enum_variants() {
        let expected = E::Unit;
        assert_eq!(
            from_iter::<E, _>(vec!["Unit"].into_iter()).unwrap(),
            expected
        );

        let expected = E::Newtype(1);
        assert_eq!(
            from_iter::<E, _>(vec!["[", "--Newtype", "1", "]"].into_iter()).unwrap(),
            expected
        );

        let expected = E::Tuple(1, 2);
        assert_eq!(
            from_iter::<E, _>(vec!["[", "--Tuple", "[", "1", "2", "]", "]"].into_iter()).unwrap(),
            expected
        );

        let expected = E::Struct { a: 1 };
        assert_eq!(
            from_iter::<E, _>(vec!["[", "--Struct", "[", "--a", "1", "]", "]"].into_iter())
                .unwrap(),
            expected
        );
    }

    #[test]
    fn ser_then_de() {
        let initial = Test {
            str: "data".to_string(),
            int: 123,
            int1: Some(456),
            int2: None,
            int3: None,
            data: true,
            seq: vec![
                "'hello there'".to_string(),
                "general".to_string(),
                "kenobi".to_string(),
            ],
            map: HashMap::from([("one".to_string(), 2), ("three".to_string(), 4)]),
            e: E::Newtype(3),
        };
        let mut out = ser::to_params(&initial).unwrap();
        // shell escapes with regular quotes are weird, so we have to emplace that single quote
        // back. TODO: output easily reingestible data
        let pos = out
            .iter()
            .position(|i| i == "''\\''hello there'\\'''")
            .unwrap();
        out[pos] = "'hello there'".to_string();
        out.insert(0, "./binary".to_string());
        let output = from_args(out.into_iter()).unwrap();
        assert_eq!(initial, output);
    }
}
