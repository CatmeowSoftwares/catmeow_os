use core::ptr::null_mut;

pub struct Node<Item: 'static> {
    pub item: Item,
    pub next: *mut Node<Item>,
}

impl<Item: 'static> Node<Item> {
    fn new(item: Item) -> Self {
        Self {
            item,
            next: null_mut(),
        }
    }
}

pub struct LinkedList<Item: 'static> {
    pub head: Node<Item>,
    pub last: *mut Node<Item>,
}

impl<Item: 'static> LinkedList<Item> {
    pub fn new(item: Item) -> Self {
        let node = Node::new(item);
        let mut lili = Self {
            head: node,
            last: null_mut(),
        };
        lili.last = &raw mut lili.head;
        lili
    }
    pub fn add(&mut self, item: Item) {
        let mut new_node = Node::new(item);
        let last = unsafe { &mut *self.last };
        last.next = &raw mut new_node;
        self.last = &raw mut new_node;
    }
    pub fn add_node(&mut self, node: &mut Node<Item>) {
        //node.next = Some(&mut self.head);
    }
    /*
    pub fn add(&mut self, item: Item) {
        let mut node = &mut self.head;
        loop {
            let has_next_node = node.next.take();
            match has_next_node {
                Some(next_node) => {
                    node = next_node;
                }
                None => {
                    let mut new_node = Node::new(item);
                    let new_node_ptr = (&mut new_node) as *mut Node<Item>;
                    unsafe {
                        node.next = Some(&mut *new_node_ptr);
                    }
                    return;
                }
            }
        }
    }
     */
}
