use std::cmp::{max, min};
use std::ops::Range;

use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::Star;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StarData {
    pub pos: DVec3,
    pub vel: DVec3,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct View {
    max_time: f64,
    total_points: usize,

    stars: Vec<Star>,
    star_data: Vec<StarData>,
}

impl View {
    pub fn new(
        max_time: f64,
        interior_points: usize,
        stars: Vec<Star>,
        data: impl Fn(f64, &mut [StarData]),
    ) -> Self {
        let n = stars.len();

        let total_points = interior_points + 2;

        let mut star_data = Vec::<StarData>::new();
        star_data.resize(
            (interior_points + 2) * stars.len(),
            StarData {
                pos: DVec3::ZERO,
                vel: DVec3::ZERO,
            },
        );

        (data)(0.0, &mut star_data[0..n]);

        for i in 0..interior_points {
            let time = (i + 1) as f64 / (interior_points + 1) as f64 * max_time;

            (data)(time, &mut star_data[(i * n)..((i + 1) * n)]);
        }

        (data)(
            max_time,
            &mut star_data[((total_points - 1) * n)..(total_points * n)],
        );

        Self {
            max_time,
            total_points,
            stars,
            star_data,
        }
    }

    pub fn max_time(&self) -> f64 {
        self.max_time
    }

    pub fn total_points(&self) -> usize {
        self.total_points
    }

    pub fn moment(&self, time: f64) -> Moment<'_> {
        let prev_index = max(
            0,
            ((time / self.max_time) * (self.total_points - 1) as f64).floor() as usize,
        );

        let next_index = min(
            self.total_points - 1,
            ((time / self.max_time) * (self.total_points - 1) as f64).ceil() as usize,
        );

        let prev_time = prev_index as f64 / (self.total_points - 1) as f64 * self.max_time;
        let next_time = next_index as f64 / (self.total_points - 1) as f64 * self.max_time;

        let interp = (time - prev_time) / (next_time - prev_time);

        Moment {
            view: self,
            len: self.stars.len(),

            interp,

            prev_offset: prev_index * self.stars.len(),
            next_offset: next_index * self.stars.len(),

            current_index: 0,
        }
    }
}

pub struct Moment<'a> {
    view: &'a View,
    len: usize,

    interp: f64,

    prev_offset: usize,
    next_offset: usize,

    current_index: usize,
}

impl<'a> Iterator for Moment<'a> {
    type Item = (&'a Star, StarData);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.len {
            return None;
        }

        let prev_data = &self.view.star_data[self.prev_offset + self.current_index];
        let next_data = &self.view.star_data[self.next_offset + self.current_index];

        let data = StarData {
            pos: prev_data.pos.lerp(next_data.pos, self.interp),
            vel: prev_data.vel.lerp(next_data.vel, self.interp),
        };

        let star = &self.view.stars[self.current_index];

        self.current_index += 1;

        Some((star, data))
    }
}
