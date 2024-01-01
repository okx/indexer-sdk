use crate::Event;

#[derive(Clone, Debug)]
pub enum ZeroMQEvent {}

impl Event for ZeroMQEvent {}
