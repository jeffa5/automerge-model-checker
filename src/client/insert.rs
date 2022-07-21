use stateright::actor::{Actor, Id};

use super::Request;

/// A client strategy that just inserts at the start of the list.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ListInserter {
    pub index: usize,
    pub request_count: usize,
}

impl Actor for ListInserter {
    type Msg = Request;

    type State = ();

    fn on_start(
        &self,
        _id: stateright::actor::Id,
        o: &mut stateright::actor::Out<Self>,
    ) -> Self::State {
        // ensure we have a list to insert into
        // let unique_request_id = index; // next will be 2 * index
        // o.send(
        //     server_id,
        //     MyRegisterMsg::Client(ClientMsg::PutObject(
        //         unique_request_id,
        //         LIST_KEY.to_owned(),
        //         ObjType::List,
        //     )),
        // );
        for _ in 1..self.request_count {
            let value = 'A';
            let msg = Request::Insert(self.index, value.to_string());
            o.send(Id::from(0), msg);
        }
    }
}
