//! The bigram metric [`FingerRepeats`] incurrs a cost for bigram that uses the same finger
//! for different keys (thumb excluded). If the finger is the index, the cost may be multiplied
//! with a configurable factor (usually lessening the cost).
//!
//! *Note:* In contrast to ArneBab's version of the metric, thumbs are excluded.

use crate::sval::SvalKeyDirection;

use super::BigramMetric;

use ahash::AHashMap;
use keyboard_layout::{
    key::{Finger, FingerMap, Hand},
    layout::{LayerKey, Layout},
};

use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct Parameters {
    pub finger_factors: AHashMap<Finger, f64>,
    pub stretch_factor: f64,
    pub curl_factor: f64,
    pub lateral_factor: f64,
    pub same_key_offset: f64,
    pub thumb_bigram: f64,
}

#[derive(Clone, Debug)]
pub struct FingerRepeats {
    finger_factors: FingerMap<f64>,
    stretch_factor: f64,
    curl_factor: f64,
    lateral_factor: f64,
    same_key_offset: f64,
    thumb_bigram: f64,
}

impl FingerRepeats {
    pub fn new(params: &Parameters) -> Self {
        Self {
            finger_factors: FingerMap::with_hashmap(&params.finger_factors, 1.0),
            stretch_factor: params.stretch_factor,
            curl_factor: params.curl_factor,
            lateral_factor: params.lateral_factor,
            same_key_offset: params.same_key_offset,
            thumb_bigram: params.thumb_bigram,
        }
    }
}

impl BigramMetric for FingerRepeats {
    fn name(&self) -> &str {
        "Finger Repeats"
    }

    #[inline(always)]
    fn individual_cost(
        &self,
        k1: &LayerKey,
        k2: &LayerKey,
        weight: f64,
        _total_weight: f64,
        _layout: &Layout,
    ) -> Option<f64> {
        if (k1 == k2 && k1.is_modifier.is_some())
            || k1.key.hand != k2.key.hand
            || k1.key.finger != k2.key.finger
        {
            return Some(0.0);
        }
        let pos1 = k1.key.matrix_position;
        let center_keys = [
            (2, 2),
            (5, 2),
            (8, 2),
            (11, 2),
            (14, 2),
            (17, 2),
            (20, 2),
            (23, 2),
        ];
        let is_thumb: bool = k1.key.finger == Finger::Thumb;
        if is_thumb {
            if k1 == k2 {
                return Some(weight * 1.0);
            }
            return Some(weight * 2.0);
        }
        let closest_center = center_keys
            .iter()
            .min_by_key(|(x, y)| pos1.0.abs_diff(*x) + pos1.1.abs_diff(*y))
            .unwrap();
        let sval_key_1 = SvalKeyDirection::from_key(&k1.key, closest_center);
        let sval_key_2 = SvalKeyDirection::from_key(&k2.key, closest_center);

        // The scale:
        // 0 = as if there's no SFB, would as easily type this as alternating fingers
        // 1 = as annoying as a curling SFB on a regular keyboard

        // center-south is virtually free, count as 0
        const CENTER_SOUTH: f64 = -0.0;
        // like east-center, or north-center
        const TO_CENTER: f64 = 1.0;
        // center-north
        const CENTER_NORTH: f64 = 0.3;
        // ex center-east on a left hand, or center-west on a right hand
        // also virtualy free
        const INWARD_ROLL: f64 = 0.4;
        // ex center-west on a left hand, or center-east on a right hand
        const OUTWARD_ROLL: f64 = 3.0;
        // ex east-west or west-east
        const WALL_TO_WALL_LATERAL: f64 = 1.75;
        // ex north-south or south-north
        const WALL_TO_WALL_VERTICAL: f64 = 1.75;
        // ex south-west or east-north
        const WALL_TO_WALL_OTHER: f64 = 1.0;

        let finger_factor = self.finger_factors.get(&k1.key.finger);
        let inward_direction = if k1.key.hand == Hand::Left {
            SvalKeyDirection::East
        } else {
            SvalKeyDirection::West
        };
        if sval_key_1 == SvalKeyDirection::South
            && sval_key_2 == inward_direction
            && k1.key.finger == Finger::Index
        {
            return Some(weight * finger_factor * 0.2);
        }
        let sval_factor = match (sval_key_1, sval_key_2) {
            (_, _) if sval_key_1 == sval_key_2 => {
                // the double-presses
                match sval_key_1 {
                    SvalKeyDirection::North => 1.0,
                    SvalKeyDirection::South => 0.5,
                    SvalKeyDirection::East => 1.0,
                    SvalKeyDirection::West => 1.0,
                    SvalKeyDirection::Center => 0.7,
                }
            }
            (SvalKeyDirection::Center, _) => match sval_key_2 {
                SvalKeyDirection::South => CENTER_SOUTH,
                SvalKeyDirection::North => CENTER_NORTH,
                lateral => {
                    if lateral == inward_direction {
                        INWARD_ROLL
                    } else {
                        OUTWARD_ROLL
                    }
                }
            },
            (_, SvalKeyDirection::Center) => TO_CENTER,
            (SvalKeyDirection::West, SvalKeyDirection::East)
            | (SvalKeyDirection::East, SvalKeyDirection::West) => WALL_TO_WALL_LATERAL,
            (SvalKeyDirection::North, SvalKeyDirection::South)
            | (SvalKeyDirection::South, SvalKeyDirection::North) => WALL_TO_WALL_VERTICAL,
            (_, _) => WALL_TO_WALL_OTHER,
        };
        return Some(weight * finger_factor * sval_factor);
    }
}
