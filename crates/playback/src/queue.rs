use std::{collections::VecDeque, path::PathBuf};

#[derive(Default)]
pub struct UpNext {
    current: Option<PathBuf>,
    pending: VecDeque<PathBuf>,
    history: Vec<PathBuf>,
}

impl UpNext {
    pub fn pending(&self) -> &VecDeque<PathBuf> {
        &self.pending
    }

    pub fn enqueue(&mut self, path: PathBuf) {
        self.pending.push_back(path);
    }

    pub fn set_current(&mut self, path: Option<PathBuf>) {
        self.current = path;
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<PathBuf> {
        let next = self.pending.pop_front()?;
        if let Some(current) = self.current.replace(next.clone()) {
            self.history.push(current);
        }
        Some(next)
    }

    pub fn previous(&mut self) -> Option<PathBuf> {
        let previous = self.history.pop()?;
        if let Some(current) = self.current.replace(previous.clone()) {
            self.pending.push_front(current);
        }
        Some(previous)
    }

    pub fn play_now(&mut self, path: PathBuf) {
        if let Some(current) = self.current.replace(path) {
            self.history.push(current);
        }
    }

    pub fn play_at(&mut self, index: usize) -> Option<PathBuf> {
        let path = self.pending.remove(index)?;
        self.play_now(path.clone());
        Some(path)
    }

    pub fn remove(&mut self, index: usize) -> Option<PathBuf> {
        self.pending.remove(index)
    }

    pub fn move_item(&mut self, from: usize, to: usize) -> bool {
        if from >= self.pending.len() || to >= self.pending.len() || from == to {
            return false;
        }
        let Some(item) = self.pending.remove(from) else {
            return false;
        };
        self.pending.insert(to, item);
        true
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::UpNext;
    use std::path::PathBuf;

    fn track(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    #[test]
    fn next_and_previous_keep_queue_order_and_history() {
        let mut queue = UpNext::default();
        queue.play_now(track("current.mp3"));
        queue.enqueue(track("one.mp3"));
        queue.enqueue(track("two.mp3"));

        assert_eq!(queue.next(), Some(track("one.mp3")));
        assert_eq!(queue.previous(), Some(track("current.mp3")));
        assert_eq!(queue.pending().front(), Some(&track("one.mp3")));
        assert_eq!(queue.next(), Some(track("one.mp3")));
        assert_eq!(queue.next(), Some(track("two.mp3")));
        assert_eq!(queue.next(), None);
    }

    #[test]
    fn queue_items_can_be_reordered_removed_and_cleared() {
        let mut queue = UpNext::default();
        queue.enqueue(track("one.mp3"));
        queue.enqueue(track("two.mp3"));
        queue.enqueue(track("three.mp3"));

        assert!(queue.move_item(2, 0));
        assert_eq!(queue.pending()[0], track("three.mp3"));
        assert_eq!(queue.remove(1), Some(track("one.mp3")));
        queue.clear();
        assert!(queue.pending().is_empty());
        assert!(!queue.move_item(0, 1));
    }
}
