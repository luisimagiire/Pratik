extern crate alloc;
extern crate rand;

use anyhow::Result;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Rand)]
enum Key {
    Ab,
    A,
    B,
    Bb,
    C,
    D,
    Db,
    E,
    Eb,
    F,
    G,
    Gb,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Rand)]
enum Scale {
    Major,
    Dorian,
    Lydian,
    Lydian7,
    Mixolydian,
    Minor,
    MelodicMinor,
    Altered,
    HalfWholeDim,
    WholeHalfDim,
    BebopMajor,
    Bebop7,
    BluesMajor,
    BluesMinor,
    FullDim,
    Chromatic,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Rand)]
enum Rythm {
    Samba,
    BossaNova,
    Baiao,
    PartidoAlto,
    Bebop,
    Blues,
    Waltz,
    Ballad,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Rand)]
enum PracticeType {
    Song,
    Scale,
    Improv,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SpacedRepetition {
    New,
    One,
    Seven,
    Sixteen,
    ThrityFive,
    Done,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Practice {
    practice_type: PracticeType,
    scale: Scale,
    key: Key,
    rythm: Rythm,
    created: DateTime<Utc>,
    pub repetition_lvl: SpacedRepetition,
    pub last_practiced: DateTime<Utc>,
}
/*
impl alloc::string::ToString for Practice {
    fn to_string(&self) -> String {
        format!(
            "{:?} - {:?} - {:?} - {:?}",
            self.practice_type, self.scale, self.key, self.rythm
        )
    }
}
*/
impl std::fmt::Display for Practice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lvl = match self.repetition_lvl {
            SpacedRepetition::New => "[NEW]",
            SpacedRepetition::One => "*",
            SpacedRepetition::Seven => "**",
            SpacedRepetition::Sixteen => "***",
            SpacedRepetition::ThrityFive => "****",
            SpacedRepetition::Done => "[DONE]",
        };
        write!(
            f,
            "{:} {:?} - {:?} - {:?} - {:?}",
            &lvl, self.practice_type, self.scale, self.key, self.rythm
        )
    }
}

impl Practice {
    pub fn update_practice(&mut self) {
        self.last_practiced = chrono::offset::Utc::now();
        self.repetition_lvl = match self.repetition_lvl {
            SpacedRepetition::New => SpacedRepetition::One,
            SpacedRepetition::One => SpacedRepetition::Seven,
            SpacedRepetition::Seven => SpacedRepetition::Sixteen,
            SpacedRepetition::Sixteen => SpacedRepetition::ThrityFive,
            SpacedRepetition::ThrityFive => SpacedRepetition::Done,
            SpacedRepetition::Done => SpacedRepetition::Done,
        }
    }
    pub fn needs_training(&self) -> bool {
        let diff_last = Utc::now().signed_duration_since(self.last_practiced);
        match self.repetition_lvl {
            SpacedRepetition::New => true,
            SpacedRepetition::One => diff_last.num_days() >= 1,
            SpacedRepetition::Seven => diff_last.num_days() >= 7,
            SpacedRepetition::Sixteen => diff_last.num_days() >= 16,
            SpacedRepetition::ThrityFive => diff_last.num_days() >= 35,
            SpacedRepetition::Done => false,
        }
    }
    pub fn new() -> Practice {
        let mut rng = rand::thread_rng();
        Practice {
            practice_type: rng.gen::<PracticeType>(),
            scale: rng.gen::<Scale>(),
            rythm: rng.gen::<Rythm>(),
            key: rng.gen::<Key>(),
            created: Utc::now(),
            repetition_lvl: SpacedRepetition::New,
            last_practiced: Utc::now(),
        }
    }
    pub fn to_json(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;
        Ok(json)
    }
    pub fn init_dataset(dataset_num: i32) -> Result<Vec<Practice>> {
        let e = vec![0; dataset_num as usize]
            .into_iter()
            .map(|_| Practice::new())
            .collect_vec();
        Ok(e)
    }
}
#[cfg(test)]
mod test_music {
    use super::*;

    #[test]
    fn parse_new_file() {
        let line = r#"{"practice_type": "Scale","scale": "Major","key": "Ab","created": "2021-01-02T18:30:09.453+00:00",
        "repetition_lvl": "Seven","last_practiced": "2021-01-26T18:30:09.453+00:00"}"#;
        let mut e: Practice = serde_json::from_str(&line).unwrap();

        // Updates date_last practice to 8 days ago
        e.last_practiced = Utc::now()
            .checked_sub_signed(chrono::Duration::days(8))
            .unwrap();
        assert_eq!(e.repetition_lvl, SpacedRepetition::Seven);
        assert_eq!(e.needs_training(), true);
        e.update_practice();
        assert_eq!(e.needs_training(), false);
    }
}
