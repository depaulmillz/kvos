use alloc::vec::Vec;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::boxed::Box;

const MAX_LEVEL: usize = 32;

fn get_rand_level(max_level: usize) -> usize {
    unsafe {
        static mut Y: u32  = 2463534242u32;
        Y ^= Y << 13;
        Y ^= Y >> 17;
        Y ^= Y << 5;
        let mut temp = Y;
        let mut level = 1;
        while (temp >> 1) & 1 != 0 {
            level += 1;
            temp >>= 1;
        }
        
        if level > max_level {
            max_level as usize
        } else {
            level as usize
        }
    }
}

struct SkipNode<K, V> {
    key: K,
    val: V,
    marker: bool,
    deleted: AtomicUsize,
    toplevel: usize,
    next: Vec<AtomicUsize>,
}

impl <K: Default + Ord + Copy, V: Default + Copy> SkipNode<K, V> {
    fn new(key: K, val: V, toplevel: usize) -> Self {
        let mut next = Vec::with_capacity(MAX_LEVEL);
        for _ in 0..MAX_LEVEL {
            next.push(AtomicUsize::new(0));
        }
        SkipNode {
            key,
            val,
            marker: false,
            deleted: AtomicUsize::new(0),
            toplevel,
            next,
        }
    }
}

impl <K: Default + Ord + Clone + Copy, V: Clone + Default + Copy> Clone for SkipNode<K, V> {
    fn clone(&self) -> Self {
        let next = self.next.iter().map(|atomic| AtomicUsize::new(atomic.load(Ordering::Relaxed))).collect();
        SkipNode {
            key: self.key,
            val: self.val,
            marker: self.marker,
            deleted: AtomicUsize::new(self.deleted.load(Ordering::Relaxed)),
            toplevel: self.toplevel,
            next,
        }
    }
}


pub struct SkipMap<K, V> {
    head: *mut SkipNode<K, V>,
}

impl <K:Default + Ord + Copy, V: Default + Copy> SkipMap<K, V> {
    pub fn new() -> Self {
        let head = Box::into_raw(Box::new(SkipNode::new(K::default(), V::default(), MAX_LEVEL)));
        let tail = Box::into_raw(Box::new(SkipNode::new(K::default(), V::default(), MAX_LEVEL)));

        //set as marked 
        unsafe {
            (*tail).marker = true;
        }

        for i in 0..MAX_LEVEL {
            unsafe {
                (*head).next[i].store(tail as usize, Ordering::Relaxed);
            }
        }

        SkipMap {
            head,
        }
    }

    fn is_marked(i: usize) -> bool {
        i & 0x01 == 0x01
    }

    fn unset_mark(i: usize) -> usize {
        i & !0x01
    }

    fn set_mark(i: usize) -> usize {
        i | 0x01
    }

    fn search(&self, key: K, left_list: &mut [*mut SkipNode<K, V>; MAX_LEVEL], right_list: &mut [*mut SkipNode<K, V>; MAX_LEVEL]) {
        let mut left: *mut SkipNode<K, V>;
        let mut _right: *mut SkipNode<K, V>;
        let mut left_next: usize;
        let mut right_next: usize;
        'retry: loop {
            left = self.head;
            for i in (0..MAX_LEVEL).rev() {
                left_next = unsafe { (*left).next[i].load(Ordering::Relaxed) };
                if Self::is_marked(left_next) {
                    continue 'retry;
                }
                let mut right = left_next as *mut SkipNode<K, V>;
                loop {
                    loop {
                        right_next = unsafe {(*right).next[i].load(Ordering::Relaxed)};

                        if !Self::is_marked(right_next) {
                            break;
                        }
                        right = Self::unset_mark(right_next) as *mut SkipNode<K, V>
                    }

                    if unsafe{(*right).marker || (*right).key >= key} {
                        break;
                    }
                    left = right;
                    left_next = right_next;
                    right = right_next as *mut SkipNode<K, V>;

                }
                if left_next != right as usize && unsafe { !(*left).next[i].compare_exchange(left_next, right as usize, Ordering::Relaxed, Ordering::Relaxed).is_ok() } {
                    continue 'retry;
                }
                left_list[i] = left;
                right_list[i] = right;

            }
            break;
        }
    }

    // ... remaining methods ...

    pub fn insert(&self, key:K, val: V) -> bool {
        let new_node = Box::into_raw(Box::new(SkipNode::new(key, val, get_rand_level(MAX_LEVEL))));
        let mut new_next: usize;
        let mut preds: [*mut SkipNode<K, V>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        let mut succs: [*mut SkipNode<K, V>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];

        loop {
            self.search(key, &mut preds, &mut succs);

            if unsafe{ (*succs[0]).key } == key {
                if unsafe{ (*succs[0]).deleted.load(Ordering::Relaxed) } == 0 {
                    Self::mark_node_ptrs(&self, succs[0]);
                    continue;
                }
                return false;
            }
            for i in 0..unsafe{(*new_node).toplevel} {
                loop {
                    let pred = preds[i];
                    let mut succ = succs[i];

                    new_next = unsafe {(*new_node).next[i].load(Ordering::Relaxed)};

                    if new_next != succ as usize && unsafe{!(*new_node).next[i].compare_exchange(Self::unset_mark(new_next), succ as usize, Ordering::Relaxed, Ordering::Relaxed).is_ok()} {
                        break;
                    }

                    if unsafe{(*succ).key} == key {
                        unsafe {succ = Self::unset_mark((*succ).next[0].load(Ordering::Relaxed)) as *mut SkipNode<K, V>}
                    }

                    if unsafe{(*pred).next[i].compare_exchange(succ as usize,  new_node as usize, Ordering::Relaxed, Ordering::Relaxed).is_ok()} {
                        break;
                    }
                    Self::search(&self, key, &mut preds, &mut succs);
                }
            }
            break true;
        }
    }

    pub fn get(&self, key: K) -> Option<V> {
        let mut preds: [*mut SkipNode<K, V>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        let mut succs: [*mut SkipNode<K, V>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        self.search(key, &mut preds, &mut succs);
        if !succs[0].is_null() && unsafe { (*succs[0]).key == key } {
            return Some(unsafe { (*succs[0]).val});
        } else {
            None
        }
    }

    pub fn remove(&self, key: K) -> bool{
        let mut preds: [*mut SkipNode<K, V>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        let mut succs: [*mut SkipNode<K, V>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        self.search(key, &mut preds, &mut succs);
        if succs[0].is_null() || unsafe { (*succs[0]).key != key } {
            return false;
        }
        if unsafe { (*succs[0]).deleted.load(Ordering::Relaxed) } != 0 {
            return false;
        }
        unsafe {
            (*succs[0]).deleted.fetch_add(1, Ordering::Relaxed);
        }
        self.mark_node_ptrs(succs[0]);
        self.search(key, &mut preds, &mut succs);
        return true;
    }

    fn mark_node_ptrs(&self, node: *mut SkipNode<K, V>) {
        let mut n_next;
        for i in (0..unsafe { (*node).toplevel }).rev() {
            loop {
                n_next = unsafe { (*node).next[i].load(Ordering::Relaxed) };
                if Self::is_marked(n_next) {
                    break;
                }
                if unsafe { (*node).next[i].compare_exchange(n_next, Self::set_mark(n_next), Ordering::Relaxed, Ordering::Relaxed).is_ok() } {
                    break;
                }
            }
        }
    }
}
