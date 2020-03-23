//! A crate for parsing the ISF aka Interactive Shader Format as described by the spec:
//!
//! https://www.interactiveshaderformat.com/spec
//!
//! Also provides deserialization and serialization of the `Isf` instance for ease of both
//! consuming and generating ISF shaders.
//!
//! The [**parse**](./fn.parse.html) function can parse a given GLSL string to produce an
//! [**Isf**](./struct.Isf.html) instance. The **Isf** type represents a fully structured
//! representation of the format, including typed [**Input**](./struct.Input.html)s.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::ops::Deref;
use std::path::PathBuf;
use thiserror::Error;

/// Representation of the JSON structure parsed from the top-level GLSL comment.
///
/// This is referred to as the "top-level dict" in the spec.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Isf {
    #[serde(default, rename = "ISFVSN")]
    pub isfvsn: Option<String>,
    #[serde(default, rename = "VSN")]
    pub vsn: Option<String>,
    #[serde(default, rename = "DESCRIPTION")]
    pub description: Option<String>,
    #[serde(default, rename = "CATEGORIES")]
    pub categories: Vec<String>,
    #[serde(default, rename = "INPUTS")]
    pub inputs: Vec<Input>,
    #[serde(default, rename = "PASSES")]
    pub passes: Vec<Pass>,
    #[serde(default, rename = "IMPORTED")]
    pub imported: BTreeMap<String, ImageImport>,
}

/// Describes an input to the ISF shader.
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub name: String,
    pub label: Option<String>,
    pub ty: InputType,
}

/// Input types supported by ISF.
#[derive(Clone, Debug, PartialEq)]
pub enum InputType {
    Event,
    Bool(InputBool),
    Long(InputLong),
    Float(InputFloat),
    Point2d(InputPoint2d),
    Color(InputColor),
    Image,
    Audio(InputAudio),
    AudioFft(InputAudioFft),
}

/// Possible values stored for the type.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InputValues<T> {
    #[serde(rename = "DEFAULT")]
    pub default: Option<T>,
    #[serde(rename = "MIN")]
    pub min: Option<T>,
    #[serde(rename = "MAX")]
    pub max: Option<T>,
    #[serde(rename = "IDENTITY")]
    pub identity: Option<T>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputBool {
    pub default: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputLong {
    pub input_values: InputValues<i32>,
    pub values: Vec<i32>,
    pub labels: Vec<String>,
}

pub type InputFloat = InputValues<f32>;

pub type InputPoint2d = InputValues<[f32; 2]>;

pub type InputColor = InputValues<Vec<f32>>;

#[derive(Clone, Debug, PartialEq)]
pub struct InputAudio {
    pub num_samples: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputAudioFft {
    pub num_columns: Option<u32>,
}

/// A helper type to simplify implementation of serialize/deserialize for `Input`.
#[derive(Debug, Deserialize, Serialize)]
struct InputDict {
    #[serde(rename = "NAME")]
    pub name: String,
    #[serde(rename = "LABEL")]
    pub label: Option<String>,
    #[serde(rename = "TYPE")]
    pub ty: String,
    #[serde(default, rename = "DEFAULT")]
    pub default: Option<serde_json::Value>,
    #[serde(default, rename = "MIN")]
    pub min: Option<serde_json::Value>,
    #[serde(default, rename = "MAX")]
    pub max: Option<serde_json::Value>,
    #[serde(default, rename = "IDENTITY")]
    pub identity: Option<serde_json::Value>,
    #[serde(default, rename = "VALUES")]
    pub values: Vec<i32>,
    #[serde(default, rename = "LABELS")]
    pub labels: Vec<String>,
}

/// Describes a pass of an ISF shader.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Pass {
    #[serde(default, rename = "TARGET")]
    pub target: Option<String>,
    #[serde(default, deserialize_with = "deserialize_bool", rename = "PERSISTENT")]
    pub persistent: bool,
    #[serde(default, deserialize_with = "deserialize_bool", rename = "FLOAT")]
    pub float: bool,
    #[serde(default, deserialize_with = "deserialize_opt_string", rename = "WIDTH")]
    pub width: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string", rename = "HEIGHT")]
    pub height: Option<String>,
}

/// A described image import
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ImageImport {
    #[serde(rename = "PATH")]
    pub path: PathBuf,
}

/// Errors that might occur while parsing a GLSL string for an ISF blob.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to find the top comment containing the JSON blob")]
    MissingTopComment,
    #[error("failed to parse JSON from the top comment: {err}")]
    Json {
        #[from]
        err: serde_json::Error,
    },
}

impl<T> InputValues<T> {
    fn from_opts(
        default: Option<serde_json::Value>,
        min: Option<serde_json::Value>,
        max: Option<serde_json::Value>,
        identity: Option<serde_json::Value>,
    ) -> Result<Self, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let default = match default {
            Some(t) => Some(serde_json::from_value(t)?),
            None => None,
        };
        let min = match min {
            Some(t) => Some(serde_json::from_value(t)?),
            None => None,
        };
        let max = match max {
            Some(t) => Some(serde_json::from_value(t)?),
            None => None,
        };
        let identity = match identity {
            Some(t) => Some(serde_json::from_value(t)?),
            None => None,
        };
        Ok(InputValues {
            default,
            min,
            max,
            identity,
        })
    }

    fn write_to_dict(&self, dict: &mut InputDict)
    where
        T: Clone + Into<serde_json::Value>,
    {
        dict.default = self.default.as_ref().map(|t| t.clone().into());
        dict.min = self.min.as_ref().map(|t| t.clone().into());
        dict.max = self.max.as_ref().map(|t| t.clone().into());
        dict.identity = self.identity.as_ref().map(|t| t.clone().into());
    }
}

