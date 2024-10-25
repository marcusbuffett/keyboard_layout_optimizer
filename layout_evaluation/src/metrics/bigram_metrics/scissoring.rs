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
pub struct Parameters {}

#[derive(Clone, Debug)]
pub struct Scissoring {}

impl Scissoring {
    pub fn new(params: &Parameters) -> Self {
        Self {}
    }
}

impl BigramMetric for Scissoring {
    fn name(&self) -> &str {
        "Scissoring"
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
        if k1.key.hand != k2.key.hand {
            return Some(0.0);
        }
        if k1.key.finger.distance(&k2.key.finger) != 1 {
            return Some(0.0);
        }
        let pos1 = k1.key.matrix_position;
        let pos2 = k2.key.matrix_position;
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
        if k1.key.finger == Finger::Thumb || k2.key.finger == Finger::Thumb {
            return Some(0.);
        }
        let closest_center_1 = center_keys
            .iter()
            .min_by_key(|(x, y)| pos1.0.abs_diff(*x) + pos1.1.abs_diff(*y))
            .unwrap();
        let sval_key_1 = SvalKeyDirection::from_key(&k1.key, closest_center_1);
        let closest_center_2 = center_keys
            .iter()
            .min_by_key(|(x, y)| pos2.0.abs_diff(*x) + pos2.1.abs_diff(*y))
            .unwrap();
        let sval_key_2 = SvalKeyDirection::from_key(&k2.key, closest_center_2);
        if sval_key_1 == sval_key_2 {
            return Some(0.);
        }
        let includes_pinky = k1.key.finger == Finger::Pinky || k2.key.finger == Finger::Pinky;
        match (sval_key_1, sval_key_2) {
            (SvalKeyDirection::North, SvalKeyDirection::South)
            | (SvalKeyDirection::South, SvalKeyDirection::North)
            | (SvalKeyDirection::East, SvalKeyDirection::West)
            | (SvalKeyDirection::West, SvalKeyDirection::East) => {
                Some(weight * if includes_pinky { 1.0 } else { 0.5 })
            }
            _ => Some(0.),
        }
    }
}
