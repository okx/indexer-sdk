use crate::Event;

#[derive(Clone)]
pub enum ZeroMQEvent {}

impl Event for ZeroMQEvent {}
