use automerge::ObjType;
use stateright::actor::{Actor, Id};

use crate::MyRegisterMsg;

use super::ClientMsg;

/// A client strategy that just inserts at the start of the list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListInserter {
    pub index: usize,
    pub request_count: usize,
    pub server_count: usize,
}

impl Actor for ListInserter {
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

        let server_id = Id::from(index % self.server_count);
        // ensure we have a list to insert into
        let unique_request_id = index; // next will be 2 * index
        o.send(
            server_id,
            MyRegisterMsg::Client(ClientMsg::PutObject(
                unique_request_id,
                "list".to_owned(),
                ObjType::List,
            )),
        );
        for i in 1..self.request_count {
            let unique_request_id = (i + 1) * index; // next will be 2 * index
            let value = (b'A' + (index % self.server_count) as u8) as char;
            let msg = ClientMsg::Insert(unique_request_id, self.index, value.to_string());
            o.send(server_id, MyRegisterMsg::Client(msg));
        }
    }
}
