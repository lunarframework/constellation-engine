use super::math::AbstractVector;
use serde::{Deserialize, Serialize};
use std::any::Any;

// #[derive(Error, Debug)]
// pub enum RecordContextError {
//     #[error("Attachment Id {0} has not been registered yet")]
//     InvalidAttachmentId(usize),
// }

// pub struct RecordContext {
//     events: usize,
//     min_time: f64,
//     max_time: f64,
//     delta_time: f64,
// }

// impl RecordContext {
//     pub fn new(min_time: f64, max_time: f64, events: usize) -> Self {
//         let delta_time = (max_time - min_time) / (events + 1) as f64;

//         Self {
//             events,
//             min_time,
//             max_time,
//             delta_time,
//         }
//     }

//     pub fn global(&self, time: f64) -> Option<f64> {
//         if self.events == 0 || time < self.min_time || time > self.max_time {
//             None
//         } else {
//             Some((time - self.min_time) / self.delta_time - 1.0)
//         }
//     }

//     pub fn local(&self, begin: f64, time: f64) -> Option<f64> {
//         if begin > time {
//             return None;
//         }
//         Some(self.global(time)? - self.global(begin)?)
//     }
// }

#[derive(Serialize, Deserialize)]
pub struct ContinuousRecord<V: Send + Sync + Clone + AbstractVector + Any> {
    times: Vec<f64>,
    values: Vec<V>,
}

impl<V: Send + Sync + Clone + AbstractVector + Any> ContinuousRecord<V> {
    pub fn new() -> Self {
        Self {
            times: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn save(&mut self, time: f64, value: V) {
        self.times.push(time);
        self.values.push(value);
    }

    pub fn load(&self, time: f64) -> V {
        let mut index = -1;

        for i in 0..(self.times.len() - 1) {
            if time >= self.times[i] && time < self.times[i + 1] {
                index = i as i32;
            }
        }

        if index == -1 {
            return V::zero();
        }

        let interp = (time - self.times[index as usize])
            / (self.times[index as usize + 1] - self.times[index as usize]);

        let mut v = self.values[index as usize].clone();
        v.lerp(self.values[index as usize + 1].clone(), interp);
        v
    }
}

// #[derive(Serialize, Deserialize)]
// pub struct ContinuousRecord<V: Send + Sync + Clone + AbstractVector> {
//     events: usize,
//     times: Vec<f64>,
//     values: Vec<V>,
// }

// impl<V: Send + Sync + Clone + AbstractVector> ContinuousRecord<V> {
//     pub fn new(begin: f64, end: f64, events: usize) -> Self {
//         let mut times = vec![0.0; events + 2];

//         for (i, time) in times.iter_mut().enumerate() {
//             *time = i as f64 / (events + 1) as f64 * (end - begin) + begin;
//         }

//         let values = vec![V::zero(); events];

//         Self {
//             times,
//             values,
//             events,
//         }
//     }

//     pub fn begin_time(&self) -> f64 {
//         self.times[0]
//     }

//     pub fn end_time(&self) -> f64 {
//         self.times[self.events + 1]
//     }

//     pub fn events(&self) -> usize {
//         self.events
//     }

//     pub fn event_time(&mut self, index: usize) -> f64 {
//         self.times[index + 1]
//     }

//     pub fn save_begin(&mut self, value: V) {
//         self.values[0] = value;
//     }

//     pub fn save_end(&mut self, value: V) {
//         self.values[self.events + 1] = value;
//     }

//     pub fn save_event(&mut self, index: usize, value: V) {
//         self.values[index + 1] = value;
//     }

//     pub fn save_linear(&mut self, mut a: (f64, V), mut b: (f64, V)) {
//         if self.events == 0 {
//             return;
//         }

//         if a.0 > b.0 {
//             let tmp = b;
//             b = a;
//             a = tmp;
//         }

//         let delta = self.times[1] - self.times[0];

//         let a_index = ((a.0 - self.begin_time()) / delta).ceil() as i64;
//         let b_index = ((b.0 - self.begin_time()) / delta).floor() as i64;

//         for i in a_index..=b_index {
//             if let Ok(i) = <_ as TryInto<usize>>::try_into(i) {
//                 let time = self.times[i];

//                 let x = (time - a.0) / (b.0 - a.0);

//                 let mut value = a.1.clone();

//                 value.lerp(b.1.clone(), x);

//                 self.values[i] = value;
//             }
//         }
//     }

//     pub fn load_begin(&self) -> V {
//         self.values[0].clone()
//     }

//     pub fn load_end(&self) -> V {
//         self.values[self.events + 1].clone()
//     }

//     pub fn load_event(&self, index: usize) -> V {
//         self.values[index].clone()
//     }

//     pub fn load(&self, time: f64) -> V {
//         let mut total = V::zero();

//         for (i, v) in self.values.iter().enumerate() {
//             let coefficient = {
//                 let mut total = 1.0;
//                 for (j, &t) in self.times.iter().enumerate() {
//                     if i != j {
//                         total *= (time - t) / (self.begin_time() - t);
//                     }
//                 }
//                 total
//             };

//             total.add_scaled(coefficient, v.clone());
//         }

//         total
//     }
// }
