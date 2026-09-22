//! Appearance-only theme defaults; no action or target fields belong here.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Font {
    #[default]
    Sans,
    Serif,
    Mono,
    Roboto,
    OpenSans,
    Lato,
    Montserrat,
    Oswald,
    Raleway,
    Poppins,
    Nunito,
    Merriweather,
    SourceSans3,
    VikingRunes,
}
#[derive(Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TextSize {
    Small,
    #[default]
    Normal,
    Large,
}
impl TextSize {
    pub fn scale(self) -> f32 {
        match self {
            Self::Small => 0.8,
            Self::Normal => 1.,
            Self::Large => 1.2,
        }
    }
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    MakersMark,
    Dark,
    Light,
    HighContrast,
}
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "icon_size"
    )]
    pub icon_size: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<TextSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_color: Option<crate::pages::LabelColor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<crate::pages::LabelColor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_position: Option<crate::pages::LabelPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_background: Option<crate::pages::LabelBackground>,
}
fn icon_size<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u8>, D::Error> {
    let value = Option::<u8>::deserialize(d)?;
    if value.is_some_and(|n| !(10..=100).contains(&n) || n % 5 != 0) {
        return Err(serde::de::Error::custom(
            "icon size must be 10 to 100 in steps of 5",
        ));
    }
    Ok(value)
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub preset: Preset,
    #[serde(default)]
    pub appearance: Appearance,
}
#[derive(Clone, Copy)]
pub struct Style {
    pub icon_size: Option<u8>,
    pub font: Font,
    pub size: TextSize,
    pub color: [u8; 3],
    pub background: [u8; 3],
    pub accent: [u8; 3],
    pub position: crate::pages::LabelPosition,
    pub backing: bool,
}
impl Style {
    pub fn typography(self) -> crate::key_text::Typography {
        crate::key_text::Typography {
            font: match self.font {
                Font::Sans => 0,
                Font::Serif => 1,
                Font::Mono => 2,
                Font::Roboto => 3,
                Font::OpenSans => 4,
                Font::Lato => 5,
                Font::Montserrat => 6,
                Font::Oswald => 7,
                Font::Raleway => 8,
                Font::Poppins => 9,
                Font::Nunito => 10,
                Font::Merriweather => 11,
                Font::SourceSans3 => 12,
                Font::VikingRunes => 13,
            },
            scale: self.size.scale(),
        }
    }
    pub fn new(theme: Option<&Theme>, background: [u8; 3], artwork: bool, dial: bool) -> Self {
        use crate::pages::LabelPosition;
        let mut s = Self {
            icon_size: None,
            font: Font::Sans,
            size: TextSize::Normal,
            color: [240, 230, 215],
            background,
            accent: [90, 170, 255],
            position: if artwork {
                LabelPosition::Hidden
            } else {
                LabelPosition::Middle
            },
            backing: true,
        };
        if let Some(t) = theme {
            s.icon_size = Some(100);
            let (key, strip, text, accent) = match t.preset {
                Preset::MakersMark => {
                    ([107, 58, 26], [30, 34, 39], [240, 230, 215], [90, 170, 255])
                }
                Preset::Dark => ([30, 34, 39], [22, 25, 30], [245, 245, 245], [90, 170, 255]),
                Preset::Light => (
                    [236, 240, 245],
                    [242, 244, 248],
                    [24, 30, 38],
                    [25, 90, 175],
                ),
                Preset::HighContrast => ([0, 0, 0], [0, 0, 0], [255, 255, 255], [255, 225, 60]),
            };
            s.background = if dial { strip } else { key };
            s.color = text;
            s.accent = accent;
            s.position = LabelPosition::Bottom;
            s.backing = false;
            s.apply(&t.appearance);
        }
        s
    }
    pub fn apply(&mut self, a: &Appearance) {
        use crate::pages::{LabelBackground, LabelColor};
        if let Some(v) = a.icon_size {
            self.icon_size = Some(v);
        }
        if let Some(v) = a.font {
            self.font = v;
        }
        if let Some(v) = a.size {
            self.size = v;
        }
        if let Some(v) = a.label_color
            && !matches!(v, LabelColor::Default)
        {
            self.color = v.rgb();
            self.accent = v.rgb();
        }
        if let Some(v) = a.background_color
            && !matches!(v, LabelColor::Default)
        {
            self.background = v.rgb();
        }
        if let Some(v) = a.label_position {
            self.position = v;
        }
        if let Some(v) = a.label_background {
            self.backing = matches!(v, LabelBackground::Dark);
        }
    }
}
