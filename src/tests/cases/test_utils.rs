use crate::tests::fakes::{
   KeyboardEvents,
};
use std::iter;

use ::termion::event::{Event, Key};

pub fn sleep_ctrl_c_keyboard_events(sleep_num:usize)->Box<KeyboardEvents> {
    let mut events:Vec<Option<Event>> = iter::repeat(None).take(sleep_num).collect();
    events.push(Some(Event::Key(Key::Ctrl('c'))));
    Box::new(KeyboardEvents::new(events))
}