

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Mod {
    #[serde(rename="mod")]
    pub mod_json: ModJson,

    #[serde(default)]
    pub statuscode: Option<String>
}


#[derive(Deserialize, Serialize, Debug)]
pub struct ModJson {
    #[serde(default)]
    pub modid: u32,
    #[serde(default)]
    pub assetid: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urlalias: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logofilename: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logofile: Option<Option<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logofiledb: Option<Option<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepageurl: Option<Option<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sourcecodeurl: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trailervideourl: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuetrackerurl: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wikiurl: Option<String>,

    #[serde(default)]
    pub downloads: u32,
    #[serde(default)]
    pub follows: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,

    #[serde(default, rename = "type")]
    pub mod_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lastreleased: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lastmodified: Option<String>,
    #[serde(default)]
    pub tags: Vec<Option<String>>,
    #[serde(default)]
    pub releases: Vec<Releases>,
    #[serde(default)]
    pub screenshots: Vec<Screenshots>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Releases {
    #[serde(default)]
    pub releaseid: u32,
    #[serde(default)]
    pub mainfile: Option<String>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub fileid: u32,
    #[serde(default)]
    pub downloads: u32,
    #[serde(default)]
    pub tags: Vec<Option<String>>,
    #[serde(default)]
    pub modidstr: Option<String>,
    #[serde(default)]
    pub modversion: Option<String>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Screenshots {
    #[serde(default)]
    pub fileid: u32,
    #[serde(default)]
    pub mainfile: Option<String>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub thumbnailfilename: Option<String>,
    #[serde(default)]
    pub created: Option<String>,
}

