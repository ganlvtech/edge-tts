use serde::{Deserialize, Serialize};
use ureq::get;

const VOICES_URL: &str = "https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";

// region voice list

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Voice {
    /// 例如："Microsoft Server Speech Text to Speech Voice (zh-CN, XiaoxiaoNeural)"
    #[serde(rename = "Name")]
    pub name: String,
    /// 例如："zh-CN-XiaoxiaoNeural"
    #[serde(rename = "ShortName")]
    pub short_name: String,
    /// 例如："Female"
    #[serde(rename = "Gender")]
    pub gender: String,
    /// 例如："zh-CN"
    #[serde(rename = "Locale")]
    pub locale: String,
    /// 例如："audio-24khz-48kbitrate-mono-mp3"
    #[serde(rename = "SuggestedCodec")]
    pub suggested_codec: String,
    /// 例如："Microsoft Xiaoxiao Online (Natural) - Chinese (Mainland)"
    #[serde(rename = "FriendlyName")]
    pub friendly_name: String,
    /// 例如："GA"
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "VoiceTag")]
    pub voice_tag: VoiceTag,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceTag {
    /// 例如："Cartoon", "Conversation", "Dialect", "General", "News", "Novel", "Sports"
    #[serde(rename = "ContentCategories")]
    pub content_categories: Vec<String>,
    /// 例如："Authority", "Bright", "Comfort", "Confident", "Considerate", "Cute", "Friendly", "Humorous", "Lively", "Passion", "Pleasant", "Positive", "Professional", "Rational", "Reliable", "Sunshine", "Warm"
    #[serde(rename = "VoicePersonalities")]
    pub voice_personalities: Vec<String>,
}

pub fn get_voice_list() -> anyhow::Result<Vec<Voice>> {
    Ok(get(VOICES_URL).call()?.into_json()?)
}