impl Deref for InputLong {
    type Target = InputValues<i32>;
    fn deref(&self) -> &Self::Target {
        &self.input_values
    }
}

impl Serialize for Input {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Input { ref name, ref label, ref ty } = self;

        let mut dict = InputDict {
            name: name.clone(),
            label: label.clone(),
            ty: String::new(),
            default: None,
            min: None,
            max: None,
            identity: None,
            values: vec![],
            labels: vec![],
        };

        fn pt2_to_json_value([x, y]: [f32; 2]) -> serde_json::Value {
            serde_json::Value::Array(vec![x.into(), y.into()])
        }

        match ty {
            InputType::Event => {
                dict.ty = "event".to_string();
            },

            InputType::Bool(ref t) => {
                dict.ty = "bool".to_string();
                dict.default = t.default.map(Into::into);
            },

            InputType::Long(ref t) => {
                dict.ty = "long".to_string();
                t.write_to_dict(&mut dict);
                dict.values = t.values.clone();
                dict.labels = t.labels.clone();
            },

            InputType::Float(ref t) => {
                dict.ty = "float".to_string();
                t.write_to_dict(&mut dict);
            },

            InputType::Point2d(ref t) => {
                dict.ty = "point2D".to_string();
                dict.default = t.default.map(pt2_to_json_value);
                dict.min = t.min.map(pt2_to_json_value);
                dict.max = t.max.map(pt2_to_json_value);
                dict.identity = t.identity.map(pt2_to_json_value);
            },

            InputType::Color(ref t) => {
                dict.ty = "color".to_string();
                t.write_to_dict(&mut dict);
            },

            InputType::Image => {
                dict.ty = "image".to_string();
            },

            InputType::Audio(ref t) => {
                dict.ty = "audio".to_string();
                dict.max = t.num_samples.map(Into::into);
            },

            InputType::AudioFft(ref t) => {
                dict.ty = "audioFFT".to_string();
                dict.max = t.num_columns.map(Into::into);
            },
        };

        dict.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Input {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let InputDict {
            name,
            label,
            ty,
            default,
            min,
            max,
            identity,
            values,
            labels,
        } = InputDict::deserialize(d)?;

        let ty = match &ty[..] {
            "event" => InputType::Event,

            "bool" => InputType::Bool(InputBool {
                default: match default {
                    Some(serde_json::Value::Bool(b)) => Some(b),
                    Some(serde_json::Value::Number(n)) if n.is_u64() => {
                        Some(n.as_u64().unwrap() != 0)
                    }
                    Some(serde_json::Value::Number(n)) if n.is_f64() => {
                        Some(n.as_f64().unwrap() as u64 != 0)
                    }
                    Some(value) => {
                        serde_json::from_value(value).map_err(serde::de::Error::custom)?
                    }
                    None => None,
                },
            }),

            "long" => InputType::Long(InputLong {
                input_values: InputValues::from_opts(default, min, max, identity)
                    .map_err(serde::de::Error::custom)?,
                values,
                labels,
            }),

            "float" => InputType::Float(
                InputFloat::from_opts(default, min, max, identity)
                    .map_err(serde::de::Error::custom)?,
            ),

            "point2D" => InputType::Point2d(
                InputPoint2d::from_opts(default, min, max, identity)
                    .map_err(serde::de::Error::custom)?,
            ),

            "color" => InputType::Color(
                InputColor::from_opts(default, min, max, identity)
                    .map_err(serde::de::Error::custom)?,
            ),

            "image" => InputType::Image,

            "audio" => InputType::Audio(InputAudio {
                num_samples: match max {
                    Some(value) => {
                        serde_json::from_value(value).map_err(serde::de::Error::custom)?
                    }
                    None => None,
                },
            }),

            "audioFFT" => InputType::AudioFft(InputAudioFft {
                num_columns: match max {
                    Some(value) => {
                        serde_json::from_value(value).map_err(serde::de::Error::custom)?
                    }
                    None => None,
                },
            }),

            _ => unimplemented!(), // TODO: Return serde err "unknown type".
        };

        Ok(Input { name, label, ty })
    }
}

/// Attempt to parse an ISF blob from a GLSL source string.
///
/// This will not do any GLSL parsing and simply checks the top of the string for a `/* */` comment
/// containing JSON that may be parsed as an ISF blob.
pub fn parse(glsl_src: &str) -> Result<Isf, ParseError> {
    let comment_contents = top_comment_contents(glsl_src).ok_or(ParseError::MissingTopComment)?;
    Ok(serde_json::from_str(comment_contents)?)
}

/// Find the top `/* */` comment in a GLSL src string and return the contents with whitespace
/// trimmed.
fn top_comment_contents(glsl_src: &str) -> Option<&str> {
    let start = glsl_src.find("/*")? + "/*".len();
    let end = start + glsl_src[start..].find("*/")?;
    Some(glsl_src[start..end].trim())
}

/// Support integers for bool seriallization.
fn deserialize_bool<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let b = match serde_json::Value::deserialize(d)? {
        serde_json::Value::Bool(b) => b,
        serde_json::Value::Number(n) if n.is_u64() => {
            n.as_u64().unwrap() != 0
        }
        serde_json::Value::Number(n) if n.is_f64() => {
            n.as_f64().unwrap() as u64 != 0
        }
        value => serde_json::from_value(value).map_err(serde::de::Error::custom)?,
    };
    Ok(b)
}

/// A string deserialization that also supports integers.
fn deserialize_opt_string<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = match <_>::deserialize(d)? {
        None => None,
        Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        Some(serde_json::Value::String(s)) => Some(s),
        Some(value) => serde_json::from_value(value).map_err(serde::de::Error::custom)?,
    };
    Ok(opt)
}
