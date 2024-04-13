use orgize::rowan::TextSize;
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::Deserialize;

pub fn deserialize<'de, D>(d: D) -> Result<TextSize, D::Error>
where
    D: Deserializer<'de>,
{
    <u32 as Deserialize>::deserialize(d).map(TextSize::new)
}

pub fn serialize<S>(t: &TextSize, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u32((*t).into())
}
