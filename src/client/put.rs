use stateright::actor::{Actor, Id};

use crate::GlobalMsg;

use super::ClientMsg;

/// A client strategy that just puts at a single key into a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapSinglePutter {
    pub key: String,
    pub request_count: usize,
    pub server_count: usize,
}

impl Actor for MapSinglePutter {
    type Msg = GlobalMsg;

    type State = ();

    fn on_start(
        &self,
        id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        let index: usize = id.into();
        if index < self.server_count {
            panic!("MyRegisterActor clients must be added to the model after servers.");
        }

        for i in 0..self.request_count {
            let unique_request_id = (i + 1) * index; // next will be 2 * index
            let value = (b'A' + (index % self.server_count) as u8) as char;
            let msg = ClientMsg::PutMap(unique_request_id, self.key.clone(), value.to_string());
            o.send(Id::from(index % self.server_count), GlobalMsg::Client(msg));
        }
    }
}

/// A client strategy that just puts at the start of a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListStartPutter {
    pub request_count: usize,
    pub server_count: usize,
}

impl Actor for ListStartPutter {
    type Msg = GlobalMsg;

    type State = ();

    fn on_start(
        &self,
        id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        let index: usize = id.into();
        if index < self.server_count {
            panic!("MyRegisterActor clients must be added to the model after servers.");
        }

        for i in 0..self.request_count {
            let unique_request_id = (i + 1) * index; // next will be 2 * index
            let value = (b'A' + (index % self.server_count) as u8) as char;
            let msg = ClientMsg::PutList(unique_request_id, 0, value.to_string());
            o.send(Id::from(index % self.server_count), GlobalMsg::Client(msg));
        }
    }
}
