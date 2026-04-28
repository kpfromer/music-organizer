//! Image MIME types and audio formats / codecs persisted as TEXT.
//!
//! Each enum has an `Other(String)` variant so unknown values still
//! round-trip without dropping data — see conventions §4a.

use std::str::FromStr;

use sea_orm::{
    ColIdx, QueryResult, TryGetError, TryGetable,
    sea_query::{ArrayType, ColumnType, Nullable, StringLen, Value, ValueType, ValueTypeErr},
};
use serde::{Deserialize, Serialize};

macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $($variant:ident => $s:literal),* $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(into = "String", from = "String")]
        pub enum $name {
            $( $variant, )*
            Other(String),
        }

        impl $name {
            pub fn as_str(&self) -> &str {
                match self {
                    $( Self::$variant => $s, )*
                    Self::Other(s) => s.as_str(),
                }
            }

            pub fn parse(s: &str) -> Self {
                let lower = s.trim().to_ascii_lowercase();
                $( if lower == $s { return Self::$variant; } )*
                Self::Other(s.to_owned())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = std::convert::Infallible;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self::parse(s))
            }
        }

        impl From<$name> for String {
            fn from(v: $name) -> Self {
                v.to_string()
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self::parse(&s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self::parse(s)
            }
        }

        impl From<$name> for Value {
            fn from(v: $name) -> Self {
                Value::String(Some(Box::new(v.to_string())))
            }
        }

        impl TryGetable for $name {
            fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                let s = String::try_get_by(res, idx)?;
                Ok(Self::parse(&s))
            }
        }

        impl ValueType for $name {
            fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
                let s = <String as ValueType>::try_from(v)?;
                Ok(Self::parse(&s))
            }
            fn type_name() -> String { stringify!($name).to_owned() }
            fn array_type() -> ArrayType { ArrayType::String }
            fn column_type() -> ColumnType { ColumnType::String(StringLen::None) }
        }

        impl Nullable for $name {
            fn null() -> Value { Value::String(None) }
        }
    };
}

string_enum! {
    /// Image MIME type for cover-art rows. Stored canonical lowercase.
    ImageMimeType {
        Jpeg => "image/jpeg",
        Png  => "image/png",
        Gif  => "image/gif",
        Webp => "image/webp",
    }
}

string_enum! {
    /// Container/format string from the file header (`file.format`).
    /// Stored lowercase short name (`flac`, `mp3`, ...).
    AudioFormat {
        Flac => "flac",
        Mp3  => "mp3",
        Aac  => "aac",
        Ogg  => "ogg",
        Opus => "opus",
        Wav  => "wav",
        M4a  => "m4a",
    }
}

string_enum! {
    /// Audio codec reported by the decoder (`file_audio_info.codec`).
    AudioCodec {
        Flac   => "flac",
        Mp3    => "mp3",
        Aac    => "aac",
        Vorbis => "vorbis",
        Opus   => "opus",
        Pcm    => "pcm",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn mime_known() {
        assert_eq!(ImageMimeType::parse("image/PNG"), ImageMimeType::Png);
        assert_eq!(ImageMimeType::Png.as_str(), "image/png");
    }

    #[test]
    fn mime_other_preserves_input() {
        let m = ImageMimeType::parse("image/avif");
        assert_eq!(m, ImageMimeType::Other("image/avif".into()));
        assert_eq!(m.as_str(), "image/avif");
    }

    #[test]
    fn format_roundtrip() {
        let f = AudioFormat::Flac;
        let v: Value = f.clone().into();
        assert_eq!(<AudioFormat as ValueType>::try_from(v).unwrap(), f);
    }

    #[test]
    fn codec_other() {
        assert_eq!(AudioCodec::parse("alac"), AudioCodec::Other("alac".into()));
    }
}
