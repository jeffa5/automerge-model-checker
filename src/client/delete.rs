use stateright::actor::{Actor, Id};

use crate::MyRegisterMsg;

use super::ClientMsg;

/// A client strategy that just deletes a single key in a map.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MapSingleDeleter {
    pub key: String,
    pub request_count: usize,
    pub server_count: usize,
}

impl Actor for MapSingleDeleter {
    type Msg = MyRegisterMsg;

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
            let msg = ClientMsg::DeleteMap(unique_request_id, self.key.clone());
            o.send(
                Id::from(index % self.server_count),
                MyRegisterMsg::Client(msg),
            );
        }
    }
}

/// A client strategy that just deletes the first element in a list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListDeleter {
    pub index: usize,
    pub request_count: usize,
    pub server_count: usize,
}

impl Actor for ListDeleter {
    type Msg = MyRegisterMsg;

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
            let msg = ClientMsg::DeleteList(unique_request_id, self.index);
            o.send(
                Id::from(index % self.server_count),
                MyRegisterMsg::Client(msg),
            );
        }
    }
}
