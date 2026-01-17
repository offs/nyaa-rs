use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Torrent {
    pub title: String,
    pub link: String,
    pub magnet_url: String,
    pub date: String,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
    pub size: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Category {
    All,
    Anime,
    AnimeMusicVideo,
    AnimeEnglishTranslated,
    AnimeNonEnglishTranslated,
    AnimeRaw,
}

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "0_0"),
            Self::Anime => write!(f, "1_0"),
            Self::AnimeMusicVideo => write!(f, "1_1"),
            Self::AnimeEnglishTranslated => write!(f, "1_2"),
            Self::AnimeNonEnglishTranslated => write!(f, "1_3"),
            Self::AnimeRaw => write!(f, "1_4"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    #[default]
    Date,
    Downloads,
    Seeders,
    Size,
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Date => write!(f, "id"),
            Self::Downloads => write!(f, "downloads"),
            Self::Seeders => write!(f, "seeders"),
            Self::Size => write!(f, "size"),
        }
    }
}
