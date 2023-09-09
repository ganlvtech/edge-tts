#[cfg(feature = "voice_list")]
mod voice_list;
mod synthesize;

#[cfg(feature = "voice_list")]
pub use voice_list::{get_voice_list};
pub use synthesize::{build_ssml, request_audio};
