use std::{
    collections::HashMap,
    hash::Hasher,
    path::{Path, PathBuf},
    ptr,
    rc::{Rc, Weak},
    time::{Duration, Instant},
};

use sdl2::mixer::Chunk;
use weak_table::WeakValueHashMap;

/// Wrapper for `Rc<T>` that compares and hashes by pointer location.
struct RcKey<T>(Rc<T>);

impl<T> Clone for RcKey<T> {
    fn clone(&self) -> Self {
        RcKey(self.0.clone())
    }
}

impl<T> PartialEq for RcKey<T> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(Rc::as_ptr(&self.0), Rc::as_ptr(&other.0))
    }
}

impl<T> Eq for RcKey<T> {}

impl<T> std::hash::Hash for RcKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(&self.0);
        ptr.hash(state);
    }
}

/// delay dropping something until later. simple. single threaded. no real time
/// guarantees. T must never panic when dropped
pub struct RcDelayedDropper<T> {
    /// the minimum amount of time that will be delayed before something which
    /// was added to the queue will be dropped. there is no maximum amount of
    /// time - durations are only checked when elements are queued. duration
    /// will be reset if the same element is queued as before
    pub duration: Duration,

    vals: HashMap<RcKey<T>, Instant>,
}

impl<T> RcDelayedDropper<T> {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            vals: Default::default(),
        }
    }

    pub fn drop_later(&mut self, t: Rc<T>) {
        let now = Instant::now();
        self.vals.insert(RcKey(t), now);
        self.vals
            .retain(|_, &mut instant| now.duration_since(instant) < self.duration);
        if self.vals.len() > 64 {
            // ok hardcoding magic value for dev
            // warn during development. why are resources created so fast?
            debug_assert!(false);
        }
    }
}

/// associates a string key with a sound file, or loads it from disk if needed.
/// loaded sounds will be kept around for a little bit (for a time duration
/// which should cover the entirety of when they are played), but will be
/// dropped after some amount of time.
pub struct SoundManager {
    /// associate the file path with the loaded chunk
    sounds: WeakValueHashMap<PathBuf, Weak<Chunk>>,
    /// keep the chunks alive for a bit
    delay_dropper: RcDelayedDropper<Chunk>,
}

impl SoundManager {
    /// the maximum length of any sound that will be used
    pub fn new(max_duration: Duration) -> Self {
        Self {
            sounds: Default::default(),
            // x2 factor of safety. even if the chunk is dropped while the sound
            // is playing, rust-sdl2 makes the sound stop playing
            delay_dropper: RcDelayedDropper::new(max_duration * 2),
        }
    }

    /// get a sound. to be immediately played
    pub fn get(&mut self, sound_path: &Path) -> Result<Rc<Chunk>, String> {
        if let Some(v) = self.sounds.get(sound_path) {
            self.delay_dropper.drop_later(v.clone()); // refresh duration
            return Ok(v);
        }

        let chunk = Chunk::from_file(sound_path)?;
        let out = Rc::new(chunk);

        self.sounds.insert(sound_path.to_path_buf(), out.clone());
        self.delay_dropper.drop_later(out.clone());
        Ok(out)
    }
}
